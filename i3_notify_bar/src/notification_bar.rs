use std::sync::Arc;
use std::sync::RwLock;

use crate::icons;
use crate::rule::EvalRules;
use crate::rule::RuleExcutor;
use emoji::EmojiMode;
use log::{debug, info};
use mini_template::macros::ValueContainer;
use notify_server::notification::Action as NotificationAction;
use notify_server::notification::Urgency;
use notify_server::CloseReason;
use notify_server::NotificationId;
use notify_server::NotifyServer;
use notify_server::{notification::Notification, Event};
use serde::Serialize;

use crate::rule::Style;

pub struct NotificationManager<Src = NotifyServer, RE = RuleExcutor>
where
    Src: notify_server::NotificationSource + Send + Sync + 'static,
    RE: EvalRules,
{
    notifications: Vec<Arc<RwLock<NotificationData>>>,
    rule_executor: RE,
    default_emoji_mode: EmojiMode,
    minimum_urgency: Arc<RwLock<MinimalUrgency>>,
    notify_server: Src,
    commands_rx: std::sync::mpsc::Receiver<NotificationManagerCommand>,
    commands_tx: std::sync::mpsc::Sender<NotificationManagerCommand>,
    events_tx: std::sync::mpsc::Sender<NotificationEvent>,
    events_rx: Option<std::sync::mpsc::Receiver<NotificationEvent>>,
}

pub trait InvokeAction {
    fn action_invoked(&self, id: notify_server::NotificationId, action: impl Into<String>);
}

pub trait CloseNotification {
    fn notification_closed(&self, id: notify_server::NotificationId, reason: CloseReason);
}

pub trait CloseAllNotifications {
    fn close_all_notifications(&self, reason: CloseReason);
}

impl<Src, RE> NotificationManager<Src, RE>
where
    Src: notify_server::NotificationSource + Send + Sync + 'static,
    RE: EvalRules + Send + Sync + 'static,
{
    pub fn new(
        default_emoji_mode: EmojiMode,
        minimum_urgency: Arc<RwLock<MinimalUrgency>>,
        notify_server: Src,
        rule_executor: RE,
    ) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let (events_tx, events_rx) = std::sync::mpsc::channel();
        Self {
            notifications: Vec::new(),
            rule_executor,
            default_emoji_mode,
            minimum_urgency,
            notify_server,
            commands_rx: rx,
            commands_tx: tx,
            events_tx,
            events_rx: Some(events_rx),
        }
    }

    pub fn event_channel(&mut self) -> std::sync::mpsc::Receiver<NotificationEvent> {
        self.events_rx
            .take()
            .expect("receiver has already been extracted")
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

        self.rule_executor.eval(
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

        if let Some(mut n) = self
            .notifications
            .iter_mut()
            .filter_map(|n| n.write().ok())
            .find(|n| n.id == notification_data.id)
        {
            *n = notification_data;
            return;
        }
        let notification = Arc::new(RwLock::new(notification_data));
        self.notifications.push(Arc::clone(&notification));
        self.events_tx
            .send(NotificationEvent::Add(notification))
            .ok();
    }

    pub async fn update(&mut self, dt: f64) {
        for cmd in self.commands_rx.try_iter() {
            match cmd {
                NotificationManagerCommand::ActionInvoked { id, action } => {
                    self.notify_server.action_invoked(id, &action).await.ok();
                }
                NotificationManagerCommand::CloseNotification { id, reason } => {
                    self.notify_server
                        .notification_closed(id, &reason)
                        .await
                        .ok();
                }
                NotificationManagerCommand::CloseAll { reason } => {
                    for n in self
                        .notifications
                        .iter()
                        .map(|e| e.read().unwrap_or_else(|e| e.into_inner()))
                        .map(|n| n.id)
                    {
                        self.notify_server
                            .notification_closed(n, &reason)
                            .await
                            .ok();
                    }
                }
            }
        }
        if let Some(events) = self.notify_server.take_events() {
            for event in events {
                match event {
                    Event::Notify(n) => self.notify(&n),
                    Event::Close(id, CloseReason::RequesedByClient) => self
                        .notify_server
                        .notification_closed(id, &CloseReason::Undefined)
                        .await
                        .unwrap_or(()),
                    Event::Close(id, reason) => self.remove(id, reason),
                }
            }
        }

        let mut ids_to_be_removed = Vec::new();
        for n in &self.notifications {
            let Ok(mut n) = n.write() else {
                log::error!("Unable to lock notification for write");
                continue;
            };
            let Some(rm) = n.remove_in_secs.as_mut() else {
                continue;
            };
            *rm -= dt;
            if *rm <= 0. {
                ids_to_be_removed.push(n.id);
            }
        }
        ids_to_be_removed
            .into_iter()
            .for_each(|id| self.remove(id, CloseReason::Expired))
    }

    fn remove(&mut self, id: NotificationId, close_reason: CloseReason) {
        log::debug!("Close notification id: {id} reason: {close_reason:?}");
        let mut notification = None;
        self.notifications.retain(|n| match n.read() {
            Ok(n_l) if n_l.id == id => {
                notification = Some(Arc::clone(n));
                false
            }
            Err(n_l) if n_l.get_ref().id == id => {
                notification = Some(Arc::clone(n));
                false
            }
            Err(_) | Ok(_) => true,
        });
        if let Some(n) = notification {
            log::debug!("Found notification to close {id}");
            self.events_tx.send(NotificationEvent::Remove(n)).ok();
        }
    }

    #[cfg(tray_icon)]
    pub fn set_minimal_urgency(&mut self, min: Urgency) {
        self.minimum_urgency = min;
    }

    pub fn linked_commands(&self) -> NotificationManagerCommands {
        NotificationManagerCommands {
            commands: self.commands_tx.clone(),
        }
    }
}

