use observer::SingleEventSystem;
use zbus::blocking::InterfaceRef;
use zbus::zvariant::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use zbus::{dbus_interface, blocking::{ConnectionBuilder, Connection}, SignalContext};

use crate::notification::Notification;
use crate::Event;

pub struct NotifyServer {
    interface_ref: InterfaceRef<NotifyServerInterface>,
}

impl NotifyServer {

    pub fn start() -> zbus::Result<Self> {
        let interface = NotifyServerInterface::default();
        let connection = interface.run()?;
        let i = connection.object_server().interface::<_,NotifyServerInterface>("/org/freedesktop/Notifications")?;
        Ok(Self {
            interface_ref: i,
        })
    }

    pub fn add_observer(
        &mut self,
        observer: Arc<Mutex<dyn observer::Observer<Event> + Send + Sync + 'static>>,
    ) {
        self.interface_ref.get_mut().add_observer(observer)
    }

    pub async fn action_invoked(&self, id: u32, action: &str) -> zbus::Result<()>{
        let context = self.interface_ref.signal_context();
       NotifyServerInterface::action_invoked(context, id, action).await
    }

    pub async fn notification_closed(&mut self, id: u32, reason: &CloseReason) -> zbus::Result<()> {
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

    pub fn run(self) -> zbus::Result<Connection> {
        ConnectionBuilder::session()?
            .name("org.freedesktop.Notifications")?
            .serve_at("/org/freedesktop/Notifications", self)?
            .build()
    }

    fn create_new_notification(
        &mut self,
        app_name: String,
        replace_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints: HashMap<String, Value>,
        expire_timeout: i32,
    ) -> Notification {
        match replace_id {
            0 => {
                self.last_id += 1;
                Notification::new(
                    app_name,
                    self.last_id,
                    app_icon,
                    summary,
                    body,
                    actions,
                    hints,
                    expire_timeout,
                )
            }
            id => Notification::new(
                app_name,
                id,
                app_icon,
                summary,
                body,
                actions,
                hints,
                expire_timeout,
            ),
        }
    }
}

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
        let notification = self.create_new_notification(
            app_name,
            replaces_id,
            app_icon,
            summary,
            body,
            actions,
            hints,
            expire_timeout,
        );
        let id = notification.id;

        self.event_system
            .lock()
            .unwrap()
            .notify(&Event::Notify(notification));
        id
    }

    fn close_notification(&self, id: u32) {
        self.event_system.lock().unwrap().notify(&Event::Close(id, CloseReason::Undeined))
    }

    fn get_server_information(&self) -> (&str, &str, &str, &str) {
        ("test", "Julian Alberts", "0.1", "1.2")
    }

    #[dbus_interface(signal)]
    async fn action_invoked(signal_ctxt: &SignalContext<'_>, id: u32, action: &str) -> zbus::Result<()>;

    #[dbus_interface(signal)]
    async fn notification_closed(signal_ctxt: &SignalContext<'_>, id: u32, reason: u32) -> zbus::Result<()>;
}

impl Default for NotifyServerInterface {

    fn default() -> Self {
        Self {
            event_system: Arc::new(Mutex::new(SingleEventSystem::new())),
            last_id: 0
        }
    }

}

#[derive(Clone, Copy)]
pub enum CloseReason {
    Expired = 1,
    Dismissed = 2,
    Closed = 3,
    Undeined = 4,
}

pub enum Message {
    NotificationClosed(u32, CloseReason),
    ActionInvoked(u32, String),
}
