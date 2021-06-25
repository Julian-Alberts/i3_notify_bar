use crate::notification::Notification;

pub enum Event {
    Notify(Notification),
}
