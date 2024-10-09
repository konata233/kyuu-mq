use crate::mq::queue::queue_object::QueueObject;
use std::sync::{Arc, Mutex};

pub struct Queue {
    name: String,
    data: Arc<Mutex<Vec<QueueObject>>>,
    len: u64
}

impl Queue {
    pub fn new(name: &String) -> Self {
        Queue {
            name: String::from(name),
            data: Arc::new(Mutex::new(Vec::new())),
            len: 0
        }
    }

    pub fn push_back(&mut self, value: QueueObject) {
        let mut data = self.data.lock().unwrap();
        data.push(value);
        self.len += 1;
    }

    pub fn pop_front(&mut self) -> Option<QueueObject> {
        if self.is_empty() {
            return None;
        }
        let mut data = self.data.lock().unwrap();
        self.len -= 1;
        Some(data.remove(0))
    }

    pub fn peek(&self) -> Option<QueueObject> {
        if self.is_empty() {
            return None;
        }
        let mut data = self.data.lock().unwrap();
        Some(data[0].clone())
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.data.lock().unwrap().clear();
    }
}