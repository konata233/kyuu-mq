use crate::mq::breaker::core::Core;

pub fn test() {
    test_host();
    return;
    let mut core = Core::new("127.0.0.1:11451");
    core.start();
}

pub fn test_host() {
    /*let mut host = VirtualHost::new("test".to_string());
    let name = host.name.clone();
    host.add_exchange("test_exc".to_string(), RoutingKey::Direct(["\0", "", "", ""].map(|x| x.to_string())));
    host.add_exchange("test_exc2".to_string(), RoutingKey::Direct(["test_exc", "!", "", ""].map(|x| x.to_string())));
    host.add_queue("test".to_string(), RoutingKey::Direct(["test_exc", "\0", "", ""].map(|x| x.to_string())));
    host.add_queue("test2".to_string(), RoutingKey::Direct(["test_exc", "test_exc2", "!", ""].map(|x| x.to_string())));

    let mut s = host.get_queue(&"test".to_string(), RoutingKey::Direct(["test_exc", "!", "", ""].map(|x| x.to_string()))).unwrap();
    s.lock().unwrap().push_back(QueueObject::new(&name, "hello".as_bytes().to_vec()));
    s.lock().unwrap().push_back(QueueObject::new(&name, "world".as_bytes().to_vec()));

    let s2 = host.get_queue(&"test2".to_string(), RoutingKey::Direct(["test_exc", "test_exc2", "", ""].map(|x| x.to_string()))).unwrap();
    s2.lock()
        .unwrap()
        .push_back(QueueObject::new(&name, "hello2".as_bytes().to_vec()));

    let raw_data = RawData {
        raw: Raw::Message(RawMessage::Push("test!!".to_string().into())),
        channel: "".to_string(),
        virtual_host: "test".to_string(),
        routing_key: RoutingKey::Direct(["test_exc", "test_exc2", "!", "test2"].map(|x| x.to_string())),
    };
    let result = host.process_incoming(raw_data);

    dbg!(String::from_utf8(s.lock().unwrap().pop_front().unwrap().content).unwrap());
    dbg!(String::from_utf8(s.lock().unwrap().pop_front().unwrap().content).unwrap());
    dbg!(String::from_utf8(s2.lock().unwrap().pop_front().unwrap().content).unwrap());
    dbg!(String::from_utf8(s2.lock().unwrap().pop_front().unwrap().content).unwrap());*/
}
