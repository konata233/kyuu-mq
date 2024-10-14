use std::cell::RefCell;
use crate::mq::net::conn::PhysicalConnection;
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex, Condvar, RwLock};
use crate::mq::common::proxy::ProxyHolder;
use crate::mq::net::manager::{ChannelManager, PhysicalConnectionManager};

pub struct PhysicalConnectionFactory {
    local: Option<SocketAddr>,
    remote: Option<SocketAddr>,
    stream: Option<TcpStream>,
    manager_proxy: Option<Arc<RwLock<PhysicalConnectionManager>>>
}

impl PhysicalConnectionFactory {
    pub fn new() -> Self {
        PhysicalConnectionFactory {
            local: None,
            remote: None,
            stream: None,
            manager_proxy: None
        }
    }

    pub fn set_manager_proxy(mut self, manager_proxy: Option<Arc<RwLock<PhysicalConnectionManager>>>) -> Self {
        self.manager_proxy = manager_proxy;
        self
    }

    pub fn set_local(mut self, local: SocketAddr) -> Self {
        self.local = Some(local);
        self
    }

    pub fn set_remote(mut self, remote: SocketAddr) -> Self {
        self.remote = Some(remote);
        self
    }

    pub fn set_stream(mut self, stream: TcpStream) -> Self {
        self.stream = Some(stream);
        self
    }

    pub fn fetch(mut self) -> Result<PhysicalConnection, ()> {
        if let Some(rem) = self.remote {
            let conn = TcpStream::connect(rem).unwrap();
            self.local = Some(conn.local_addr().unwrap().clone());
            self.remote = Some(conn.peer_addr().unwrap().clone());
            Ok(PhysicalConnection {
                local_addr: self.local.unwrap(),
                remote_addr: self.remote.unwrap(),
                stream: RefCell::from(conn),
                closed: RefCell::from(false),

                manager_proxy: self.manager_proxy.unwrap(),
                channel_manager: RefCell::from(ChannelManager::new())
            })
        }
        else if let Some(conn) = self.stream {
            self.local = Some(conn.local_addr().unwrap().clone());
            self.remote = Some(conn.peer_addr().unwrap().clone());
            Ok(PhysicalConnection {
                local_addr: self.local.unwrap(),
                remote_addr: self.remote.unwrap(),
                stream: RefCell::from(conn),
                closed: RefCell::from(false),

                manager_proxy: self.manager_proxy.unwrap(),
                channel_manager: RefCell::from(ChannelManager::new())
            })
        }
        else {
           Err(())
        }
    }
}