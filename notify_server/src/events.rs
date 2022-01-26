use crate::{notification::Notification, CloseReason};

pub enum Event {
    Notify(Notification),
    Close(u32, CloseReason),
}
