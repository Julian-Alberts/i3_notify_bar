use crate::{notification::Notification, CloseReason};

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Notify(Notification),
    Close(u32, CloseReason),
}
