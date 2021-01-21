pub trait ObserverTrait<E> {
    fn on_notify(&mut self, event: &E);
}

pub trait SingleObserver<E> {
    fn on_notify(&mut self, event: E);
}