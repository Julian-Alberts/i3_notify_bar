use std::ops::ControlFlow;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

use crate::debug_config::MatchedDefinitionTree;
use crate::{icons, rule::Action};
use emoji::EmojiMode;
use log::{debug, error, info};
use mini_template::macros::ValueContainer;
use notify_server::notification::Action as NotificationAction;
use notify_server::notification::Urgency;
use notify_server::CloseReason;
use notify_server::NotifyServer;
use notify_server::{notification::Notification, Event, Observer};
use serde::Serialize;

use crate::rule::{Definition, Style};

pub struct NotificationManager {
    notifications: Vec<Arc<RwLock<NotificationData>>>,
    events: Vec<NotificationEvent>,
    definitions: Vec<Definition>,
    default_emoji_mode: EmojiMode,
    minimum_urgency: Arc<RwLock<MinimalUrgency>>,
    notify_server: NotifyServer,
}

impl NotificationManager {
    pub fn new(
        definitions: Vec<Definition>,
        default_emoji_mode: EmojiMode,
        minimum_urgency: Arc<RwLock<MinimalUrgency>>,
        notify_server: NotifyServer,
    ) -> Arc<Mutex<Self>> {
        let manager = Self {
            notifications: Vec::new(),
            events: Vec::new(),
            definitions,
            default_emoji_mode,
            minimum_urgency,
            notify_server,
        };

        let manager = Arc::new(Mutex::new(manager));
        let manager_cp = Arc::clone(&manager);
        manager
            .lock()
            .expect("Failed to lock notification manager")
            .notify_server
            .add_observer(manager_cp);

        manager
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

        let mut notification_template_data = NotificationTemplateData::from(notification);
        debug!(
            "Notification Tempalate Data: {:#?}",
            notification_template_data
        );

        if *self
            .minimum_urgency
            .read()
            .expect("Could not access urgency")
            > notification.urgency
        {
            return;
        }

        execute_rules(
            &self.definitions,
            notification,
            &mut notification_template_data,
            &mut notification_data,
        );
        drop(notification_template_data);

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

    pub fn remove(&mut self, id: u32, close_reason: &CloseReason) {
        let filtered_notifications = self
            .notifications
            .iter()
            .filter(|n| match n.read() {
                Ok(n) => n.id == id,
                Err(_) => {
                    error!("Could not lock notification data");
                    false
                }
            })
            .map(Arc::clone)
            .collect::<Vec<Arc<RwLock<NotificationData>>>>();
        self.notifications = filtered_notifications;
        self.notification_closed(id, close_reason);
        self.events.push(NotificationEvent::Remove(id));
    }

    //TODO Rewrite as async
    pub fn action_invoked(&mut self, id: u32, action: &str) {
        async_std::task::block_on(self.notify_server.action_invoked(id, action)).ok();
    }

    //TODO Rewrite as async
    pub fn notification_closed(&mut self, id: u32, reason: &CloseReason) {
        async_std::task::block_on(self.notify_server.notification_closed(id, reason)).ok();
    }

    #[cfg(tray_icon)]
    pub fn set_minimal_urgency(&mut self, min: Urgency) {
        self.minimum_urgency = min;
    }

    pub fn close_all_notifications(&mut self, reason: CloseReason) {
        let ids = self
            .notifications
            .iter()
            .filter_map(|n| match n.read() {
                Ok(n) => Some(n.id),
                Err(_) => {
                    error!("Could not lock notification data");
                    None
                }
            })
            .collect::<Vec<_>>();
        for id in ids {
            self.remove(id, &reason)
        }
    }
}

impl Observer<Event> for NotificationManager {
    fn on_notify(&mut self, event: &Event) {
        match event {
            Event::Notify(n) => self.notify(n),
            Event::Close(id, reason) => self.remove(*id, reason),
        }
    }
}

pub fn execute_rules(
    definitions: &[Definition],
    n: &Notification,
    notification_template_data: &mut NotificationTemplateData,
    notification_data: &mut NotificationData,
) -> super::debug_config::MatchedDefinitionTree {
    let mut matched_rules = MatchedDefinitionTree::new_root();
    execute_rules_inner(
        definitions,
        n,
        notification_template_data,
        notification_data,
        &mut matched_rules,
    );
    matched_rules
}
pub fn execute_rules_inner(
    definitions: &[Definition],
    n: &Notification,
    notification_template_data: &mut NotificationTemplateData,
    notification_data: &mut NotificationData,
    matched_rules: &mut MatchedDefinitionTree,
) -> ControlFlow<()> {
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
                let mut sub_branch = MatchedDefinitionTree::new(last_definition_id);

                for action in &definition.actions {
                    match action {
                        Action::Ignore => {
                            debug!("Ignore Message");
                            notification_data.ignore = true;
                            matched_rules.add_branch(sub_branch);
                            return ControlFlow::Break(());
                        }
                        Action::Set(set_property) => {
                            set_property.set(notification_data, notification_template_data)
                        }
                        Action::Stop => {
                            matched_rules.add_branch(sub_branch);
                            return ControlFlow::Break(());
                        }
                    }
                }

                notification_data.style.extend(definition.style.clone());

                let break_execution = execute_rules_inner(
                    &definition.sub_definition,
                    n,
                    notification_template_data,
                    notification_data,
                    &mut sub_branch,
                );
                matched_rules.add_branch(sub_branch);
                if let ControlFlow::Break(_) = break_execution {
                    return break_execution;
                }
            }
            None => read_next_definition = false,
        }
    }
    ControlFlow::Continue(())
}

