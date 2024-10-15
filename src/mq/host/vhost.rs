use crate::mq::protocol::raw::{Raw, RawCommand, RawData, RawMessage};
use crate::mq::queue::qbase::Queue;
use crate::mq::queue::queue_object::QueueObject;
use crate::mq::routing::exchange::Exchange;
use crate::mq::routing::key::RoutingKey;
use std::sync::{Arc, RwLock};

pub struct VirtualHost {
    pub name: String,
    base_exchange: Arc<RwLock<Exchange>>,
}

impl VirtualHost {
    pub fn new(name: String) -> VirtualHost {
        let exchange = Arc::from(RwLock::from(Exchange::new(String::from("mq-root"))));
        exchange.write().unwrap().init(exchange.clone());
        VirtualHost {
            name,
            base_exchange: exchange,
        }
    }

    pub fn add_exchange(&mut self, name: String, routing_key: RoutingKey) -> &mut Self {
        let base = self.base_exchange.write().unwrap().walk(routing_key, 0);
        if let Some(exc) = base {
            exc[0].write().unwrap().add_exchange(name);
        }
        self
    }

    pub fn add_queue(&mut self, name: String, routing_key: RoutingKey) -> &mut Self {
        let base = self.base_exchange.write().unwrap().walk(routing_key, 0);
        if let Some(exc) = base {
            exc[0].write().unwrap().add_queue(&name);
        }
        self
    }

    pub fn get_queue(&self, name: &String, routing_key: RoutingKey) -> Option<Arc<RwLock<Queue>>> {
        let base = self
            .base_exchange
            .read()
            .unwrap()
            .walk_readonly(routing_key, 0);
        if let Some(exc) = base {
            exc.get(0)?.read().unwrap().get_queue(name)
        } else {
            None
        }
    }

    pub fn drop_exchange(&mut self, name: String, routing_key: RoutingKey) -> &mut Self {
        let base = self.base_exchange.write().unwrap().walk(routing_key, 0);
        if let Some(exc) = base {
            exc[0].write().unwrap().remove_exchange(name);
        }
        self
    }

    pub fn drop_queue(&mut self, name: String, routing_key: RoutingKey) -> &mut Self {
        let base = self.base_exchange.write().unwrap().walk(routing_key, 0);
        if let Some(exc) = base {
            exc[0].write().unwrap().remove_queue(name);
        }
        self
    }

    pub fn process_incoming(&self, raw: RawData, err_handle: &mut u16) -> Option<QueueObject> {
        // todo: implement advanced routing: * # ...
        // always remember that the last value of RoutingKey is the name of the Queue.

        // println!("[mq] incoming data: {:?}", raw);
        println!("[mq] incoming data.");
        let routing = raw.routing_key;
        let routing_copied = routing.clone();
        let host = raw.virtual_host.trim_end_matches("\0").to_string();

        let queue_name = match routing {
            RoutingKey::Direct(key) => key[3].clone(),
            RoutingKey::Topic(key) => key[3].clone(),
            RoutingKey::Fanout(key) => key[3].clone(),
        };

        let exchange = self.base_exchange.write().unwrap().walk(routing_copied, 0);
        if let Some(mut exc) = exchange {
            match raw.raw {
                Raw::Command(cmd) => {
                    match cmd {
                        RawCommand::NewQueue(data) => {
                            // dbg!("new queue");
                            let queue_name = String::from_utf8(data)
                                .unwrap()
                                .trim_end_matches("\0")
                                .trim()
                                .to_string();
                            exc[0].write().unwrap().add_queue(&queue_name);
                        }
                        RawCommand::NewExchange(data) => {
                            // dbg!("new exchange");
                            let exchange_name = String::from_utf8(data)
                                .unwrap()
                                .trim_end_matches("\0")
                                .trim()
                                .to_string();
                            exc[0].write().unwrap().add_exchange(exchange_name);
                        }
                        RawCommand::NewBinding(_) => {
                            // dbg!("new binding");
                        }
                        RawCommand::DropQueue(data) => {
                            let queue_name = String::from_utf8(data)
                                .unwrap()
                                .trim_end_matches("\0")
                                .trim()
                                .to_string();
                            exc[0].write().unwrap().remove_queue(queue_name);
                        }
                        RawCommand::DropExchange(data) => {
                            // dbg!("drop exchange");
                            let exchange_name = String::from_utf8(data)
                                .unwrap()
                                .trim_end_matches("\0")
                                .trim()
                                .to_string();
                            exc[0].write().unwrap().remove_exchange(exchange_name);
                        }
                        RawCommand::DropBinding(_) => {
                            // dbg!("drop binding");
                        }
                        RawCommand::Nop => {
                            // dbg!("nop");
                        }
                    }
                }
                Raw::Message(data) => {
                    // dbg!("message");
                    match data {
                        RawMessage::Push(data) => {
                            // dbg!("push");
                            exc[0]
                                .read()
                                .unwrap()
                                .get_queue(&queue_name)?
                                .write()
                                .unwrap()
                                .push_back(QueueObject::new(&self.name, data))
                        }
                        RawMessage::Fetch(data) => {
                            // dbg!("fetch");
                            match exc[0]
                                .read()
                                .unwrap()
                                .get_queue(&queue_name)?
                                .write()
                                .unwrap()
                                .pop_front() {
                                Some(obj) => return Some(obj),
                                None => {
                                    *err_handle = 0xf;
                                    return Some(QueueObject::new(&"".to_string(), Vec::new()));
                                }
                            };
                            // the 'write' for queue is temporary. but I have no idea how to optimize it.
                        }
                        RawMessage::Nop => {
                            // dbg!("nop");
                        }
                    }
                }
                Raw::Nop => {
                    // dbg!("nop");
                }
            }
        }
        None
        // Some(QueueObject::new(&host, String::from("success!").into_bytes()))
    }
}