impl<Src, RE> InvokeAction for NotificationManager<Src, RE>
where
    Src: notify_server::NotificationSource + Send + Sync + 'static,
    RE: EvalRules + Send + Sync + 'static,
{
    fn action_invoked(&self, id: notify_server::NotificationId, action: impl Into<String>) {
        self.commands_tx
            .send(NotificationManagerCommand::ActionInvoked {
                id,
                action: action.into(),
            })
            .ok();
    }
}

impl<Src, RE> CloseNotification for NotificationManager<Src, RE>
where
    Src: notify_server::NotificationSource + Send + Sync + 'static,
    RE: EvalRules + Send + Sync + 'static,
{
    fn notification_closed(&self, id: notify_server::NotificationId, reason: CloseReason) {
        self.commands_tx
            .send(NotificationManagerCommand::CloseNotification { id, reason })
            .ok();
    }
}

impl<Src, RE> CloseAllNotifications for NotificationManager<Src, RE>
where
    Src: notify_server::NotificationSource + Send + Sync + 'static,
    RE: EvalRules + Send + Sync + 'static,
{
    fn close_all_notifications(&self, reason: CloseReason) {
        self.commands_tx
            .send(NotificationManagerCommand::CloseAll { reason })
            .ok();
    }
}

pub struct NotificationManagerCommands {
    commands: std::sync::mpsc::Sender<NotificationManagerCommand>,
}

impl InvokeAction for NotificationManagerCommands {
    fn action_invoked(&self, id: notify_server::NotificationId, action: impl Into<String>) {
        self.commands
            .send(NotificationManagerCommand::ActionInvoked {
                id,
                action: action.into(),
            })
            .ok();
    }
}

impl CloseNotification for NotificationManagerCommands {
    fn notification_closed(&self, id: notify_server::NotificationId, reason: CloseReason) {
        self.commands
            .send(NotificationManagerCommand::CloseNotification { id, reason })
            .ok();
    }
}

impl CloseAllNotifications for NotificationManagerCommands {
    fn close_all_notifications(&self, reason: CloseReason) {
        self.commands
            .send(NotificationManagerCommand::CloseAll { reason })
            .ok();
    }
}

