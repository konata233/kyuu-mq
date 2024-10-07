use std::cell::RefCell;
use std::collections::HashMap;
use std::net::Shutdown;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use crate::mq::breaker::core::Breaker;
use crate::mq::net::chan::Channel;
use crate::mq::net::conn::PhysicalConnection;

pub struct PhysicalConnectionManager {
    breaker: Option<Arc<Mutex<Breaker>>>,
    connections: Vec<Arc<Mutex<PhysicalConnection>>>,
}

impl PhysicalConnectionManager {
    pub fn new() -> Self {
        PhysicalConnectionManager {
            breaker: None,
            connections: Vec::new(),
        }
    }

    pub fn init(mut self, breaker: Arc<Mutex<Breaker>>) -> Self {
        self.breaker = Some(breaker);
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