use std::error::Error;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;
use crate::mq::net::chan::Channel;
use crate::mq::net::manager::ChannelManager;
use crate::mq::protocol::proto::DataHead;
use crate::mq::protocol::protobase::Deserialize;

pub struct PhysicalConnection {
    pub local_addr: SocketAddr,
    pub remote_addr: SocketAddr,

    pub stream: TcpStream,

    pub handle: Option<JoinHandle<()>>,
    pub launch_flag: Arc<(Mutex<bool>, Condvar)>,

    pub channel_manager: ChannelManager
}

impl PhysicalConnection {

    pub fn launch(mut self) -> PhysicalConnection {
        let (lock, condvar) = &*self.launch_flag;
        let mut flag = lock.lock().unwrap();
        *flag = true;
        condvar.notify_one();

        if let Some(handle) = self.handle {
            handle.join().unwrap();
        }
        drop(flag);
        self.handle = None;
        self
    }

    pub fn listen(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            let mut buf = [0u8; 256];
            let n = self.stream.read(&mut buf).unwrap();
            if n == 256 {
                let head = DataHead::deserialize(buf);
                let channel = String::from_utf8(head.channel.to_vec())?;
                if !self.channel_manager.contains(&channel) {
                    let ch = Channel::new(channel.clone());
                    self.channel_manager.add(ch);
                }

                let size = head.slice_size;
                let count = head.slice_count;

                let mut channel = self.channel_manager.get(&channel.clone()).unwrap();
                channel.set_receiving(size * count);
                // slice1(data_head, data[size]), slice2(data_head, data[size]), ...

                let mut completed = false;
                for _ in 0..size / 256 {
                    let mut buf = [0u8; 256];
                    let n = self.stream.read(&mut buf).unwrap();
                    completed = channel.write_buffer(Vec::from(&buf));
                }

                if completed {
                    // todo: handle data; read data from buffer, send to router. deal with commands & normal msg
                    let mut buf = channel.read_buffer();
                    let host = String::from_utf8(head.virtual_host.to_vec())?;
                }
            }
        }
    }
}