impl Clone for NotificationManagerCommands {
    fn clone(&self) -> Self {
        Self {
            commands: self.commands.clone(),
        }
    }
}

enum NotificationManagerCommand {
    ActionInvoked {
        id: notify_server::NotificationId,
        action: String,
    },
    CloseNotification {
        id: notify_server::NotificationId,
        reason: CloseReason,
    },
    CloseAll {
        reason: CloseReason,
    },
}

#[derive(Debug)]
pub enum NotificationEvent {
    Remove(Arc<RwLock<NotificationData>>),
    Add(Arc<RwLock<NotificationData>>),
}

#[derive(Debug)]
pub struct NotificationData {
    pub id: notify_server::NotificationId,
    pub notification_update_id: usize,
    pub expire_timeout: i32,
    pub remove_in_secs: Option<f64>,
    pub icon: char,
    pub text: String,
    pub style: Vec<Style>,
    pub emoji_mode: EmojiMode,
    pub ignore: bool,
    pub actions: Vec<NotificationAction>,
    pub group: Option<String>,
}

impl NotificationData {
    pub fn new(notification: &Notification, emoji_mode: EmojiMode) -> Self {
        use std::sync::atomic;
        static NOTIFY_EVENT_ID: atomic::AtomicUsize = atomic::AtomicUsize::new(0);
        Self {
            expire_timeout: notification.expire_timeout,
            remove_in_secs: if notification.expire_timeout < 0 {
                None
            } else {
                Some(notification.expire_timeout as f64)
            },
            icon: icons::get_icon(&notification.app_name).unwrap_or(' '),
            id: notification.id,
            notification_update_id: NOTIFY_EVENT_ID.fetch_add(1, atomic::Ordering::Relaxed),
            style: Vec::new(),
            text: notification.summary.clone(),
            emoji_mode,
            ignore: false,
            actions: notification.actions.clone(),
            group: None,
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

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};

    use notify_server::{
        notification::{Notification, Urgency},
        CloseReason,
    };

    use crate::{
        notification_bar::{CloseAllNotifications as _, CloseNotification as _, InvokeAction as _},
        rule::{EvalRules, RuleExcutor},
    };

    use super::{MinimalUrgency, NotificationData, NotificationManager};

    fn minimal_notification_manager<RE: EvalRules + Send + Sync + 'static>(
        notify_src: notify_server::MockNotificationSource,
        rule_evaluator: RE,
    ) -> NotificationManager<notify_server::MockNotificationSource, RE> {
        NotificationManager::new(
            emoji::EmojiMode::Ignore,
            Arc::new(RwLock::new(MinimalUrgency::Normal)),
            notify_src,
            rule_evaluator,
        )
    }

    fn notification(id: impl Into<notify_server::NotificationId>) -> NotificationData {
        NotificationData {
            actions: Vec::default(),
            emoji_mode: emoji::EmojiMode::Ignore,
            expire_timeout: 10,
            group: None,
            icon: ' ',
            id: id.into(),
            ignore: false,
            notification_update_id: 1,
            remove_in_secs: None,
            style: Default::default(),
            text: Default::default(),
        }
    }

    fn server_notification() -> Notification {
        notify_server::notification::Notification {
            app_name: "".into(),
            id: 0.into(),
            app_icon: "".into(),
            summary: "".into(),
            body: "".into(),
            urgency: Urgency::Normal,
            actions: vec![],
            expire_timeout: -1,
        }
    }

    #[test]
    fn new_notification_mamager() {
        let notify_src = notify_server::MockNotificationSource::default();
        let nm = minimal_notification_manager(notify_src, RuleExcutor::new(vec![]));

        let _ = nm
            .minimum_urgency
            .write()
            .map(|mut m| *m = MinimalUrgency::Critical);

        assert_eq!(nm.notifications.len(), 0);
        assert_eq!(nm.default_emoji_mode, emoji::EmojiMode::Ignore);
        assert_eq!(
            *nm.minimum_urgency.read().unwrap(),
            MinimalUrgency::Critical
        );
    }

