use crate::mq::common::proxy::ProxyHolder;
use crate::mq::net::chan::Channel;
use crate::mq::net::manager::{ChannelManager, PhysicalConnectionManager};
use crate::mq::protocol::proto::DataHead;
use crate::mq::protocol::protobase::Deserialize;
use crate::mq::protocol::raw::{Raw, RawCommand, RawData, RawMessage};
use crate::mq::routing::key::RoutingKey;
use std::cell::RefCell;
use std::error::Error;
use std::io::ErrorKind::UnexpectedEof;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct PhysicalConnection {
    pub local_addr: SocketAddr,
    pub remote_addr: SocketAddr,

    pub stream: RefCell<TcpStream>,
    pub closed: RefCell<bool>,

    pub manager_proxy: Arc<Mutex<ProxyHolder<PhysicalConnectionManager>>>,
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

    fn process(&self, data_head: &DataHead, buffer: Vec<u8>) -> RawData {
        let channel = String::from_utf8(data_head.channel.clone().to_vec());
        let virtual_host = String::from_utf8(data_head.virtual_host.clone().to_vec());

        // match first byte of routing_mod as command type
        let mut raw: Raw = match data_head.routing_mod[0] {
            0u8 => {
                dbg!("normal msg");
                // match second byte of routing_mod as msg type
                let msg: RawMessage = match data_head.routing_mod[1] {
                    0u8 => {
                        dbg!("normal push");
                        RawMessage::Push(buffer)
                    }
                    1u8 => {
                        dbg!("normal fetch");
                        RawMessage::Fetch(buffer)
                    }
                    _ => {
                        dbg!("nop");
                        RawMessage::Nop
                    },
                };
                Raw::Message(msg)
            }
            1u8 => {
                dbg!("command");
                // match second byte of routing_mod as command type
                let cmd: RawCommand = match data_head.routing_mod[1] {
                    0u8 => {
                        dbg!("new queue");
                        RawCommand::NewQueue(buffer)
                    },
                    1u8 => {
                        dbg!("new exchange");
                        RawCommand::NewExchange(buffer)
                    },
                    2u8 => {
                        dbg!("new binding");
                        RawCommand::NewBinding(buffer)
                    },
                    3u8 => {
                        dbg!("drop queue");
                        RawCommand::DropQueue(buffer)
                    },
                    4u8 => {
                        dbg!("drop exchange");
                        RawCommand::DropExchange(buffer)
                    },
                    5u8 => {
                        dbg!("drop binding");
                        RawCommand::DropBinding(buffer)
                    },
                    _ => {
                        dbg!("nop");
                        RawCommand::Nop
                    }
                };
                Raw::Command(cmd)
            }
            _ => {
                dbg!("nop");
                Raw::Nop
            }
        };
        dbg!(&raw);
        
        let routing_arr = [
            String::from_utf8(data_head.route0.clone().to_vec()).unwrap(),
            String::from_utf8(data_head.route1.clone().to_vec()).unwrap(),
            String::from_utf8(data_head.route2.clone().to_vec()).unwrap(),
            String::from_utf8(data_head.route3.clone().to_vec()).unwrap()
        ];
        
        // match third byte of routing_mod as routing type
        let routing: RoutingKey = match data_head.routing_mod[2] {
            0u8 => {
                dbg!("normal");
                RoutingKey::Direct(routing_arr)
            }
            1u8 => {
                dbg!("topic");
                RoutingKey::Topic(routing_arr)
            }
            2u8 => {
                dbg!("fanout");
                RoutingKey::Fanout(routing_arr)
            },
            _ => {
                dbg!("nop");
                RoutingKey::Direct(routing_arr)
            },
        };
        
        RawData {
            raw,
            channel: channel.unwrap(),
            virtual_host: virtual_host.unwrap(),
            routing_key: routing
        }
    }

    pub fn listen(&self) -> Result<(), Box<dyn Error>> {
        'listen: loop {
            let mut buf = [0u8; 256];
            self.stream.borrow_mut().set_nodelay(false)?;
            let n = self.stream.borrow_mut().read_exact(&mut buf);
            match n {
                Ok(_) => {
                    dbg!("conn established!");
                    self.stream.borrow_mut().flush().unwrap_or(());

                    let head = DataHead::deserialize(buf);
                    let channel = String::from_utf8(head.channel.clone().to_vec())?;
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
                            match self.stream.borrow_mut().read_exact(&mut buf) {
                                Ok(_) => {
                                    dbg!("read!", i);
                                    dbg!("buf: ", String::from_utf8(buf.to_vec()).unwrap());
                                    completed = channel.write_buffer(Vec::from(&buf), SLICE_SIZE);
                                    //dbg!(channel.peek_buffer().last().unwrap());
                                    dbg!(channel.peek_buffer().len() / 256);
                                    break 'read;
                                }
                                Err(e) => {
                                    if e.kind() == UnexpectedEof {
                                        dbg!("conn closed.");
                                        self.closed.replace(true);
                                        break 'listen;
                                    }
                                }
                            }
                        }
                    }

                    if completed {
                        // todo: handle data; read data from buffer, send to router. deal with commands & normal msg
                        let buf = channel.read_buffer();
                        let raw = self.process(&head, buf);

                        let result = self.manager_proxy
                            .clone()
                            .lock()
                            .unwrap()
                            .get()
                            .clone()
                            .lock()
                            .unwrap()
                            .borrow_mut()
                            .send_raw_data(raw);

                        // todo: send feedback to remote host.
                    }
                }
                Err(e) => {
                    if e.kind() == UnexpectedEof {
                        dbg!("conn closed.");
                        self.closed.replace(true);
                        break 'listen;
                    }
                }
            }
        }
        Ok(())
    }
}