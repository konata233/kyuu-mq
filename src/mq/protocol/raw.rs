pub enum RawData {
    Message(RawMessage),
    Command(Vec<u8>),
}

pub enum RawMessage {
    Push(Vec<u8>),
    Fetch,
}