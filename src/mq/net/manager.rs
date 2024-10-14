use std::cell::RefCell;
use std::collections::HashMap;
use std::net::Shutdown;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex, RwLock};
use crate::mq::breaker::core::Breaker;
use crate::mq::common::proxy::ProxyHolder;
use crate::mq::host::manager::HostManager;
use crate::mq::net::chan::Channel;
use crate::mq::net::conn::PhysicalConnection;
use crate::mq::protocol::raw::RawData;
use crate::mq::queue::queue_object::QueueObject;

pub struct PhysicalConnectionManager {
    breaker: Option<Arc<Mutex<Breaker>>>,
    connections: Vec<Arc<Mutex<PhysicalConnection>>>,
    pub host_manager: Option<Arc<RwLock<HostManager>>>
}

impl PhysicalConnectionManager {
    pub fn new() -> Self {
        PhysicalConnectionManager {
            breaker: None,
            connections: Vec::new(),
            host_manager: None,
        }
    }

    pub fn init(mut self, breaker: Arc<Mutex<Breaker>>, host_manager: Arc<RwLock<HostManager>>) -> Self {
        self.breaker = Some(breaker);
        self.host_manager = Some(host_manager);
        self
    }

    pub fn add(&mut self, conn: PhysicalConnection) -> &mut Self {
        self.connections.push(conn.launch());
        self
    }

    pub fn remove(&mut self, conn: Arc<Mutex<PhysicalConnection>>) -> &mut Self {
        self.connections.remove(
            self.connections.iter().position(|x| Arc::ptr_eq(x, &conn)).unwrap()
        );
        self
    }

    pub fn send_raw_data(&self, raw_data: RawData) -> Option<QueueObject> {
        self.breaker
            .clone()?
            .lock()
            .unwrap()
            .send_raw_to_host(raw_data)
    }

    pub fn close(&self) {
        for conn in &self.connections {
            let c = conn.lock().unwrap();
            if !c.closed.clone().clone().take() {
                c.stream
                    .borrow_mut()
                    .shutdown(Shutdown::Both)
                    .unwrap()
            }
        }
    }
}

pub struct ChannelManager {
    channels: HashMap<String, Channel>,
}

impl ChannelManager {
    pub fn new() -> Self {
        ChannelManager {
            channels: HashMap::new(),
        }
    }

    pub fn add(&mut self, channel: Channel) {
        self.channels.insert(channel.name.clone(), channel);
    }

    pub fn get(&mut self, name: &str) -> Option<&mut Channel> {
        self.channels.get_mut(name)
    }

    pub fn remove(&mut self, name: &str) -> Option<Channel> {
        self.channels.remove(name)
    }

    pub fn list(&self) -> Vec<&Channel> {
        self.channels.values().collect()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.channels.contains_key(name)
    }
}