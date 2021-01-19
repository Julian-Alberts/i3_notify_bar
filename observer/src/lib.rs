mod event_system;
mod observer;

pub use event_system::{EventSystem};
pub use observer::ObserverTrait as Observer;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
