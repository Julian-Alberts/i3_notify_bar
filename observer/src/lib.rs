mod event_system;
mod observer;

pub use event_system::{EventSystem, SingleEventSystem};
pub use observer::{ObserverTrait as Observer, SingleObserver};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
