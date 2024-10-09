use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::mq::queue::qbase::Queue;
use crate::mq::queue::queue_object::QueueObject;

pub struct QueueManager {
    queues: HashMap<String, Arc<Mutex<Queue>>>
}

impl QueueManager {
    pub fn new() -> QueueManager {
        QueueManager {
            queues: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: &String) {
        let q = Queue::new(name);
        let q_ref = Arc::new(Mutex::new(q));
        self.queues.insert(name.clone(), q_ref.clone());
    }

    pub fn get(&mut self, name: &String) -> Option<Arc<Mutex<Queue>>> {
        self.queues.get(name).cloned()
    }

    pub fn remove(&mut self, name: String) -> Option<Arc<Mutex<Queue>>> {
        self.queues.remove(&name)
    }

    pub fn get_all(&self) -> Vec<Arc<Mutex<Queue>>> {
        self.queues.values().cloned().collect()
    }
}