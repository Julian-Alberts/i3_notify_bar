mod events;
pub mod notification;
mod notify_server;

pub use crate::notify_server::CloseReason;
pub use crate::notify_server::{Message as NotificationMessage, NotifyServer};
pub use events::Event;
pub use observer::Observer;

pub struct Options {
    notify: &'static dyn Fn(&mut notification::Notification),
}

impl Options {
    pub fn new() -> Self {
        Self { notify: &|_| {} }
    }

    pub fn set_notify(&mut self, notify: &'static dyn Fn(&mut notification::Notification)) {
        self.notify = notify;
    }
}
