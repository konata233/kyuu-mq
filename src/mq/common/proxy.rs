use std::cell::RefCell;
use std::sync::{Arc, Mutex};

pub struct ProxyHolder<T> {
    obj: Option<T>,
    obj_ref: Option<Arc<Mutex<RefCell<T>>>>
}

impl <T> ProxyHolder<T> {
    pub fn new(obj: T) -> Self {
        ProxyHolder {
            obj: Some(obj),
            obj_ref: None
        }
    }

    pub fn init(mut self) -> ProxyHolder<T> {
        self.obj_ref = Some(Arc::from(Mutex::from(RefCell::from(self.obj.take().unwrap()))));
        self.obj = None;
        self
    }

    pub fn get(&mut self) -> Arc<Mutex<RefCell<T>>> {
        self.obj_ref.as_mut()
            .unwrap()
            .clone()
    }
}