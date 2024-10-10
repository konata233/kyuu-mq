use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::mq::breaker::core::{Breaker, Core};
use crate::mq::host::vhost::VirtualHost;
use crate::mq::protocol::raw::RawData;
use crate::mq::queue::queue_object::QueueObject;

pub struct HostManager {
    breaker: Option<Arc<Mutex<Breaker>>>,
    virtual_hosts: HashMap<String, Arc<Mutex<RefCell<VirtualHost>>>>
}

impl HostManager {
    pub fn new() -> HostManager {
        HostManager {
            breaker: None,
            virtual_hosts: HashMap::new()
        }
    }

    pub fn init(mut self, breaker: Arc<Mutex<Breaker>>) -> HostManager {
        self.breaker = Some(breaker);
        self
    }

    pub fn add(&mut self, name: String, vhost: VirtualHost) {
        self.virtual_hosts.insert(name, Arc::new(Mutex::new(RefCell::from(vhost))));
    }

    pub fn get(&mut self, name: String) -> Option<&mut Arc<Mutex<RefCell<VirtualHost>>>> {
        self.virtual_hosts.get_mut(&name)
    }

    pub fn remove(&mut self, name: String) -> Option<Arc<Mutex<RefCell<VirtualHost>>>> {
        self.virtual_hosts.remove(&name)
    }

    pub fn get_all(&self) -> Vec<Arc<Mutex<RefCell<VirtualHost>>>> {
        self.virtual_hosts.values().cloned().collect()
    }

    pub fn send_raw_to_host(&self, raw: RawData) -> Option<QueueObject> {
        //println!("!!");
        let host_name = raw.virtual_host.trim().to_string();
        let vhost = self.virtual_hosts.get(&host_name).cloned();
        if let Some(vhost) = vhost {
            vhost.lock()
                .unwrap()
                .borrow_mut().process_incoming(raw)
        } else {
            None
        }
    }
}