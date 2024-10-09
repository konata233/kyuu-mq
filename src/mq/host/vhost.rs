use crate::mq::protocol::raw::{Raw, RawCommand, RawData};
use crate::mq::queue::queue_object::QueueObject;
use crate::mq::routing::exchange::Exchange;
use crate::mq::routing::key::RoutingKey;

pub struct VirtualHost {
    pub name: String,
    base_exchange: Exchange<QueueObject>
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

    pub fn get_queue(&mut self, name: String, routing_key: RoutingKey) -> &mut Self {
        let base = self.base_exchange.walk(routing_key, 0);
        if let Some(mut exc) = base {
            exc[0].get_queue(name);
        }
        self
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

    pub fn process_incoming(&mut self, raw: RawData) {
        // todo: routing
        let routing = raw.routing_key;
        let exchange = self.base_exchange.walk(routing, 0);
        if let Some(exc) = exchange {
            match raw.raw {
                Raw::Command(cmd) => {
                    match cmd {
                        RawCommand::NewQueue(data) => {
                            let queue_name = String::from_utf8(data).unwrap().trim();

                        }
                        RawCommand::NewExchange(data) => {
                            println!("new exchange");
                        }
                        RawCommand::NewBinding(data) => {
                            println!("new binding");
                        }
                        RawCommand::DropQueue(data) => {
                            let queue_name = String::from_utf8(data).unwrap().trim();

                        }
                        RawCommand::DropExchange(data) => {
                            println!("drop exchange");
                        }
                        RawCommand::DropBinding(data) => {
                            println!("drop binding");
                        }
                        RawCommand::Nop => {}
                    }
                }
            }
        }
    }
}