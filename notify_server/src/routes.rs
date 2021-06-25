use std::{collections::HashMap, sync::{Arc, Mutex}};
use observer::SingleEventSystem;
use zbus::dbus_interface;
use zvariant::Value;

use crate::{Event, notification::Notification};

pub const DBUS_INTERFACE_NAME: &str = "org.freedesktop.Notifications";
pub const DBUS_INTERFACE_PATH: &str = "/org/freedesktop/Notifications";

pub struct Routes {
    event_system: Arc<Mutex<SingleEventSystem<Event>>>,
    last_id: u32
}

impl Routes {
    pub fn new(event_system: Arc<Mutex<SingleEventSystem<Event>>>) -> Self {
        Self {
            event_system,
            last_id: 0
        }
    }

    fn create_new_notification(&mut self, app_name: String, replace_id: u32, app_icon: String, summary: String, body: String, actions: Vec<String>, hints: HashMap<String, Value>, expire_timeout: i32) -> Notification {
        match replace_id {
            0 => {
                self.last_id += 1;
                Notification::new(app_name, self.last_id, app_icon, summary, body, actions, hints, expire_timeout)
            },
            id => {
                Notification::new(app_name, id, app_icon, summary, body, actions, hints, expire_timeout)
            }
        }
    }

}

#[dbus_interface(name = "org.freedesktop.Notifications")]
impl Routes {

    fn get_capabilities(&self) -> Vec<&str> {
        vec![
            "action-icons",
            "actions",
            "body",
            "body-hyperlinks",
            "body-markup",
            "persistence"
        ]
    }
    
    fn notify(&mut self, app_name: String, replaces_id: u32, app_icon: String, summary: String, body: String, actions: Vec<String>, hints: HashMap<String, Value>, expire_timeout: i32) -> u32 {
        
        let notification = self.create_new_notification(app_name, replaces_id, app_icon, summary, body, actions, hints, expire_timeout);
        let id = notification.id;

        self.event_system.lock().unwrap().notify(&Event::Notify(notification));
        id
    }

    fn close_notification(&self, _id: u32) {
        println!("close_notification")
    }

    fn get_server_information(&self) -> (&str, &str, &str, &str) {
        (
            "test",
            "Julian Alberts",
            "0.1",
            "1.2"
        )
    }

    #[dbus_interface(signal)]
    pub fn action_invoked(&self, id: u32, action: &str) -> zbus::Result<()>;

    #[dbus_interface(signal)]
    pub fn notification_closed(&self, id: u32, reason: u32) -> zbus::Result<()>;

}