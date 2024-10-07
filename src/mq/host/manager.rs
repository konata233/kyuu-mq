use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::mq::breaker::core::{Breaker, Core};
use crate::mq::host::vhost::VirtualHost;

pub struct HostManager {
    breaker: Option<Arc<Mutex<Breaker>>>,
    virtual_hosts: HashMap<String, VirtualHost>
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
        self.virtual_hosts.insert(name, vhost);
    }

    pub fn get(&mut self, name: String) -> Option<&mut VirtualHost> {
        self.virtual_hosts.get_mut(&name)
    }

    pub fn remove(&mut self, name: String) -> Option<VirtualHost> {
        self.virtual_hosts.remove(&name)
    }

    pub fn get_all(&self) -> Vec<&VirtualHost> {
        self.virtual_hosts.values().collect()
    }
}