use std::cell::RefCell;
use std::error::Error;
use std::io::{Read, Write};
use std::io::ErrorKind::UnexpectedEof;
use std::net::{SocketAddr, TcpStream};
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::mq::net::chan::Channel;
use crate::mq::net::manager::{ChannelManager, PhysicalConnectionManager};
use crate::mq::protocol::proto::DataHead;
use crate::mq::protocol::protobase::Deserialize;

pub struct PhysicalConnection {
    pub local_addr: SocketAddr,
    pub remote_addr: SocketAddr,

    pub stream: RefCell<TcpStream>,
    pub closed: RefCell<bool>,

    pub channel_manager: RefCell<ChannelManager>
}

impl PhysicalConnection {

    pub fn launch(mut self) -> Arc<Mutex<PhysicalConnection>> {
        let s0 = Arc::new(Mutex::new(self));
        let s1 = s0.clone();
        thread::spawn(move || {
            s1.lock().unwrap().listen().unwrap();
        });
        s0
    }

    pub fn listen(&self) -> Result<(), Box<dyn Error>> {
        loop {
            let mut buf = [0u8; 256];
            self.stream.borrow_mut().set_nodelay(false)?;
            let n = self.stream.borrow_mut().read_exact(&mut buf);
            match n {
                Ok(_) => {
                    dbg!("{:?}", String::from_utf8(buf.to_vec()).unwrap());
                    self.stream.borrow_mut().flush().unwrap_or(());

                    let head = DataHead::deserialize(buf);
                    let channel = String::from_utf8(head.channel.to_vec())?;
                    if !self.channel_manager.borrow_mut().contains(&channel) {
                        let ch = Channel::new(channel.clone());
                        self.channel_manager.borrow_mut().add(ch);
                    }

                    let size = u64::from(head.slice_size);
                    let count = u64::from(head.slice_count) ;

                    let mut channel_manager = self.channel_manager.borrow_mut();
                    let channel = channel_manager
                        .get(&channel.clone())
                        .unwrap();

                    channel.set_receiving(u64::from(size) * u64::from(count));
                    // slice1(data_head, data[size]), slice2(data_head, data[size]), ...

                    let mut completed = false;
                    const SLICE_SIZE: u64 = 256;
                    dbg!(size, count);
                    for i in 0..(size * count / SLICE_SIZE) {
                        self.stream.borrow_mut().flush().unwrap_or(());
                        let mut buf = [0u8; SLICE_SIZE as usize];
                        'read: loop {
                            if let Ok(_) = self.stream.borrow_mut().read_exact(&mut buf) {
                                dbg!("read!", i);
                                dbg!("buf: ", String::from_utf8(buf.to_vec()).unwrap());
                                completed = channel.write_buffer(Vec::from(&buf), SLICE_SIZE);
                                //dbg!(channel.peek_buffer().last().unwrap());
                                dbg!(channel.peek_buffer().len() / 256);
                                break 'read;
                            }
                        }
                        /*
                        let n = self.stream.borrow_mut()
                            .read(&mut buf)
                            .unwrap();
                        */
                    }

                    if completed {
                        // todo: handle data; read data from buffer, send to router. deal with commands & normal msg
                        let buf = channel.read_buffer();
                        //dbg!(buf);
                        let host = String::from_utf8(head.virtual_host.to_vec())?;
                    }
                }
                Err(e) => {
                    if e.kind() == UnexpectedEof {
                        self.closed.replace(true);
                        break
                    }
                }
            }
        }
        Ok(())
    }
}