use crate::mq::host::vhost::VirtualHost;

pub struct QueueObject {
    pub virtual_host: String,
    pub routing_key: [String; 4],
    pub content: Vec<u8>
}

impl QueueObject {
    pub fn new(virtual_host: &String, routing_key: [String; 4], content: Vec<u8>) -> QueueObject {
        QueueObject {
            virtual_host: virtual_host.clone(),
            routing_key,
            content
        }
    }
}

impl Clone for QueueObject {
    fn clone(&self) -> Self {
        QueueObject {
            virtual_host: self.virtual_host.clone(),
            routing_key: self.routing_key.clone(),
            content: self.content.clone()
        }
    }
}