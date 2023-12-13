use std::ops::ControlFlow;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

use crate::rule::NotificationRuleData;
use crate::{icons, rule::Action};
use emoji::EmojiMode;
use log::{debug, error, info};
use mini_template::macros::ValueContainer;
use notify_server::notification::Action as NotificationAction;
use notify_server::notification::Urgency;
use notify_server::CloseReason;
use notify_server::NotificationId;
use notify_server::NotifyServer;
use notify_server::{notification::Notification, Event, Observer};
use serde::Serialize;

use crate::rule::{Definition, Style};

pub struct NotificationManager<Src = NotifyServer>
where
    Src: notify_server::NotificationSource + Send + Sync + 'static,
{
    notifications: Vec<Arc<RwLock<NotificationData>>>,
    events: Vec<NotificationEvent>,
    definitions: Vec<Definition>,
    default_emoji_mode: EmojiMode,
    minimum_urgency: Arc<RwLock<MinimalUrgency>>,
    notify_server: Src,
}

impl<Src> NotificationManager<Src>
where
    Src: notify_server::NotificationSource + Send + Sync + 'static,
{
    pub fn new(
        definitions: Vec<Definition>,
        default_emoji_mode: EmojiMode,
        minimum_urgency: Arc<RwLock<MinimalUrgency>>,
        notify_server: Src,
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

        if let Some(mut n) = self.notifications
            .iter_mut()
            .filter_map(|n| n.write().ok())
            .find(|n| n.id == notification_data.id) {
            *n = notification_data;
            return
        }
        let notification = Arc::new(RwLock::new(notification_data));
        self.notifications.push(Arc::clone(&notification));
        self.events.push(NotificationEvent::Add(notification));
    }

    pub fn update(&mut self, dt: f64) {
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
            .for_each(|id| self.remove(id, &CloseReason::Expired))
    }

    pub fn get_events(&mut self) -> Vec<NotificationEvent> {
        std::mem::take(&mut self.events)
    }

    pub fn remove(&mut self, id: NotificationId, close_reason: &CloseReason) {
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
        self.notification_closed(id, close_reason);
        if let Some(n) = notification {
            self.events.push(NotificationEvent::Remove(n));
        }
    }

    //TODO Rewrite as async
    pub fn action_invoked(&mut self, id: notify_server::NotificationId, action: &str) {
        async_std::task::block_on(self.notify_server.action_invoked(id, action)).ok();
    }

    //TODO Rewrite as async
    pub fn notification_closed(&mut self, id: notify_server::NotificationId, reason: &CloseReason) {
        async_std::task::block_on(self.notify_server.notification_closed(id, reason)).ok();
    }

    #[cfg(tray_icon)]
    pub fn set_minimal_urgency(&mut self, min: Urgency) {
        self.minimum_urgency = min;
    }

    pub fn close_all_notifications(&mut self, reason: CloseReason) {
        self.notifications
            .iter()
            .filter_map(|n| n.read().ok())
            .map(|n| n.id)
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|id| self.remove(id, &reason))
    }
}

