use observer::SingleEventSystem;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use zbus::blocking::InterfaceRef;
#[cfg(not(test))]
use zbus::blocking::{Connection, ConnectionBuilder};
use zbus::zvariant::Value;
use zbus::{dbus_interface, SignalContext};

use crate::notification::NotificationBuilder;
use crate::{Event, NotificationId};

pub struct NotifyServer {
    interface_ref: InterfaceRef<NotifyServerInterface>,
}

impl NotifyServer {
    // Calling this method in tests could break the notification service of operating systems.
    #[cfg(not(test))]
    pub fn start() -> zbus::Result<Self> {
        let interface = NotifyServerInterface::default();
        let connection = interface.run()?;
        let i = connection
            .object_server()
            .interface::<_, NotifyServerInterface>("/org/freedesktop/Notifications")?;
        Ok(Self { interface_ref: i })
    }

    pub fn add_observer(
        &mut self,
        observer: Arc<Mutex<dyn observer::Observer<Event> + Send + Sync + 'static>>,
    ) {
        self.interface_ref.get_mut().add_observer(observer)
    }

    pub async fn action_invoked(
        &self,
        NotificationId(id): NotificationId,
        action: &str,
    ) -> zbus::Result<()> {
        let context = self.interface_ref.signal_context();
        NotifyServerInterface::action_invoked(context, id, action).await
    }

    pub async fn notification_closed(
        &mut self,
        NotificationId(id): NotificationId,
        reason: &CloseReason,
    ) -> zbus::Result<()> {
        let context = self.interface_ref.signal_context();
        NotifyServerInterface::notification_closed(context, id, *reason as u32).await
    }
}

struct NotifyServerInterface {
    event_system: Arc<Mutex<SingleEventSystem<Event>>>,
    last_id: u32,
}

impl NotifyServerInterface {
    fn add_observer(
        &mut self,
        observer: Arc<Mutex<dyn observer::Observer<Event> + Send + Sync + 'static>>,
    ) {
        let mut event_system = self.event_system.lock().unwrap();
        event_system.set_observer(observer);
    }

    // Calling this method in tests could break the notification service of the current operating system.
    #[cfg(not(test))]
    pub fn run(self) -> zbus::Result<Connection> {
        ConnectionBuilder::session()?
            .name("org.freedesktop.Notifications")?
            .serve_at("/org/freedesktop/Notifications", self)?
            .build()
    }
}

#[allow(clippy::too_many_arguments)]
#[dbus_interface(name = "org.freedesktop.Notifications")]
impl NotifyServerInterface {
    fn get_capabilities(&self) -> Vec<&str> {
        vec![
            "action-icons",
            "actions",
            "body",
            "body-hyperlinks",
            "body-markup",
            "persistence",
        ]
    }

    fn notify(
        &mut self,
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints: HashMap<String, Value>,
        expire_timeout: i32,
    ) -> u32 {
        let mut builder = NotificationBuilder::default()
            .with_app_name(app_name)
            .with_app_icon(app_icon)
            .with_summary(summary)
            .with_body(body)
            .with_expire_timeout(expire_timeout);
        let id = match replaces_id {
            0 => {
                self.last_id += 1;
                self.last_id
            }
            id => id,
        };
        builder.set_id(id.into());
        hints.into_iter().for_each(|(key, hint)| {
            if &key[..] == "urgency" {
                builder.set_urgency(hint.into())
            }
        });

        let mut actions_vec = Vec::with_capacity(actions.len() / 2);
        // TODO change to group_by once https://github.com/rust-lang/rust/issues/80552 is stable
        let mut actions_iter = actions.into_iter();
        while let Some(key) = actions_iter.next() {
            let Some(text) = actions_iter.next() else {
                continue;
            };
            actions_vec.push(crate::notification::Action { key, text });
        }
        builder.set_actions(actions_vec);

        let notification = builder.build();

        self.event_system
            .lock()
            .unwrap()
            .notify(&Event::Notify(notification));
        id
    }

    fn close_notification(&self, id: u32) {
        self.event_system
            .lock()
            .unwrap()
            .notify(&Event::Close(id.into(), CloseReason::Undefined))
    }

    fn get_server_information(&self) -> (&str, &str, &str, &str) {
        (
            "i3_notify_bar_notification_server",
            "Julian Alberts",
            "0.1",
            "1.2",
        )
    }

    #[dbus_interface(signal)]
    async fn action_invoked(
        signal_ctxt: &SignalContext<'_>,
        id: u32,
        action: &str,
    ) -> zbus::Result<()>;

    #[dbus_interface(signal)]
    async fn notification_closed(
        signal_ctxt: &SignalContext<'_>,
        id: u32,
        reason: u32,
    ) -> zbus::Result<()>;
}

impl Default for NotifyServerInterface {
    fn default() -> Self {
        Self {
            event_system: Arc::new(Mutex::new(SingleEventSystem::default())),
            last_id: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CloseReason {
    Expired = 1,
    Dismissed = 2,
    Closed = 3,
    Undefined = 4,
}

pub enum Message {
    NotificationClosed(u32, CloseReason),
    ActionInvoked(u32, String),
}

#[cfg(test)]
mod interface_tests {

    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use crate::{notification::Notification, Event};

    use super::NotifyServerInterface;

    #[test]
    fn notify() {
        let app_name = String::from("my app name");
        let app_name_cp = app_name.clone();
        let replace_id = 0;
        let app_icon = String::from("my app icon");
        let app_icon_cp = app_icon.clone();
        let summary = String::from("my app name");
        let summary_cp = summary.clone();
        let body = String::from("my app name");
        let body_cp = body.clone();
        let actions = vec![];
        let actions_cp = actions.clone();
        let hints = HashMap::new();
        let hints_cp = hints.clone();
        let expire_timeout = 0;

        let observer = TestObserver { last_event: None };

        let observer = Arc::new(Mutex::new(observer));

        let observer_cp = Arc::clone(&observer);
        let observer_cp: Arc<
            Mutex<dyn observer::Observer<Event> + Sync + std::marker::Send + 'static>,
        > = observer_cp;

        let mut interface = NotifyServerInterface::default();
        interface.add_observer(observer_cp);

        interface.notify(
            app_name_cp,
            replace_id,
            app_icon_cp,
            summary_cp,
            body_cp,
            actions_cp,
            hints_cp,
            expire_timeout,
        );

        assert_eq!(
            observer.lock().unwrap().last_event,
            Some(Event::Notify(Notification {
                actions: vec![],
                app_icon,
                app_name,
                body,
                expire_timeout,
                id: 1.into(),
                summary,
                urgency: crate::notification::Urgency::Normal
            }))
        );

        assert_eq!(interface.last_id, 1);
    }

    struct TestObserver {
        last_event: Option<Event>,
    }

    impl observer::Observer<Event> for TestObserver {
        fn on_notify(&mut self, event: &Event) {
            self.last_event = Some(event.clone())
        }
    }

    unsafe impl Sync for TestObserver {}
    unsafe impl Send for TestObserver {}
}
