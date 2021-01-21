use crate::{SingleObserver as SO, observer::ObserverTrait};

use std::sync::{Arc, Mutex};

type Observer<E> = dyn ObserverTrait<E> + Sync + Send + 'static;
type SingleObserver<E> = dyn SO<E> + Sync + Send + 'static;


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

pub struct SingleEventSystem<E> {
    wrapped_observer: Option<Arc<Mutex<SingleObserver<E>>>>
}

impl <E> SingleEventSystem<E> {
    pub fn new() -> Self {
        Self {
            wrapped_observer: None
        }
    }

    pub fn notify(&self, event: E) {
        match &self.wrapped_observer {
            Some(wo) => {
                let mut observer = wo.lock().unwrap();
                observer.on_notify(event)
            },
            None => {}
        }
    }

    pub fn set_observer(&mut self, observer: Arc<Mutex<SingleObserver<E>>>) {
        self.wrapped_observer = Some(observer);
    }

}