use crate::{notification::Notification, CloseReason, NotificationId};

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Notify(Notification),
    Close(NotificationId, CloseReason),
}
