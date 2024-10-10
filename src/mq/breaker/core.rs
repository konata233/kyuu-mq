use crate::mq::net::factory::PhysicalConnectionFactory;
use crate::mq::net::manager::PhysicalConnectionManager;
use std::net::{TcpListener, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::thread;
use crate::mq::common::proxy::ProxyHolder;
use crate::mq::host::manager::HostManager;
use crate::mq::host::vhost::VirtualHost;
use crate::mq::protocol::raw::RawData;
use crate::mq::queue::queue_object::QueueObject;

pub struct Breaker {
    tcp_listener: TcpListener,
    host_manager: Option<Arc<Mutex<ProxyHolder<HostManager>>>>,
    physical_connection_manager: Option<Arc<Mutex<ProxyHolder<PhysicalConnectionManager>>>>,
}

impl Breaker {
    pub fn new<A: ToSocketAddrs>(addr: A) -> Breaker {
        Breaker {
            tcp_listener: TcpListener::bind(addr).unwrap(),
            host_manager: None,
            physical_connection_manager: None,
        }
    }

    pub fn init_managers(&mut self, self_ref: Arc<Mutex<Breaker>>) {
        let host_manager = Arc::new(
            Mutex::new(
                ProxyHolder::new(HostManager::new().init(self_ref.clone())).init()
            )
        );
        self.host_manager = Some(
            host_manager.clone()
        );

        self.physical_connection_manager = Some(
            Arc::new(
                Mutex::new(
                    ProxyHolder::new(PhysicalConnectionManager::new().init(self_ref.clone(), host_manager.clone())).init()
                )
            )
        );
    }


    pub fn start_worker(&mut self) {
        thread::scope(|scope| {
             scope.spawn(|| {
                self.listen().unwrap();
            }).join().unwrap();
        })
    }

    pub fn send_raw_to_host(&mut self, data: RawData) -> Option<QueueObject> {
        self.host_manager.as_mut()?
            .lock()
            .unwrap()
            .get()
            .lock()
            .unwrap()
            .borrow_mut()
            .send_raw_to_host(data)
    }

    pub fn stop_worker(&mut self) {
        self.physical_connection_manager.as_mut()
            .unwrap()
            .lock()
            .unwrap()
            .get()
            .lock()
            .unwrap()
            .borrow_mut()
            .close();
    }

    pub fn listen(&mut self) -> Result<(), ()> {
        for incoming in self.tcp_listener.incoming() {
            if let Ok(stream) = incoming {
                let conn = PhysicalConnectionFactory::new()
                    .set_manager_proxy(self.physical_connection_manager.clone())
                    .set_stream(stream)
                    .fetch();
                self.physical_connection_manager.as_mut()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .get()
                    .lock()
                    .unwrap()
                    .get_mut()
                    .add(conn?);
            }
        }
        Ok(())
    }
}

pub struct Core {
    breaker: Arc<Mutex<Breaker>>,
}

impl Core {
    pub fn new<A: ToSocketAddrs>(addr: A) -> Core {
        let mut breaker = Breaker::new(addr);

        let self_ref = Arc::new(Mutex::new(breaker));
        self_ref.lock()
            .as_mut()
            .unwrap()
            .init_managers(self_ref.clone());

        // default host
        let default_name = String::from("MQ_HOST");
        self_ref.lock().unwrap().host_manager.clone()
            .unwrap()
            .lock()
            .unwrap()
            .get()
            .lock()
            .unwrap()
            .borrow_mut()
            .add(default_name.clone(), VirtualHost::new(default_name));

        Core {
            breaker: self_ref,
        }
    }

    pub fn start(&mut self) {
        self.breaker.lock().unwrap().start_worker();
    }

    pub fn stop(&mut self) {
        self.breaker.lock().unwrap().stop_worker();
    }
}