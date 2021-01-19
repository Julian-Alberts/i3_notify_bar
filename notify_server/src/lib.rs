mod routes;
pub mod notification;
mod events;
mod notify_server;

pub use events::Event;
pub use notify_server::NotifyServer;
pub use observer::Observer;
pub struct Options {
    notify: &'static dyn Fn(&mut notification::Notification)
}

impl Options {
    pub fn new() -> Self {
        Self {
            notify: &|_| {}
        }
    }

    pub fn set_notify(&mut self, notify: &'static  dyn Fn(&mut notification::Notification)) {
        self.notify = notify;
    }
}