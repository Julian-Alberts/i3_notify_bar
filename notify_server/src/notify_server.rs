use zbus::fdo;
use observer::EventSystem;
use std::convert::TryInto;
use std::sync::{Arc, Mutex};

use crate::{Event, routes};

pub struct NotifyServer {
    event_system: Arc<Mutex<EventSystem<Event>>>
}

impl NotifyServer {

    pub fn start() -> Self {

        let event_system = Arc::new(Mutex::new(EventSystem::new()));
        let event_system_cp = Arc::clone(&event_system);

        std::thread::spawn(move || {
            let connection = zbus::Connection::new_session().unwrap();
            fdo::DBusProxy::new(&connection).unwrap().request_name(
                routes::DBUS_INTERFACE_NAME,
                fdo::RequestNameFlags::ReplaceExisting.into(),
            ).unwrap();
    
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
    
        Self {
            event_system
        }
    }

    pub fn add_observer(&mut self, observer: Arc<Mutex<dyn observer::Observer<Event> + Send + Sync + 'static>>) {
        let mut event_system = self.event_system.lock().unwrap();
        event_system.add_observer(observer);
    }

}