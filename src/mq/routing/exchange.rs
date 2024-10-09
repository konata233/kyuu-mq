use std::collections::HashMap;
use crate::mq::queue::manager::QueueManager;
use crate::mq::queue::qbase::Queue;
use crate::mq::routing::key::RoutingKey;

pub struct Exchange<T: Clone> {
    name: String,
    lower_exchange: HashMap<String, Exchange<T>>,
    queue_manager: QueueManager<T>,
}

impl<T: Clone> Exchange<T> {
    pub fn new(name: String) -> Exchange<T> {
        Exchange {
            name,
            lower_exchange: HashMap::new(),
            queue_manager: QueueManager::new(),
        }
    }

    pub fn add_exchange(&mut self, name: String) -> &mut Self {
        self.lower_exchange.insert(name.clone(), Exchange::new(name));
        self
    }

    pub fn get_exchange(&mut self, name: &String) -> Option<&mut Exchange<T>> {
        self.lower_exchange.get_mut(name)
    }

    pub fn remove_exchange(&mut self, name: String) -> Option<Exchange<T>> {
        for value in self.lower_exchange.values_mut() {
            value.remove_all_exchanges();
        }
        self.lower_exchange.remove(&name)
    }

    pub fn remove_all_exchanges(&mut self) -> &mut Self {
        for value in self.lower_exchange.values_mut() {
            value.remove_all_exchanges();
        }
        self.lower_exchange.clear();
        self
    }

    pub fn add_queue(&mut self, name: &String) -> &mut Self {
        self.queue_manager.add(name);
        self
    }

    pub fn get_queue(&mut self, name: String) -> Option<&mut Queue<T>> {
        self.queue_manager.get(name)
    }

    pub fn remove_queue(&mut self, name: String) -> Option<Queue<T>> {
        self.queue_manager.remove(name)
    }

    pub fn get_all_queues(&self) -> Vec<&Queue<T>> {
        self.queue_manager.get_all()
    }

    pub fn clear_queue(&mut self, name: String) -> &mut Self {
        self.queue_manager.get(name).unwrap().clear();
        self
    }

    pub fn walk(&mut self, routing: RoutingKey, next: usize) -> Option<Vec<&mut Exchange<T>>> {
        if next > 3 {
            return None;
        }

        let mut r: [String; 4];
        let routing_cloned = routing.clone();
        match routing {
            RoutingKey::Direct(key) => r = key,
            RoutingKey::Topic(key) => r = key,
            RoutingKey::Fanout(key) => r = key,
        }
        let mut next_key  = r[next].clone();
        if (&next_key).starts_with("\0") || next_key.is_empty() || next_key =="!" || next == 3 {
            Some(vec![self])
        } else if next_key == "*" {
            let mut vec = vec![];
            for value in self.lower_exchange.values_mut() {
                vec.push(value);
            }
            return Some(vec);
        } else {
            self.lower_exchange.get_mut(&next_key)?.walk(routing_cloned, next + 1)
        }
    }
}