use crate::observer::ObserverTrait;

use std::sync::{Arc, Mutex};

type Observer<E> = dyn ObserverTrait<E> + Sync + Send + 'static;

pub struct EventSystem<E> {
    wrapped_observers: Vec<Arc<Mutex<Observer<E>>>>,
}

impl<E> EventSystem<E> {
    pub fn notify(&self, event: &E) {
        self.wrapped_observers.iter().for_each(|wo| {
            let mut observer = wo.lock().unwrap();
            observer.on_notify(event)
        });
    }

    pub fn add_observer(&mut self, observer: Arc<Mutex<Observer<E>>>) {
        self.wrapped_observers.push(observer);
    }
}

impl<E> Default for EventSystem<E> {
    fn default() -> Self {
        Self {
            wrapped_observers: Vec::default(),
        }
    }
}

pub struct SingleEventSystem<E> {
    wrapped_observer: Option<Arc<Mutex<Observer<E>>>>,
}

impl<E> SingleEventSystem<E> {
    pub fn notify(&self, event: &E) {
        match &self.wrapped_observer {
            Some(wo) => {
                let mut observer = wo.lock().unwrap();
                observer.on_notify(event)
            }
            None => {}
        }
    }

    pub fn set_observer(&mut self, observer: Arc<Mutex<Observer<E>>>) {
        self.wrapped_observer = Some(observer);
    }
}

impl<E> Default for SingleEventSystem<E> {
    fn default() -> Self {
        Self {
            wrapped_observer: None,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    macro_rules! create_observer {
        ($name: ident, $t: ty, $expected: literal) => {
            struct $name {
                pub value: Option<()>,
            }
            impl $name {
                fn new() -> Self {
                    Self { value: None }
                }
            }
            impl ObserverTrait<$t> for $name {
                fn on_notify(&mut self, data: &String) {
                    assert_eq!(data, $expected);
                    self.value = Some(())
                }
            }
            unsafe impl Send for $name {}
            unsafe impl Sync for $name {}
        };
    }

    #[test]
    fn event_system_create() {
        EventSystem::<String>::default();
    }

    #[test]
    fn event_system_notify() {
        let es = EventSystem::default();
        es.notify(&"test");
    }

    #[test]
    fn event_system_single_listener() {
        create_observer!(Observer, String, "test");

        let mut es = EventSystem::default();
        let observer = Arc::new(Mutex::new(Observer::new()));
        let observer_cp = Arc::clone(&observer);
        es.add_observer(observer_cp);
        es.notify(&String::from("test"));
        assert!(observer.lock().unwrap().value.is_some());
    }

    #[test]
    fn event_system_multiple_listener() {
        create_observer!(Observer, String, "test");

        let mut es = EventSystem::default();
        let observer = Arc::new(Mutex::new(Observer::new()));
        let observer_cp = Arc::clone(&observer);
        es.add_observer(observer_cp);
        let observer2 = Arc::new(Mutex::new(Observer::new()));
        let observer_cp = Arc::clone(&observer2);
        es.add_observer(observer_cp);
        es.notify(&String::from("test"));
        assert!(observer.lock().unwrap().value.is_some());
        assert!(observer2.lock().unwrap().value.is_some());
    }
}
