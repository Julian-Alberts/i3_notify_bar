use observer::SingleEventSystem;
use std::convert::TryInto;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use zbus::{fdo, ObjectServer};

use crate::{routes, Event};

pub struct NotifyServer {
    event_system: Arc<Mutex<SingleEventSystem<Event>>>,
    object_server: ObjectServer<'static>,
    message_tx: Arc<Sender<Message>>
}

impl NotifyServer {
    pub fn start() -> Self {
        let event_system = Arc::new(Mutex::new(SingleEventSystem::new()));

        let connection = Arc::new(zbus::Connection::new_session().unwrap());
        fdo::DBusProxy::new(&connection)
            .unwrap()
            .request_name(
                routes::DBUS_INTERFACE_NAME,
                fdo::RequestNameFlags::ReplaceExisting.into(),
            )
            .unwrap();

        let connection_cp = Arc::clone(&connection);
        let event_system_cp = Arc::clone(&event_system);

        std::thread::spawn(move || {
            let mut object_server = zbus::ObjectServer::new(&connection_cp);
            object_server
                .at(
                    &routes::DBUS_INTERFACE_PATH.try_into().unwrap(),
                    routes::Routes::new(event_system_cp),
                )
                .unwrap();
            loop {
                match object_server.try_handle_next() {
                    Ok(Some(_)) => {}
                    Err(err) => eprintln!("{}", err),
                    _ => {}
                }
            }
        });

        let connection_cp = Arc::clone(&connection);
        let event_system_cp = Arc::clone(&event_system);
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let mut object_server = zbus::ObjectServer::new(&connection_cp);
            object_server
                .at(
                    &routes::DBUS_INTERFACE_PATH.try_into().unwrap(),
                    routes::Routes::new(event_system_cp),
                )
                .unwrap();
            loop {
                match rx.recv() {
                    Ok(Message::ActionInvoked(id, key)) => {
                        send_action_invoked(&object_server, id, key.as_str())
                    },
                    Ok(Message::NotificationClosed(id, reason)) => {
                        notification_closed(&object_server, id, reason)
                    },
                    Err(_) => {}
                }
            }
        });

        let object_server = zbus::ObjectServer::new(&connection);
        

        Self {
            event_system,
            object_server,
            message_tx: Arc::new(tx)
        }
    }

    pub fn add_observer(
        &mut self,
        observer: Arc<Mutex<dyn observer::Observer<Event> + Send + Sync + 'static>>,
    ) {
        let mut event_system = self.event_system.lock().unwrap();
        event_system.set_observer(observer);
    }

    pub fn send_action_invoked(&self, id: u32, action: &str) {
        send_action_invoked(&self.object_server, id, action)
    }

    pub fn notification_closed(&self, id: u32, reason: CloseReason) {
        notification_closed(&self.object_server, id, reason);
    }

    pub fn get_message_channel(&self) -> Arc<Sender<Message>> {
        Arc::clone(&self.message_tx)
    }
}

fn send_action_invoked(object_server: &ObjectServer, id: u32, action: &str) {
    object_server
            .with(
                &routes::DBUS_INTERFACE_PATH.try_into().unwrap(),
                |interface: &routes::Routes| interface.action_invoked(id, action),
            )
            .unwrap();
}

fn notification_closed(object_server: &ObjectServer, id: u32, reason: CloseReason) {
    object_server
            .with(
                &routes::DBUS_INTERFACE_PATH.try_into().unwrap(),
                |interface: &routes::Routes| interface.notification_closed(id, reason as u32),
            )
            .unwrap();
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
    ActionInvoked(u32, String)
}