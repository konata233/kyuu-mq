pub struct Channel {
    pub name: String,
    receiving: u64,
    received: u64,

    buffer: Vec<u8>,
}

impl Channel {
    pub fn new(name: String) -> Channel {
        Channel {
            name,
            receiving: 0,
            received: 0,
            buffer: vec![],
        }
    }

    pub fn set_receiving(&mut self, receiving: u64) -> &mut Self {
        self.receiving = receiving;
        self
    }

    pub fn write_buffer(&mut self, mut data: Vec<u8>, size: u64) -> bool {
        self.buffer.append(&mut data);
        self.received += size;

        self.received >= self.receiving
    }

    pub fn read_buffer(&mut self) -> Vec<u8> {
        self.receiving = 0;
        self.received = 0;

        self.buffer.drain(0..self.buffer.len()).collect()
    }

    pub fn peek_buffer(&self) -> Vec<u8> {
        self.buffer.clone()
    }
}
