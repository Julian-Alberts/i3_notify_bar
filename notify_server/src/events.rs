use crate::{NotificationId, notification::Notification, CloseReason};

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Notify(Notification),
    Close(NotificationId, CloseReason),
}
