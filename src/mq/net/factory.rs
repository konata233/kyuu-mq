use crate::mq::net::conn::PhysicalConnection;
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use crate::mq::net::manager::ChannelManager;

pub struct PhysicalConnectionFactory {
    local: Option<SocketAddr>,
    remote: Option<SocketAddr>,
    stream: Option<TcpStream>
}

impl PhysicalConnectionFactory {
    pub fn new() -> Self {
        PhysicalConnectionFactory {
            local: None,
            remote: None,
            stream: None
        }
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
        self.local = Some(stream.local_addr().unwrap().clone());
        self.remote = Some(stream.peer_addr().unwrap().clone());
        self.stream = Some(stream);
        self
    }

    pub fn fetch(mut self) -> Result<PhysicalConnection, ()> {
        let flag = Arc::new((Mutex::new(false), Condvar::new()));
        let flag2 = flag.clone();
        if let Some(rem) = self.remote {
            let conn = TcpStream::connect(rem).unwrap();
            self.local = Some(conn.local_addr().unwrap().clone());
            self.remote = Some(conn.peer_addr().unwrap().clone());
            Ok(PhysicalConnection {
                local_addr: self.local.unwrap(),
                remote_addr: self.remote.unwrap(),
                stream: conn,
                handle: Some(thread::spawn(move || {
                    let (lock, condvar) = &*flag;
                    let mut flag = lock.lock().unwrap();
                    while !*flag {
                        flag = condvar.wait(flag).unwrap();
                    }
                })),
                launch_flag: flag2,
                channel_manager: ChannelManager::new()
            })
        }
        else if let Some(conn) = self.stream {
            self.local = Some(conn.local_addr().unwrap().clone());
            self.remote = Some(conn.peer_addr().unwrap().clone());
            Ok(PhysicalConnection {
                local_addr: self.local.unwrap(),
                remote_addr: self.remote.unwrap(),
                stream: conn,
                handle: Some(thread::spawn(move || {
                    let (lock, condvar) = &*flag;
                    let mut flag = lock.lock().unwrap();
                    while !*flag {
                        flag = condvar.wait(flag).unwrap();
                    }
                })),
                launch_flag: flag2,
                channel_manager: ChannelManager::new()
            })
        }
        else {
           Err(())
        }
    }
}