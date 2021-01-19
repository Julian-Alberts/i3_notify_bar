use crate::observer::ObserverTrait;

use std::sync::{Arc, Mutex};

type Observer<E> = dyn ObserverTrait<E> + Sync + Send + 'static;

pub struct EventSystem<E> {
    wrapped_observers: Vec<Arc<Mutex<Observer<E>>>>
}

impl <E> EventSystem<E> {
    pub fn new() -> Self {
        Self {
            wrapped_observers: Vec::new()
        }
    }

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