    #[test]
    fn notification_manager_notify() {
        let notify_src = notify_server::MockNotificationSource::default();
        let mut nm = minimal_notification_manager(notify_src, RuleExcutor::new(vec![]));
        let mut notification = server_notification();
        assert_eq!(nm.notifications.len(), 0);
        nm.notify(&notification);
        assert_eq!(nm.notifications.len(), 1);
        nm.notify(&notification);
        // Both notifications have the same id
        assert_eq!(nm.notifications.len(), 1);
        notification.id = 1.into();
        nm.notify(&notification);
        assert_eq!(nm.notifications.len(), 2);
    }

    #[test]
    fn notification_manager_notify_urgency_check() {
        let notify_src = notify_server::MockNotificationSource::default();
        let mut nm = minimal_notification_manager(notify_src, RuleExcutor::new(vec![]));

        let mut urgency = nm.minimum_urgency.write().unwrap();
        *urgency = MinimalUrgency::Critical;
        drop(urgency);

        let mut notification = server_notification();
        assert_eq!(nm.notifications.len(), 0);
        nm.notify(&notification);
        assert_eq!(nm.notifications.len(), 0);
        notification.urgency = Urgency::Critical;
        nm.notify(&notification);
        assert_eq!(nm.notifications.len(), 1);
    }

    #[async_std::test]
    async fn notification_manager_action_invoked() {
        use mockall::predicate::eq;
        let mut notify_src = notify_server::MockNotificationSource::default();
        notify_src
            .expect_action_invoked()
            .once()
            .with(
                eq::<notify_server::NotificationId>(10.into()),
                eq("default"),
            )
            .returning(|_, _| Ok(()));
        notify_src.expect_take_events().once().returning(|| None);
        let mut nm = minimal_notification_manager(notify_src, RuleExcutor::new(vec![]));
        nm.action_invoked(10.into(), "default");
        nm.update(0.0).await;
    }

    #[async_std::test]
    async fn notification_manager_notification_closed() {
        use mockall::predicate::eq;
        let mut notify_src = notify_server::MockNotificationSource::default();
        notify_src
            .expect_notification_closed()
            .once()
            .with(
                eq::<notify_server::NotificationId>(10.into()),
                eq(&CloseReason::Expired),
            )
            .returning(|_, _| Ok(()));
        notify_src.expect_take_events().once().returning(|| None);
        let mut nm = minimal_notification_manager(notify_src, RuleExcutor::new(vec![]));
        nm.notification_closed(10.into(), CloseReason::Expired);
        nm.update(0.0).await;
    }

    #[async_std::test]
    async fn notification_manager_close_all_notifications() {
        use mockall::predicate::{eq, in_iter};
        let notify_src = notify_server::MockNotificationSource::default();
        let mut nm = minimal_notification_manager(notify_src, RuleExcutor::new(vec![]));
        nm.notifications.append(&mut vec![
            Arc::new(RwLock::new(notification(1))),
            Arc::new(RwLock::new(notification(12))),
            Arc::new(RwLock::new(notification(13))),
        ]);
        nm.close_all_notifications(CloseReason::Expired);

        nm.notify_server
            .expect_notification_closed()
            .times(3)
            .with(
                in_iter::<_, notify_server::NotificationId>(vec![1.into(), 12.into(), 13.into()]),
                eq(&CloseReason::Expired),
            )
            .returning(|_, _| Ok(()));
        nm.notify_server.expect_take_events().once().returning(|| {
            use notify_server::Event::Close;
            Some(vec![
                Close(1.into(), CloseReason::Expired),
                Close(12.into(), CloseReason::Expired),
                Close(13.into(), CloseReason::Expired),
            ])
        });
        nm.update(0.0).await;
        assert!(nm.notifications.is_empty());
    }
}
