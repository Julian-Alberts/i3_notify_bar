mod event_system;
mod observer;

pub use crate::observer::ObserverTrait as Observer;
pub use event_system::{EventSystem, SingleEventSystem};
