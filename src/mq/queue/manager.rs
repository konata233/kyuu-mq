use std::collections::HashMap;
use crate::mq::queue::qbase::Queue;

pub struct QueueManager<T> {
    queues: HashMap<String, Queue<T>>
}

impl<T> QueueManager<T> where T: Clone {
    pub fn new() -> QueueManager<T> {
        QueueManager {
            queues: HashMap::new()
        }
    }

    pub fn add(&mut self, name: &String) {
        let q = Queue::new(name);
        self.queues.insert(name.clone(), q);
    }

    pub fn get(&mut self, name: String) -> Option<&mut Queue<T>> {
        self.queues.get_mut(&name)
    }

    pub fn remove(&mut self, name: String) -> Option<Queue<T>> {
        self.queues.remove(&name)
    }

    pub fn get_all(&self) -> Vec<&Queue<T>> {
        self.queues.values().collect()
    }
}