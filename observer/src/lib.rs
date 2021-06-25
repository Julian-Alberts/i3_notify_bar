mod event_system;
mod observer;

pub use event_system::{EventSystem, SingleEventSystem};
pub use crate::observer::{ObserverTrait as Observer, SingleObserver};