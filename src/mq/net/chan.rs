pub struct Channel {
    pub name: String,
    receiving: u32,
    received: u32,
    
    buffer: Vec<u8>
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

    pub fn set_receiving(&mut self, receiving: u32) {
        self.receiving = receiving;
    }

    pub fn write_buffer(&mut self, mut data: Vec<u8>) -> bool {
        self.buffer.append(&mut data);
        self.received += 512;

        self.received >= self.receiving
    }

    pub fn read_buffer(&mut self) -> Vec<u8> {
        self.receiving = 0;
        self.received = 0;

        self.buffer.drain(0..self.buffer.len()).collect()
    }
}