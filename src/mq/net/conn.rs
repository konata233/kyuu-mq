use crate::mq::host::manager::HostManager;
use crate::mq::host::vhost::VirtualHost;
use crate::mq::net::chan::Channel;
use crate::mq::net::manager::{ChannelManager, PhysicalConnectionManager};
use crate::mq::protocol::proto::DataHead;
use crate::mq::protocol::protobase::{Deserialize, Serialize};
use crate::mq::protocol::raw::{IOType, Raw, RawCommand, RawData, RawMessage};
use crate::mq::routing::key::RoutingKey;
use std::cell::RefCell;
use std::error::Error;
use std::io::ErrorKind::UnexpectedEof;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

pub struct PhysicalConnection {
    pub local_addr: SocketAddr,
    pub remote_addr: SocketAddr,

    pub stream: RefCell<TcpStream>,
    pub closed: RefCell<bool>,

    pub manager_proxy: Arc<RwLock<PhysicalConnectionManager>>,
    pub channel_manager: RefCell<ChannelManager>,
}

impl PhysicalConnection {
    pub fn launch(self) -> Arc<Mutex<PhysicalConnection>> {
        let s0 = Arc::new(Mutex::new(self));
        let s1 = s0.clone();
        thread::spawn(move || {
            s1.lock().unwrap().listen().unwrap();
        });
        s0
    }

    fn process(&self, data_head: &DataHead, buffer: Vec<u8>) -> RawData {
        let mut io_type = IOType::Write;

        let channel = String::from_utf8(data_head.channel.clone().to_vec())
            .unwrap()
            .trim_end_matches("\0") // IMPORTANT!!
            .to_string();
        let virtual_host = String::from_utf8(data_head.virtual_host.clone().to_vec())
            .unwrap()
            .trim_end_matches("\0")
            .to_string();

        // match first byte of routing_mod as command type
        let raw: Raw = match data_head.routing_mod[0] {
            0u8 => {
                // dbg!("normal msg");
                // match second byte of routing_mod as msg type
                let msg: RawMessage = match data_head.routing_mod[1] {
                    0u8 => {
                        // dbg!("normal push");
                        RawMessage::Push(buffer)
                    }
                    1u8 => {
                        // dbg!("normal fetch");
                        io_type = IOType::Read;
                        RawMessage::Fetch(buffer)
                    }
                    _ => {
                        // dbg!("nop");
                        RawMessage::Nop
                    }
                };
                Raw::Message(msg)
            }
            1u8 => {
                // dbg!("command");
                // match second byte of routing_mod as command type
                let cmd: RawCommand = match data_head.routing_mod[1] {
                    0u8 => {
                        // dbg!("new queue");
                        RawCommand::NewQueue(buffer)
                    }
                    1u8 => {
                        // dbg!("new exchange");
                        RawCommand::NewExchange(buffer)
                    }
                    2u8 => {
                        // dbg!("new binding");
                        RawCommand::NewBinding(buffer)
                    }
                    3u8 => {
                        // dbg!("drop queue");
                        RawCommand::DropQueue(buffer)
                    }
                    4u8 => {
                        // dbg!("drop exchange");
                        RawCommand::DropExchange(buffer)
                    }
                    5u8 => {
                        // dbg!("drop binding");
                        RawCommand::DropBinding(buffer)
                    }
                    _ => {
                        // dbg!("nop");
                        RawCommand::Nop
                    }
                };
                Raw::Command(cmd)
            }
            _ => {
                // dbg!("nop");
                Raw::Nop
            }
        };
        // // dbg!(&raw);

        let routing_arr = [
            String::from_utf8(data_head.route0.clone().to_vec())
                .unwrap()
                .trim_end_matches("\0")
                .to_string(),
            String::from_utf8(data_head.route1.clone().to_vec())
                .unwrap()
                .trim_end_matches("\0")
                .to_string(),
            String::from_utf8(data_head.route2.clone().to_vec())
                .unwrap()
                .trim_end_matches("\0")
                .to_string(),
            String::from_utf8(data_head.route3.clone().to_vec())
                .unwrap()
                .trim_end_matches("\0")
                .to_string(),
        ];

        // match third byte of routing_mod as routing type
        let routing: RoutingKey = match data_head.routing_mod[2] {
            0u8 => {
                // dbg!("direct");
                RoutingKey::Direct(routing_arr)
            }
            1u8 => {
                // dbg!("topic");
                RoutingKey::Topic(routing_arr)
            }
            2u8 => {
                // dbg!("fanout");
                RoutingKey::Fanout(routing_arr)
            }
            _ => {
                // dbg!("nop");
                RoutingKey::Direct(routing_arr)
            }
        };

        RawData {
            raw,
            channel: channel.clone(),
            virtual_host: virtual_host.clone(),
            routing_key: routing,
            io_type,
        }
    }

    fn get_host_manager_proxy(&self, io_type: &IOType) -> Arc<RwLock<HostManager>> {
        match io_type {
            IOType::Read => {
                // dbg!("read");
                self.manager_proxy
                    .clone()
                    .read()
                    .unwrap()
                    .host_manager
                    .clone()
                    .unwrap()
            }
            IOType::Write => {
                // dbg!("write");
                self.manager_proxy
                    .clone()
                    .write()
                    .unwrap()
                    .host_manager
                    .clone()
                    .unwrap()
            }
        }
    }

