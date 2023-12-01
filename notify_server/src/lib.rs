mod events;
pub mod notification;
mod notify_server;

use std::fmt::Display;

pub use crate::notify_server::CloseReason;
pub use crate::notify_server::{Message as NotificationMessage, NotifyServer};
pub use events::Event;
pub use observer::Observer;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NotificationId(u32);

pub struct Options {
    notify: &'static dyn Fn(&mut notification::Notification),
}

impl From<u32> for NotificationId {
    fn from(id: u32) -> NotificationId {
        NotificationId(id)
    }
}

impl Display for NotificationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Options {
    pub fn set_notify(&mut self, notify: &'static dyn Fn(&mut notification::Notification)) {
        self.notify = notify;
    }
}

impl Default for Options {
    fn default() -> Self {
        Self { notify: &|_| {} }
    }
}
