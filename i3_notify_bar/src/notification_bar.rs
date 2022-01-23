use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::time::SystemTime;

use crate::emoji::EmojiMode;
use crate::{icons, rule::Action};
use log::{debug, error, info};
use notify_server::notification::Urgency;
use notify_server::{notification::Notification, Event, Observer};
use serde::Serialize;

use crate::rule::{Definition, Style};

pub struct NotificationManager {
    notifications: Vec<Arc<RwLock<NotificationData>>>,
    events: Vec<NotificationEvent>,
    definitions: Vec<Definition>,
    default_emoji_mode: EmojiMode,
    minimum_urgency: Arc<Mutex<Urgency>>
}

impl NotificationManager {
    pub fn new(definitions: Vec<Definition>, default_emoji_mode: EmojiMode, minimum_urgency: Arc<Mutex<Urgency>>) -> Self {
        Self {
            notifications: Vec::new(),
            events: Vec::new(),
            definitions,
            default_emoji_mode,
            minimum_urgency
        }
    }

    fn notify(&mut self, notification: &Notification) {
        info!(
            r#"Got new notification app_name "{}" summary "{}" body "{}""#,
            notification.app_name, notification.summary, notification.body
        );
        debug!("Notification: {:#?}", notification);

        let mut notification_data =
            NotificationData::new(notification, self.default_emoji_mode.clone());
        debug!("Notification Data: {:#?}", notification_data);

        let notification_template_data = NotificationTemplateData {
            app_name: notification.app_name.clone(),
            icon: notification.app_icon.clone(),
            summary: notification.summary.clone(),
            body: notification.body.clone(),
            expire_timeout: notification.expire_timeout,
            time: SystemTime::now(),
        };
        debug!(
            "Notification Tempalate Data: {:#?}",
            notification_template_data
        );

        if notification.urgency < self.minimum_urgency.lock().unwrap().clone() {
            return;
        }

        execute_rules(
            &self.definitions,
            notification,
            notification_template_data,
            &mut notification_data,
        );

        if notification_data.ignore {
            return;
        }

        debug!("Finished definitions");
        debug!("Final notification_data {:#?}", notification_data);

        let notification_position = self.notifications.iter().position(|n| match n.read() {
            Ok(n) => n.id == notification_data.id,
            Err(_) => {
                error!("Could not read notification data");
                false
            }
        });
        match notification_position {
            Some(index) => {
                let mut notification_data_storage = match self.notifications[index].write() {
                    Ok(nds) => nds,
                    Err(_) => {
                        error!("Could not lock notifications");
                        return;
                    }
                };
                *notification_data_storage = notification_data;
                drop(notification_data_storage);
                self.events.push(NotificationEvent::Update(Arc::clone(
                    &self.notifications[index],
                )));
            }
            None => {
                let notification = Arc::new(RwLock::new(notification_data));
                self.notifications.push(Arc::clone(&notification));
                self.events.push(NotificationEvent::Add(notification));
            }
        }
    }

    pub fn get_events(&mut self) -> Vec<NotificationEvent> {
        std::mem::take(&mut self.events)
    }

    pub fn remove(&mut self, id: &str) {
        let id = id.to_owned();
        let filtered_notifications = self
            .notifications
            .iter()
            .filter(|n| match n.read() {
                Ok(n) => n.id == id,
                Err(_) => {
                    error!("Could not read notification data");
                    false
                }
            })
            .map(Arc::clone)
            .collect::<Vec<Arc<RwLock<NotificationData>>>>();
        self.notifications = filtered_notifications;
        self.events.push(NotificationEvent::Remove(id));
    }

    #[cfg(tray_icon)]
    pub fn set_minimal_urgency(&mut self, min: Urgency) {
        self.minimum_urgency = min;
    }
}

impl Observer<Event> for NotificationManager {
    fn on_notify(&mut self, event: &Event) {
        match event {
            Event::Notify(n) => self.notify(n),
        }
    }
}

pub fn execute_rules(
    definitions: &[Definition],
    n: &Notification,
    notification_template_data: NotificationTemplateData,
    notification_data: &mut NotificationData,
) -> Vec<usize> {
    let mut matched_rules = Vec::new();
    let mut last_definition_id = 0;
    let mut read_next_definition = true;

    while read_next_definition {
        let definition = definitions[last_definition_id..]
            .iter()
            .enumerate()
            .find(|(_, r)| r.matches(n));

        match definition {
            Some((index, definition)) => {
                debug!(
                    "Matched definition {} {:#?}",
                    last_definition_id + index,
                    definition.conditions
                );
                last_definition_id += index + 1;
                matched_rules.push(last_definition_id);

                for action in &definition.actions {
                    match action {
                        Action::Ignore => {
                            debug!("Ignore Message");
                            notification_data.ignore = true;
                            return matched_rules;
                        }
                        Action::Set(set_property) => {
                            set_property.set(notification_data, &notification_template_data)
                        }
                        Action::Stop => read_next_definition = false,
                    }
                }

                notification_data.style.extend(definition.style.clone());
            }
            None => read_next_definition = false,
        }
    }

    matched_rules
}

#[derive(Debug)]
pub enum NotificationEvent {
    Remove(String),
    Add(Arc<RwLock<NotificationData>>),
    Update(Arc<RwLock<NotificationData>>),
}

#[derive(Debug)]
pub struct NotificationData {
    pub id: String,
    pub expire_timeout: i32,
    pub icon: char,
    pub text: String,
    pub style: Vec<Style>,
    pub emoji_mode: EmojiMode,
    pub ignore: bool,
}

impl NotificationData {
    pub fn new(notification: &Notification, emoji_mode: EmojiMode) -> Self {
        Self {
            expire_timeout: notification.expire_timeout,
            icon: icons::get_icon(&notification.app_name).unwrap_or(' '),
            id: notification.id.to_string(),
            style: Vec::new(),
            text: notification.summary.clone(),
            emoji_mode,
            ignore: false,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct NotificationTemplateData {
    pub app_name: String,
    pub icon: String,
    pub summary: String,
    pub body: String,
    pub expire_timeout: i32,
    pub time: SystemTime,
}

impl From<&Notification> for NotificationTemplateData {
    fn from(notification: &Notification) -> Self {
        Self {
            app_name: notification.app_name.clone(),
            icon: notification.app_icon.clone(),
            summary: notification.summary.clone(),
            body: notification.body.clone(),
            expire_timeout: notification.expire_timeout,
            time: SystemTime::now(),
        }
    }
}