#[derive(Debug)]
pub enum NotificationEvent {
    Remove(u32),
    Add(Arc<RwLock<NotificationData>>),
    Update(Arc<RwLock<NotificationData>>),
}

#[derive(Debug)]
pub struct NotificationData {
    pub id: u32,
    pub expire_timeout: i32,
    pub icon: char,
    pub text: String,
    pub style: Vec<Style>,
    pub emoji_mode: EmojiMode,
    pub ignore: bool,
    pub actions: Vec<NotificationAction>,
}

impl NotificationData {
    pub fn new(notification: &Notification, emoji_mode: EmojiMode) -> Self {
        Self {
            expire_timeout: notification.expire_timeout,
            icon: icons::get_icon(&notification.app_name).unwrap_or(' '),
            id: notification.id,
            style: Vec::new(),
            text: notification.summary.clone(),
            emoji_mode,
            ignore: false,
            actions: notification.actions.clone(),
        }
    }
}

#[derive(Debug, Serialize, ValueContainer, Clone)]
pub struct NotificationTemplateData {
    pub app_name: String,
    pub icon: String,
    pub summary: String,
    pub body: String,
    pub expire_timeout: i32,
    pub time: i64,
}

impl From<&Notification> for NotificationTemplateData {
    fn from(notification: &Notification) -> Self {
        Self {
            app_name: notification.app_name.clone(),
            icon: notification.app_icon.clone(),
            summary: notification.summary.clone(),
            body: notification.body.clone(),
            expire_timeout: notification.expire_timeout,
            time: chrono::Local::now().timestamp(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum MinimalUrgency {
    All = 0,
    Normal = 1,
    Critical = 2,
    None = 3,
}

impl std::cmp::PartialEq<Urgency> for MinimalUrgency {
    fn eq(&self, other: &Urgency) -> bool {
        *self as usize == *other as usize
    }
}

impl std::cmp::PartialOrd<Urgency> for MinimalUrgency {
    fn ge(&self, other: &Urgency) -> bool {
        *self as usize >= *other as usize
    }

    fn gt(&self, other: &Urgency) -> bool {
        *self as usize > *other as usize
    }

    fn le(&self, other: &Urgency) -> bool {
        *self as usize <= *other as usize
    }

    fn lt(&self, other: &Urgency) -> bool {
        (*self as usize) < *other as usize
    }

    fn partial_cmp(&self, other: &Urgency) -> Option<std::cmp::Ordering> {
        Some((*self as usize).cmp(&(*other as usize)))
    }
}
