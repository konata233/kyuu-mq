use crate::mq::common::context::RuntimeContext;
use crate::mq::host::manager::HostManager;
use crate::mq::host::vhost::VirtualHost;
use crate::mq::net::factory::PhysicalConnectionFactory;
use crate::mq::net::manager::PhysicalConnectionManager;
use crate::mq::protocol::raw::RawData;
use crate::mq::queue::queue_object::QueueObject;
use std::net::{TcpListener, ToSocketAddrs};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

pub struct Breaker {
    tcp_listener: TcpListener,
    host_manager: Option<Arc<RwLock<HostManager>>>,
    physical_connection_manager: Option<Arc<RwLock<PhysicalConnectionManager>>>,
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
        let host_manager = Arc::new(RwLock::new(HostManager::new().init(self_ref.clone())));
        self.host_manager = Some(host_manager.clone());

        self.physical_connection_manager = Some(Arc::new(RwLock::new(
            PhysicalConnectionManager::new().init(self_ref.clone(), host_manager.clone()),
        )));
    }

    pub fn start_worker(&mut self) {
        thread::scope(|scope| {
            scope
                .spawn(|| {
                    self.listen().unwrap();
                })
                .join()
                .unwrap();
        })
    }

    pub fn send_raw_to_host(&mut self, data: RawData, err_handle: &mut u16) -> Option<QueueObject> {
        self.host_manager
            .as_mut()?
            .write()
            .unwrap()
            .send_raw_to_host(data, err_handle)
    }

    pub fn stop_worker(&mut self) {
        self.physical_connection_manager
            .as_mut()
            .unwrap()
            .write()
            .unwrap()
            .close();
    }

    pub fn listen(&mut self) -> Result<(), ()> {
        for incoming in self.tcp_listener.incoming() {
            if let Ok(stream) = incoming {
                // Note: this is a hack to prevent self-connection
                // in the meantime, this hack is just temporary.
                if stream.peer_addr().unwrap() == stream.local_addr().unwrap() {
                    continue;
                }
                dbg!("new connection");
                let conn = PhysicalConnectionFactory::new()
                    .set_manager_proxy(self.physical_connection_manager.clone())
                    .set_stream(stream)
                    .fetch();
                self.physical_connection_manager
                    .as_mut()
                    .unwrap()
                    .write()
                    .unwrap()
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
    pub fn new(ctx: Arc<Mutex<RuntimeContext>>) -> Core {
        let addr = format!("{}:{}", ctx.lock().unwrap().local_host, ctx.lock().unwrap().local_port);
        let breaker = Breaker::new(addr);

        let self_ref = Arc::new(Mutex::new(breaker));
        self_ref
            .lock()
            .as_mut()
            .unwrap()
            .init_managers(self_ref.clone());

        // default host
        let default_name = String::from("MQ_HOST");
        self_ref
            .lock()
            .unwrap()
            .host_manager
            .clone()
            .unwrap()
            .write()
            .unwrap()
            .add(default_name.clone(), VirtualHost::new(default_name));

        for h in &ctx.lock().unwrap().hosts {
            self_ref
                .lock()
                .unwrap()
                .host_manager
                .clone()
                .unwrap()
                .write()
                .unwrap()
                .add(h.clone(), VirtualHost::new(h.clone()));
        }

        Core { breaker: self_ref }
    }

    pub fn start(&mut self) {
        self.breaker.lock().unwrap().start_worker();
    }

    pub fn stop(&mut self) {
        self.breaker.lock().unwrap().stop_worker();
    }
}