impl<Src> Observer<Event> for NotificationManager<Src>
where
    Src: notify_server::NotificationSource + Send + Sync + 'static,
{
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
) {
    execute_rules_inner(
        definitions,
        n,
        notification_template_data,
        notification_data,
    );
}
fn execute_rules_inner(
    definitions: &[Definition],
    n: &Notification,
    notification_template_data: &mut NotificationTemplateData,
    notification_data: &mut NotificationData,
) -> ControlFlow<ExecuteActionBreakReason> {
    for rule in definitions {
        use ExecuteActionBreakReason::*;
        let rule_data = NotificationRuleData {
            app_icon: &n.app_icon,
            app_name: &n.app_name,
            body: &n.body,
            expire_timeout: notification_data.expire_timeout,
            group: notification_data.group.as_deref(),
            summary: &n.summary,
            urgency: &n.urgency,
        };
        if !rule.matches(&rule_data) {
            continue;
        };
        let action_result = rule.actions.iter().try_for_each(|action| {
            excute_action(action, notification_data, notification_template_data)
        });

        match action_result {
            ControlFlow::Break(Stop) => return ControlFlow::Break(Stop),
            ControlFlow::Break(Ignore) => return ControlFlow::Break(Ignore),
            ControlFlow::Continue(_) => {}
        }

        notification_data.style.extend(rule.style.clone());

        let sub_rule_result = execute_rules_inner(
            &rule.sub_definition,
            n,
            notification_template_data,
            notification_data,
        );
        if matches!(sub_rule_result, ControlFlow::Break(_)) {
            return sub_rule_result;
        }
    }
    ControlFlow::Continue(())
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

enum ExecuteActionBreakReason {
    Stop,
    Ignore,
}

fn excute_action(
    action: &Action,
    notification_data: &mut NotificationData,
    notification_template_data: &mut NotificationTemplateData,
) -> ControlFlow<ExecuteActionBreakReason> {
    use ExecuteActionBreakReason::*;
    match action {
        Action::Ignore => {
            notification_data.ignore = true;
            ControlFlow::Break(Ignore)
        }
        Action::Set(set_property) => {
            set_property.set(notification_data, notification_template_data);
            ControlFlow::Continue(())
        }
        Action::Stop => ControlFlow::Break(Stop),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};

    use notify_server::{
        notification::{Notification, Urgency},
        CloseReason,
    };

    use crate::rule::{Action, Conditions, Definition};

    use super::{
        MinimalUrgency, NotificationData, NotificationEvent, NotificationManager,
        NotificationTemplateData,
    };

    fn very_minimal_notification_manager(
        mut notify_src: notify_server::MockNotificationSource,
    ) -> Arc<std::sync::Mutex<NotificationManager<notify_server::MockNotificationSource>>> {
        notify_src.expect_add_observer().once().returning(|_| {});
        NotificationManager::new(
            vec![Definition {
                conditions: Default::default(),
                actions: Default::default(),
                style: Default::default(),
                sub_definition: Default::default(),
            }],
            emoji::EmojiMode::Ignore,
            Arc::new(RwLock::new(MinimalUrgency::Normal)),
            notify_src,
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
            expire_timeout: 0,
        }
    }
    fn notification_template() -> NotificationTemplateData {
        NotificationTemplateData {
            app_name: "".into(),
            icon: "".into(),
            summary: "".into(),
            body: "".into(),
            expire_timeout: 0,
            time: 0,
        }
    }

    #[test]
    fn new_notification_mamager() {
        let notify_src = notify_server::MockNotificationSource::default();
        let nm = very_minimal_notification_manager(notify_src);
        let nm_l = nm.lock().unwrap();

        let _ = nm_l
            .minimum_urgency
            .write()
            .map(|mut m| *m = MinimalUrgency::Critical);

        assert_eq!(nm_l.notifications.len(), 0);
        assert_eq!(nm_l.events.len(), 0);
        assert_eq!(nm_l.definitions.len(), 1);
        assert_eq!(nm_l.default_emoji_mode, emoji::EmojiMode::Ignore);
        assert_eq!(
            *nm_l.minimum_urgency.read().unwrap(),
            MinimalUrgency::Critical
        );
    }

    #[test]
    fn notification_manager_action_invoked() {
        use mockall::predicate::eq;
        let mut notify_src = notify_server::MockNotificationSource::default();
        notify_src
            .expect_action_invoked()
            .once()
            .with(
                eq::<notify_server::NotificationId>(10.into()),
                eq("default"),
            )
            .returning(|_, _| Box::pin(async { Ok(()) }));
        let nm = very_minimal_notification_manager(notify_src);
        nm.lock().unwrap().action_invoked(10.into(), "default");
    }

    #[test]
    fn notification_manager_notification_closed() {
        use mockall::predicate::eq;
        let mut notify_src = notify_server::MockNotificationSource::default();
        notify_src
            .expect_notification_closed()
            .once()
            .with(
                eq::<notify_server::NotificationId>(10.into()),
                eq(&CloseReason::Expired),
            )
            .returning(|_, _| Box::pin(async { Ok(()) }));
        let nm = very_minimal_notification_manager(notify_src);
        nm.lock()
            .unwrap()
            .notification_closed(10.into(), &CloseReason::Expired);
    }

    #[test]
    fn notification_manager_close_all_notifications() {
        use mockall::predicate::{eq, in_iter};
        let mut notify_src = notify_server::MockNotificationSource::default();
        notify_src
            .expect_notification_closed()
            .times(3)
            .with(
                in_iter::<_, notify_server::NotificationId>(vec![1.into(), 12.into(), 13.into()]),
                eq(&CloseReason::Expired),
            )
            .returning(|_, _| Box::pin(async { Ok(()) }));
        let nm = very_minimal_notification_manager(notify_src);
        let mut nm_l = nm.lock().unwrap();
        nm_l.notifications.append(&mut vec![
            Arc::new(RwLock::new(notification(1))),
            Arc::new(RwLock::new(notification(12))),
            Arc::new(RwLock::new(notification(13))),
        ]);
        nm_l.close_all_notifications(CloseReason::Expired);
        assert!(nm_l.notifications.is_empty());
    }

    #[test]
    fn notification_manager_events() {
        let notify_src = notify_server::MockNotificationSource::default();
        let nm = very_minimal_notification_manager(notify_src);
        let mut nm_l = nm.lock().unwrap();
        nm_l.events = vec![NotificationEvent::Add(Arc::new(RwLock::new(notification(
            1,
        ))))];
        assert_eq!(nm_l.get_events().len(), 1);
        assert_eq!(nm_l.events.len(), 0);
    }

    #[test]
    fn execute_rule_ignore() {
        let n = server_notification();
        let mut ntd = notification_template();
        let mut nd = notification(0);
        super::execute_rules(
            &[Definition {
                actions: vec![Action::Ignore],
                ..Default::default()
            }],
            &n,
            &mut ntd,
            &mut nd,
        );
        assert!(nd.ignore);
    }

    #[test]
    fn execute_rule_empty() {
        let n = server_notification();
        let mut ntd = notification_template();
        let mut nd = notification(0);
        super::execute_rules(&[], &n, &mut ntd, &mut nd);
        assert!(!nd.ignore);
        assert!(nd.actions.is_empty());
        assert_eq!(nd.expire_timeout, 10);
        assert_eq!(nd.id, 0.into());
    }

    #[test]
    fn execute_rule_set_group() {
        let n = server_notification();
        let mut ntd = notification_template();
        let mut nd = notification(0);
        super::execute_rules(
            &[Definition {
                actions: vec![Action::Set(crate::rule::SetProperty::Group(
                    "TestGroup".into(),
                ))],
                ..Default::default()
            }],
            &n,
            &mut ntd,
            &mut nd,
        );
        assert_eq!(nd.group, Some("TestGroup".into()));
    }

    #[test]
    fn execute_rule_stop() {
        let n = server_notification();
        let mut ntd = notification_template();
        let mut nd = notification(0);
        super::execute_rules(
            &[
                Definition {
                    actions: vec![Action::Stop],
                    ..Default::default()
                },
                Definition {
                    actions: vec![Action::Set(crate::rule::SetProperty::Group(
                        "TestGroup".into(),
                    ))],
                    ..Default::default()
                },
            ],
            &n,
            &mut ntd,
            &mut nd,
        );
        assert!(nd.group.is_none());
    }

    #[test]
    fn execute_rule_multiple() {
        let n = server_notification();
        let mut ntd = notification_template();
        let mut nd = notification(0);
        super::execute_rules(
            &[
                Definition {
                    actions: vec![Action::Set(crate::rule::SetProperty::Icon('W'))],
                    ..Default::default()
                },
                Definition {
                    actions: vec![Action::Set(crate::rule::SetProperty::Group(
                        "TestGroup".into(),
                    ))],
                    ..Default::default()
                },
            ],
            &n,
            &mut ntd,
            &mut nd,
        );
        assert_eq!(nd.group, Some("TestGroup".into()));
        assert_eq!(nd.icon, 'W');
    }

    #[test]
    fn execute_rule_multiple_not_all_matching() {
        let n = server_notification();
        let mut ntd = notification_template();
        let mut nd = notification(0);
        super::execute_rules(
            &[
                Definition {
                    actions: vec![Action::Set(crate::rule::SetProperty::Icon('W'))],
                    ..Default::default()
                },
                Definition {
                    conditions: vec![Conditions::AppName("other name".to_string())],
                    actions: vec![Action::Set(crate::rule::SetProperty::Group(
                        "TestGroup".into(),
                    ))],
                    ..Default::default()
                },
            ],
            &n,
            &mut ntd,
            &mut nd,
        );
        assert_eq!(nd.group, None);
        assert_eq!(nd.icon, 'W');
    }

    #[test]
    fn execute_rule_sub_rule() {
        let n = server_notification();
        let mut ntd = notification_template();
        let mut nd = notification(0);
        super::execute_rules(
            &[Definition {
                actions: vec![Action::Set(crate::rule::SetProperty::Icon('W'))],
                sub_definition: vec![Definition {
                    actions: vec![Action::Set(crate::rule::SetProperty::Group(
                        "TestGroup".into(),
                    ))],
                    ..Default::default()
                }],
                ..Default::default()
            }],
            &n,
            &mut ntd,
            &mut nd,
        );
        assert_eq!(nd.group, Some("TestGroup".into()));
        assert_eq!(nd.icon, 'W');
    }

    #[test]
    fn execute_rule_stop_in_sub_rule() {
        let n = server_notification();
        let mut ntd = notification_template();
        let mut nd = notification(0);
        super::execute_rules(
            &[
                Definition {
                    actions: vec![Action::Set(crate::rule::SetProperty::Icon('W'))],
                    sub_definition: vec![Definition {
                        actions: vec![Action::Stop],
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                Definition {
                    actions: vec![Action::Set(crate::rule::SetProperty::Group(
                        "TestGroup".into(),
                    ))],
                    ..Default::default()
                },
            ],
            &n,
            &mut ntd,
            &mut nd,
        );
        assert!(nd.group.is_none());
        assert_eq!(nd.icon, 'W');
    }

    #[test]
    fn execute_rule_ignore_in_sub_rule() {
        let n = server_notification();
        let mut ntd = notification_template();
        let mut nd = notification(0);
        super::execute_rules(
            &[
                Definition {
                    actions: vec![Action::Set(crate::rule::SetProperty::Icon('W'))],
                    sub_definition: vec![Definition {
                        actions: vec![Action::Ignore],
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                Definition {
                    actions: vec![Action::Set(crate::rule::SetProperty::Group(
                        "TestGroup".into(),
                    ))],
                    ..Default::default()
                },
            ],
            &n,
            &mut ntd,
            &mut nd,
        );
        assert!(nd.ignore);
        assert!(nd.group.is_none());
        assert_eq!(nd.icon, 'W');
    }
}
