use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::{Arc, Mutex};

pub struct RuntimeContext {
    pub local_host: String,
    pub local_port: u16,
}

impl RuntimeContext {
    pub fn new(local_host: String, local_port: u16) -> RuntimeContext {
        RuntimeContext {
            local_host,
            local_port,
        }
    }
}