use std::rc::{Rc, Weak};

pub trait Observer<E: Clone> {
    fn on_notify(&self, event: E);
}

pub struct Subject<E: Clone> {
    observers: Vec<Weak<dyn Observer<E>>>,
}
impl<E: Clone> Subject<E> {
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
        }
    }
    pub fn add_observer(&mut self, observer: Rc<dyn Observer<E>>) {
        self.observers.push(Rc::downgrade(&observer));
    }
    pub fn notify_event(&self, event: E) {
        for observer in self.observers.iter() {
            if let Some(observer) = observer.upgrade() {
                observer.on_notify(event.clone());
            }
        }
    }
}