    pub fn listen(&self) -> Result<(), Box<dyn Error>> {
        'listen: loop {
            let mut buf = [0u8; 256];
            self.stream.borrow_mut().set_nodelay(false)?;
            let n = self.stream.borrow_mut().read_exact(&mut buf);
            match n {
                Ok(_) => {
                    // dbg!("incoming transmission!");
                    self.stream.borrow_mut().flush().unwrap_or(());

                    let head = DataHead::deserialize(buf);
                    let channel = String::from_utf8(head.channel.clone().to_vec())?;
                    if !self.channel_manager.borrow_mut().contains(&channel) {
                        let ch = Channel::new(channel.clone());
                        self.channel_manager.borrow_mut().add(ch);
                    }

                    let size = u64::from(head.slice_size);
                    // note that the head is not included when calculating 'count'.
                    let count = u64::from(head.slice_count);

                    let mut channel_manager = self.channel_manager.borrow_mut();
                    let channel = channel_manager.get(&channel.clone()).unwrap();

                    channel.set_receiving(u64::from(size) * u64::from(count));
                    // slice1(data_head, data[size]), slice2(data_head, data[size]), ...

                    let mut completed = false || size * count == 0; // if the package doesn't have body.
                    const SLICE_SIZE: u64 = 256;
                    // dbg!(size, count);
                    for _ in 0..(size * count / SLICE_SIZE) {
                        self.stream.borrow_mut().flush().unwrap_or(());
                        let mut buf = [0u8; SLICE_SIZE as usize];
                        'read: loop {
                            match self.stream.borrow_mut().read_exact(&mut buf) {
                                Ok(_) => {
                                    // dbg!("read!", i);
                                    // dbg!("buf: ", String::from_utf8(buf.to_vec()).unwrap());
                                    completed = channel.write_buffer(Vec::from(&buf), SLICE_SIZE);
                                    //// dbg!(channel.peek_buffer().last().unwrap());
                                    // dbg!(channel.peek_buffer().len() / 256);
                                    break 'read;
                                }
                                Err(e) => {
                                    if e.kind() == UnexpectedEof {
                                        // dbg!("conn closed.");
                                        self.closed.replace(true);
                                        break 'listen;
                                    }
                                }
                            }
                        }
                    }

                    if completed {
                        // always remember that the last value of RoutingKey is the name of the Queue.
                        let buf = channel.read_buffer();
                        let raw = self.process(&head, buf);
                        let host = VirtualHost::new(raw.virtual_host.clone());

                        let io_type = &raw.io_type;
                        let result = self
                            .get_host_manager_proxy(io_type)
                            .read()
                            .unwrap()
                            .send_raw_to_host(raw);

                        // todo: I see no difference whether to use read() or write().
                        // but that remains to be tested.
                        /*match io_type {
                            IOType::Read => {
                                self.get_host_manager_proxy(io_type)
                                    .read()
                                    .unwrap()
                                    .send_raw_to_host(raw)
                            }
                            IOType::Write => {
                                self.get_host_manager_proxy(io_type)
                                    .write()
                                    .unwrap()
                                    .send_raw_to_host(raw)
                            }
                        };*/

                        // seems that when lock is acquired here, send_raw_data() can't use it, causing deadlock.
                        if let Some(feedback) = result {
                            let mut buffer = feedback.content;
                            if buffer.len() % 256 != 0 {
                                // dbg!("buffer size not aligned to 256 bytes!", buffer.len());
                                let align = 256 - buffer.len() % 256;
                                for _ in 0..align {
                                    buffer.push(0u8);
                                }
                            }

                            // send feedback.
                            let channel = head.channel.clone();
                            let slice_count = buffer.len() / 256;
                            let data_head = DataHead::new(
                                host,
                                channel,
                                [0u8; 4],
                                [0u8; 24],
                                [0u8; 128],
                                slice_count as u32,
                                SLICE_SIZE as u32,
                                0,
                                0,
                            );
                            let mut head_serialized = data_head.serialize_vec();
                            head_serialized.append(&mut buffer);
                            let concatenated = head_serialized;
                            self.stream
                                .borrow_mut()
                                .write_all(concatenated.as_slice())
                                .unwrap();
                            // dbg!("feedback sent!");
                            //// dbg!(&concatenated);
                        }
                    }

                    let channel_name = channel.name.clone();
                    let command = String::from_utf8(head.command.to_vec()).unwrap();
                    drop(channel_manager);

                    // todo: (IMPORTANT) add logic for closing the channel using attr 'command'
                    let cmd = command
                        .to_uppercase()
                        .as_str()
                        .trim_end_matches("\0")
                        .to_string();
                    // dbg!(&cmd);
                    match cmd.as_str() {
                        "CLOSE-CH" => {
                            // dbg!("close channel");
                            self.channel_manager
                                .borrow_mut()
                                .remove(channel_name.as_str());
                        }
                        _ => {
                            // dbg!("nop(cmd)");
                        }
                    }
                }
                Err(e) => {
                    if e.kind() == UnexpectedEof {
                        // dbg!("conn closed.");
                        self.closed.replace(true);
                        break 'listen;
                    }
                }
            }
        }
        Ok(())
    }
}
