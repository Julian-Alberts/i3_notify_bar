use zbus::{ObjectServer, fdo};
use observer::SingleEventSystem;
use std::convert::TryInto;
use std::sync::{Arc, Mutex};

use crate::{Event, routes};

pub struct NotifyServer {
    event_system: Arc<Mutex<SingleEventSystem<Event>>>,
    object_server: ObjectServer<'static>
}

impl NotifyServer {

    pub fn start() -> Self {

        let event_system = Arc::new(Mutex::new(SingleEventSystem::new()));
        let event_system_cp = Arc::clone(&event_system);

        let connection = Arc::new(zbus::Connection::new_session().unwrap());
        fdo::DBusProxy::new(&connection).unwrap().request_name(
            routes::DBUS_INTERFACE_NAME,
            fdo::RequestNameFlags::ReplaceExisting.into(),
        ).unwrap();

        let connection_cp = Arc::clone(&connection);

        std::thread::spawn(move || {
            let mut object_server = zbus::ObjectServer::new(&connection);
            object_server.at(&routes::DBUS_INTERFACE_PATH.try_into().unwrap(), routes::Routes::new(event_system_cp)).unwrap();
            loop {
                match object_server.try_handle_next() {
                    Ok(Some(_)) => {
                    },
                    Err(err) => eprintln!("{}", err),
                    _ => {}
                }
            }
        });

        let object_server = zbus::ObjectServer::new(&connection_cp);
    
        Self {
            event_system,
            object_server
        }
    }

    pub fn add_observer(&mut self, observer: Arc<Mutex<dyn observer::SingleObserver<Event> + Send + Sync + 'static>>) {
        let mut event_system = self.event_system.lock().unwrap();
        event_system.set_observer(observer);
    }

    pub fn send_action_invoked(&self, id: u32, action: &str) {
        self.object_server.with(&routes::DBUS_INTERFACE_PATH.try_into().unwrap(), |interface: &routes::Routes| {
            interface.action_invoked(id, action)
        }).unwrap();
    }

    pub fn notification_closed(&self, id: u32, reason: CloseReason) {
        self.object_server.with(&routes::DBUS_INTERFACE_PATH.try_into().unwrap(), |interface: &routes::Routes| {
            interface.notification_closed(id, reason as u32)
        }).unwrap();
    }

}

#[derive(Clone, Copy)]
pub enum CloseReason {
    Expired = 1,
    Dismissed = 2,
    Closed = 3,
    Undeined = 4
}