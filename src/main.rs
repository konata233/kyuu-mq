use crate::mq::breaker;
use crate::mq::common::context::RuntimeContext;
use inio::io::reader;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{io, thread};

// always remember that the last value of RoutingKey is the name of the Queue.
// note that the \0 at the end of the strings must be trimmed using trim_end_matches() !!
pub mod mq;
pub mod test;

fn main() -> Result<(), Box<dyn Error>> {
    let ctx = ini_loader()?; // Arc::new(Mutex::new(construct_ctx()?));
    let rt_handle = set_runtime(Arc::from(Mutex::from(ctx)));
    rt_handle.join().unwrap();
    Ok(())
}

fn ini_loader() -> Result<RuntimeContext, Box<dyn Error>> {
    let conf = reader::IniReader::new("config.ini".to_string()).read()?;
    let host: String;
    let port: u16;
    let virtual_host_counts: usize;

    if let Some(sec) = conf.get_section("Net") {
        host = if let Some(key) = sec.get_key("Host") {
            key.value.clone()
        } else {
            "127.0.0.1".to_string()
        };

        port = if let Some(key) = sec.get_key("Port") {
            key.value.parse()?
        } else {
            8080
        };

        virtual_host_counts = if let Some(key) = sec.get_key("VirtualHostCount") {
            key.value.parse()?
        } else {
            1
        };
    } else {
        panic!("[mq] config file not found");
    };

    let mut ctx = RuntimeContext::new(host, port);

    for i in 0..virtual_host_counts {
        if let Some(sec) = conf.get_section(&format!("Host{}", i)) {
            if let Some(key) = sec.get_key("name") {
                ctx.hosts.push(key.value.clone());
            } else {
                panic!("[mq] config key \"name\" of host {} not found", i);
            }
        } else {
            panic!("[mq] config section of host {} not found", i.to_string());
        }
    }

    Ok(ctx)
}

fn construct_ctx() -> Result<RuntimeContext, Box<dyn Error>> {
    let mut host = String::new();
    print!("[mq] Enter host: ");
    io::stdin().read_line(&mut host)?;
    host = host.trim().to_string();

    let mut port = String::new();
    print!("[mq] Enter port: ");
    io::stdin().read_line(&mut port)?;
    let port: u16 = port.trim().parse()?;

    let concatenated = format!("{}:{}", host, port);
    println!("[mq] starting server on {}", concatenated);

    Ok(RuntimeContext::new(host, port))
}

fn set_runtime(ctx: Arc<Mutex<RuntimeContext>>) -> JoinHandle<()> {
    thread::spawn(move || {
        run(ctx);
    })
}

fn run(ctx: Arc<Mutex<RuntimeContext>>) {
    let mut core = breaker::core::Core::new(ctx);
    core.start();
}
