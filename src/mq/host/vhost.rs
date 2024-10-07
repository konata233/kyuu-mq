use crate::mq::queue::manager::QueueManager;
use crate::mq::queue::qbase::Queue;
use crate::mq::queue::queue_object::QueueObject;

pub struct VirtualHost {
    pub name: String,
    queue_manager: QueueManager<QueueObject>
}

impl VirtualHost {
    pub fn new(name: String) -> VirtualHost {
        VirtualHost {
            name,
            queue_manager: QueueManager::new()
        }
    }

    pub fn add_queue(&mut self, name: String) -> &Self {
        self.queue_manager.add(&name);
        self
    }

    pub fn get_queue(&mut self, name: String) -> Option<&mut Queue<QueueObject>> {
        self.queue_manager.get(name)
    }

    pub fn remove_queue(&mut self, name: String) -> Option<Queue<QueueObject>> {
        self.queue_manager.remove(name)
    }

    pub fn get_all_queues(&self) -> Vec<&Queue<QueueObject>> {
        self.queue_manager.get_all()
    }

    pub fn clear_queue(&mut self, name: String) -> &Self {
        self.queue_manager.get(name).unwrap().clear();
        self
    }

    pub fn process_incoming(&mut self, buffer: Vec<u8>) {

    }
}