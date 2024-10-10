use crate::mq::protocol::raw::{Raw, RawCommand, RawData, RawMessage};
use crate::mq::queue::qbase::Queue;
use crate::mq::queue::queue_object::QueueObject;
use crate::mq::routing::exchange::Exchange;
use crate::mq::routing::key::RoutingKey;
use std::sync::{Arc, Mutex};

pub struct VirtualHost {
    pub name: String,
    base_exchange: Exchange
}

impl VirtualHost {
    pub fn new(name: String) -> VirtualHost {
        VirtualHost {
            name,
            base_exchange: Exchange::new(String::from("mq-root")),
        }
    }

    pub fn add_exchange(&mut self, name: String, routing_key: RoutingKey) -> &mut Self {
        let base = self.base_exchange.walk(routing_key, 0);
        if let Some(mut exc) = base {
            exc[0].add_exchange(name);
        }
        self
    }

    pub fn add_queue(&mut self, name: String, routing_key: RoutingKey) -> &mut Self {
        let base = self.base_exchange.walk(routing_key, 0);
        if let Some(mut exc) = base {
            exc[0].add_queue(&name);
        }
        self
    }

    pub fn get_queue(&mut self, name: &String, routing_key: RoutingKey) -> Option<Arc<Mutex<Queue>>> {
        let base = self.base_exchange.walk(routing_key, 0);
        if let Some(mut exc) = base {
            exc.get_mut(0)?.get_queue(name)
        } else {
            None
        }
    }


    pub fn drop_exchange(&mut self, name: String, routing_key: RoutingKey) -> &mut Self {
        let base = self.base_exchange.walk(routing_key, 0);
        if let Some(mut exc) = base {
            exc[0].remove_exchange(name);
        }
        self
    }

    pub fn drop_queue(&mut self, name: String, routing_key: RoutingKey) -> &mut Self {
        let base = self.base_exchange.walk(routing_key, 0);
        if let Some(mut exc) = base {
            exc[0].remove_queue(name);
        }
        self
    }

    pub fn process_incoming(&mut self, raw: RawData) -> Option<QueueObject> {
        // todo: implement advanced routing: * # ...
        // always remember that the last value of RoutingKey is the name of the Queue.
        let routing = raw.routing_key;
        let routing_copied = routing.clone();

        let queue_name = match routing {
            RoutingKey::Direct(key) => key[3].clone(),
            RoutingKey::Topic(key) => key[3].clone(),
            RoutingKey::Fanout(key) => key[3].clone(),
        };

        let exchange = self.base_exchange.walk(routing_copied, 0);
        if let Some(mut exc) = exchange {
            match raw.raw {
                Raw::Command(cmd) => {
                    match cmd {
                        RawCommand::NewQueue(data) => {
                            dbg!("new queue");
                            let queue_name = String::from_utf8(data).unwrap().trim().to_string();
                            exc[0].add_exchange(queue_name);
                        }
                        RawCommand::NewExchange(data) => {
                            dbg!("new exchange");
                            let exchange_name = String::from_utf8(data).unwrap().trim().to_string();
                            exc[0].add_exchange(exchange_name);
                        }
                        RawCommand::NewBinding(data) => {
                            dbg!("new binding");
                        }
                        RawCommand::DropQueue(data) => {
                            let queue_name = String::from_utf8(data).unwrap().trim().to_string();
                            exc[0].remove_queue(queue_name);
                        }
                        RawCommand::DropExchange(data) => {
                            dbg!("drop exchange");
                            let exchange_name = String::from_utf8(data).unwrap().trim().to_string();
                            exc[0].remove_exchange(exchange_name);
                        }
                        RawCommand::DropBinding(data) => {
                            dbg!("drop binding");
                        }
                        RawCommand::Nop => {
                            dbg!("nop");
                        }
                    }
                }
                Raw::Message(data) => {
                    dbg!("message");
                    match data {
                        RawMessage::Push(data) => {
                            exc[0].get_queue(&queue_name)?.lock()
                                .unwrap()
                                .push_back(QueueObject::new(&self.name, data))
                        },
                        RawMessage::Fetch(data) => {
                            let queue_name = String::from_utf8(data).unwrap().trim().to_string();
                            let obj = exc[0].get_queue(&queue_name)?.lock()
                                .unwrap()
                                .pop_front()?;
                            return Some(obj)
                        }
                        RawMessage::Nop => {
                            dbg!("nop");
                        }
                    }
                }
                Raw::Nop => {
                    dbg!("nop");
                }
            }
        }
        None
    }
}