use std::error::Error;
use std::sync::{Arc, Condvar, Mutex};
use std::{io, thread};
use std::any::Any;
use std::io::Write;
use std::net::ToSocketAddrs;
use std::thread::JoinHandle;
use crate::mq::breaker;
use crate::mq::common::context::RuntimeContext;
use crate::test::test::test;

// always remember that the last value of RoutingKey is the name of the Queue.
// note that the \0 at the end of the strings must be trimmed using trim_end_matches() !!
pub mod test;
pub mod mq;

fn main() -> Result<(), Box<dyn Error>> {
    let ctx = Arc::new(Mutex::new(construct_ctx()?));
    let rt_handle = set_runtime(ctx);
    rt_handle.join().unwrap();
    Ok(())
}

fn construct_ctx() -> Result<RuntimeContext, Box<dyn Error>> {
    let mut host = String::new();
    io::stdin().read_line(&mut host)?;
    host = host.trim().to_string();

    let mut port = String::new();
    io::stdin().read_line(&mut port)?;
    let port: u16 = port.trim().parse()?;

    let concatenated = format!("{}:{}", host, port);
    println!("Starting server on {}", concatenated);

    Ok(RuntimeContext::new(host, port))
}

fn set_runtime(ctx: Arc<Mutex<RuntimeContext>>) -> JoinHandle<()> {
    thread::scope(|scope| {
    });
    thread::spawn(move || {
        run(ctx);
    })
}

fn run(ctx: Arc<Mutex<RuntimeContext>>) {
    let host = ctx.lock().unwrap().local_host.clone();
    let port = ctx.lock().unwrap().local_port;

    let concatenated = format!("{}:{}", host, port);
    let mut core = breaker::core::Core::new(concatenated);
    core.start();
}
