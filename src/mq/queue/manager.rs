use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use crate::mq::queue::qbase::Queue;
use crate::mq::queue::queue_object::QueueObject;

pub struct QueueManager {
    queues: HashMap<String, Arc<RwLock<Queue>>>
}

impl QueueManager {
    pub fn new() -> QueueManager {
        QueueManager {
            queues: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: &String) {
        let q = Queue::new(name);
        let q_ref = Arc::new(RwLock::new(q));
        self.queues.insert(name.clone(), q_ref.clone());
    }

    pub fn get(&self, name: &String) -> Option<Arc<RwLock<Queue>>> {
        self.queues.get(name).cloned()
    }

    pub fn remove(&mut self, name: String) -> Option<Arc<RwLock<Queue>>> {
        self.queues.remove(&name)
    }

    pub fn get_all(&self) -> Vec<Arc<RwLock<Queue>>> {
        self.queues.values().cloned().collect()
    }
}