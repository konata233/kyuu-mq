use crate::mq::breaker::core::Breaker;
use crate::mq::host::vhost::VirtualHost;
use crate::mq::protocol::raw::RawData;
use crate::mq::queue::queue_object::QueueObject;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

pub struct HostManager {
    breaker: Option<Arc<Mutex<Breaker>>>,
    virtual_hosts: HashMap<String, Arc<RwLock<VirtualHost>>>
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
        self.virtual_hosts.insert(name, Arc::new(RwLock::new(vhost)));
    }

    pub fn get(&mut self, name: String) -> Option<&mut Arc<RwLock<VirtualHost>>> {
        self.virtual_hosts.get_mut(&name)
    }

    pub fn remove(&mut self, name: String) -> Option<Arc<RwLock<VirtualHost>>> {
        self.virtual_hosts.remove(&name)
    }

    pub fn get_all(&self) -> Vec<Arc<RwLock<VirtualHost>>> {
        self.virtual_hosts.values().cloned().collect()
    }

    pub fn send_raw_to_host(&self, raw: RawData) -> Option<QueueObject> {
        //println!("!!");
        let host_name = raw.virtual_host.trim().to_string();
        let vhost = self.virtual_hosts.get(&host_name).cloned();
        let io_type = &raw.io_type;

        if let Some(vhost) = vhost {
            vhost
                .read()
                .unwrap()
                .process_incoming(raw)

            // todo: I see no difference whether to use read() or write().
            /*match io_type {
                IOType::Read => {
                    vhost.read()
                        .unwrap()
                        .process_incoming(raw)
                }
                IOType::Write => {
                    vhost.write()
                        .unwrap()
                        .process_incoming(raw)
                }
            }*/
        } else {
            None
        }
    }
}