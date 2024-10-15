pub struct QueueObject {
    pub virtual_host: String,
    pub content: Vec<u8>
}

impl QueueObject {
    pub fn new(virtual_host: &String, content: Vec<u8>) -> QueueObject {
        QueueObject {
            virtual_host: virtual_host.clone(),
            content
        }
    }
}

impl Clone for QueueObject {
    fn clone(&self) -> Self {
        QueueObject {
            virtual_host: self.virtual_host.clone(),
            content: self.content.clone()
        }
    }
}