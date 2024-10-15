use crate::mq::queue::manager::QueueManager;
use crate::mq::queue::qbase::Queue;
use crate::mq::routing::key::RoutingKey;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct Exchange {
    name: String,
    lower_exchange: HashMap<String, Arc<RwLock<Exchange>>>,
    queue_manager: Arc<RwLock<QueueManager>>,
    self_ref: Option<Arc<RwLock<Exchange>>>,
}

impl Exchange {
    pub fn new(name: String) -> Exchange {
        Exchange {
            name,
            lower_exchange: HashMap::new(),
            queue_manager: Arc::from(RwLock::from(QueueManager::new())),
            self_ref: None,
        }
    }

    pub fn add_exchange(&mut self, name: String) -> &mut Self {
        if !self.lower_exchange.contains_key(&name) {
            let exc_ref = Arc::from(RwLock::from(Exchange::new(name.clone())));
            exc_ref.write().unwrap().init(exc_ref.clone());
            self.lower_exchange.insert(name, exc_ref.clone());
        } else {
            println!("[mq] exchange already exists: {}", name)
        }
        self
    }

    pub fn init(&mut self, self_ref: Arc<RwLock<Exchange>>) -> &mut Self {
        self.self_ref = Some(self_ref);
        self
    }

    pub fn get_exchange(&mut self, name: &String) -> Option<&mut Arc<RwLock<Exchange>>> {
        self.lower_exchange.get_mut(name)
    }

    pub fn remove_exchange(&mut self, name: String) -> Option<Arc<RwLock<Exchange>>> {
        for value in self.lower_exchange.values_mut() {
            value.write().unwrap().remove_all_exchanges();
        }
        self.lower_exchange.remove(&name)
    }

    pub fn remove_all_exchanges(&mut self) -> &mut Self {
        for value in self.lower_exchange.values_mut() {
            value.write().unwrap().remove_all_exchanges();
        }
        self.lower_exchange.clear();
        self
    }

    pub fn add_queue(&mut self, name: &String) -> &mut Self {
        self.queue_manager.write().unwrap().add(name);
        self
    }

    pub fn get_queue(&self, name: &String) -> Option<Arc<RwLock<Queue>>> {
        self.queue_manager.read().unwrap().get(name)
    }

    pub fn remove_queue(&mut self, name: String) -> Option<Arc<RwLock<Queue>>> {
        self.queue_manager.write().unwrap().remove(name)
    }

    pub fn get_all_queues(&self) -> Vec<Arc<RwLock<Queue>>> {
        self.queue_manager.read().unwrap().get_all()
    }

    pub fn clear_queue(&mut self, name: String) -> &mut Self {
        self.queue_manager
            .write()
            .unwrap()
            .get(&name)
            .unwrap()
            .write()
            .unwrap()
            .clear();
        self
    }

    pub fn walk(&mut self, routing: RoutingKey, next: usize) -> Option<Vec<Arc<RwLock<Exchange>>>> {
        if next > 2 {
            return None;
        }

        let r: [String; 4];
        let routing_cloned = routing.clone();
        match routing {
            RoutingKey::Direct(key) => r = key,
            RoutingKey::Topic(key) => r = key,
            RoutingKey::Fanout(key) => r = key,
        }
        let next_key  = r[next].clone();
        if (&next_key).starts_with("\0") || next_key.is_empty() || next_key.starts_with("!") || next == 2 {
            Some(vec![self.self_ref.clone()?])
        } else if next_key.starts_with("*") {
            let mut vec = vec![];
            for value in self.lower_exchange.values().cloned() {
                vec.push(value);
            }
            return Some(vec);
        } else {
            self.lower_exchange.get_mut(&next_key)?.write().unwrap().walk(routing_cloned, next + 1)
        }
    }

    pub fn walk_readonly(&self, routing: RoutingKey, next: usize) -> Option<Vec<Arc<RwLock<Exchange>>>> {
        if next > 2 {
            return None;
        }

        let r: [String; 4];
        let routing_cloned = routing.clone();
        match routing {
            RoutingKey::Direct(key) => r = key,
            RoutingKey::Topic(key) => r = key,
            RoutingKey::Fanout(key) => r = key,
        }
        let next_key  = r[next].clone();
        if (&next_key).starts_with("\0") || next_key.is_empty() || next_key.starts_with("!") || next == 2 {
            Some(vec![self.self_ref.clone()?])
        } else if next_key.starts_with("*") {
            let mut vec = vec![];
            for value in self.lower_exchange.values().cloned() {
                vec.push(value);
            }
            return Some(vec);
        } else {
            self.lower_exchange.get(&next_key)?.read().unwrap().walk_readonly(routing_cloned, next + 1)
        }
    }
}