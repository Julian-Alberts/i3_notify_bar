use std::sync::Arc;
use std::sync::RwLock;
use std::time::SystemTime;

use crate::emoji::EmojiMode;
use crate::{icons, rule::Action};
use log::{debug, info};
use notify_server::{notification::Notification, Event, Observer};
use serde::Serialize;

use crate::rule::{Definition, Style};

pub struct NotificationManager {
    notifications: Vec<Arc<RwLock<NotificationData>>>,
    events: Vec<NotificationEvent>,
    definitions: Vec<Definition>,
    default_emoji_mode: EmojiMode,
}

impl NotificationManager {
    pub fn new(definitions: Vec<Definition>, default_emoji_mode: EmojiMode) -> Self {
        Self {
            notifications: Vec::new(),
            events: Vec::new(),
            definitions,
            default_emoji_mode,
        }
    }

    fn notify(&mut self, n: &Notification) {
        info!(
            r#"Got new notification app_name "{}" summary "{}" body "{}""#,
            n.app_name, n.summary, n.body
        );
        debug!("Notification: {:#?}", n);

        let mut notification_data = NotificationData {
            expire_timeout: n.expire_timeout,
            icon: icons::get_icon(&n.app_name).unwrap_or(' '),
            id: n.id.to_string(),
            style: Vec::new(),
            text: n.summary.clone(),
            emoji_mode: self.default_emoji_mode.clone(),
        };
        debug!("Notification Data: {:#?}", notification_data);

        let notification_template_data = NotificationTemplateData {
            app_name: n.app_name.clone(),
            icon: n.app_icon.clone(),
            summary: n.summary.clone(),
            body: n.body.clone(),
            expire_timeout: n.expire_timeout,
            time: SystemTime::now(),
        };
        debug!(
            "Notification Tempalate Data: {:#?}",
            notification_template_data
        );

        let mut last_definition_id = 0;
        let mut read_next_definition = true;

        while read_next_definition {
            let definition = self.definitions[last_definition_id..]
                .iter()
                .enumerate()
                .find(|(_, r)| r.matches(&n));

            match definition {
                Some((index, definition)) => {
                    debug!(
                        "Matched definition {} {:#?}",
                        last_definition_id + index,
                        definition.conditions
                    );
                    last_definition_id += index + 1;

                    for action in &definition.actions {
                        match action {
                            Action::Ignore => {
                                debug!("Ignore Message");
                                return;
                            }
                            Action::Set(set_property) => set_property
                                .set(&mut notification_data, &notification_template_data),
                            Action::Stop => read_next_definition = false,
                        }
                    }

                    notification_data.style.extend(definition.style.clone());
                }
                None => read_next_definition = false,
            }
        }

        debug!("Finished definitions");
        debug!("Final notification_data {:#?}", notification_data);

        let notification_position = self
            .notifications
            .iter()
            .position(|n| n.read().unwrap().id == notification_data.id);
        match notification_position {
            Some(index) => {
                let mut notification_data_storage = self.notifications[index].write().unwrap();
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
        std::mem::replace(&mut self.events, Vec::new())
    }

    pub fn remove(&mut self, id: &str) {
        let id = id.to_owned();
        let filtered_notifications = self
            .notifications
            .iter()
            .filter(|n| n.read().unwrap().id == id)
            .map(Arc::clone)
            .collect::<Vec<Arc<RwLock<NotificationData>>>>();
        self.notifications = filtered_notifications;
        self.events.push(NotificationEvent::Remove(id));
    }
}

impl Observer<Event> for NotificationManager {
    fn on_notify(&mut self, event: &Event) {
        match event {
            Event::Notify(n) => self.notify(n),
        }
    }
}

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
