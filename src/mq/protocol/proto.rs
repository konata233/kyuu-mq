use crate::mq::protocol::protobase::{Deserialize, Serialize};
use crate::mq::host::vhost::VirtualHost;

pub struct DataHead {
    pub virtual_host: [u8; 32],

    pub channel: [u8; 32],

    pub version: [u8; 4],
    pub routing_mod: [u8; 4],
    pub command: [u8; 24],

    pub route0: [u8; 32],

    pub route1: [u8; 32],

    pub route2: [u8; 32],

    pub route3: [u8; 32],

    pub slice_count: u32,
    pub slice_size: u32, // 512
    pub count: u32,
    pub msg_sign: u16,
    pub ack: u16,
    pub reserved: [u8; 16]
}

impl DataHead {
    pub fn new(virtual_host: VirtualHost,
               channel: [u8; 32],
               routing_mod: [u8; 4],
               command: [u8; 24],
               route: [u8; 128],
               slice_count: u32,
               slice_size: u32,
               count: u32,
               msg_sign: u16) -> DataHead {
        DataHead {
            virtual_host: <[u8; 32]>::try_from(virtual_host.name.as_bytes()).unwrap(),
            channel,
            version: [1u8, 0u8, 0u8, 0u8],
            routing_mod,
            command,
            route0: <[u8; 32]>::try_from(&route[0..32]).unwrap(),
            route1: <[u8; 32]>::try_from(&route[32..64]).unwrap(),
            route2: <[u8; 32]>::try_from(&route[64..96]).unwrap(),
            route3: <[u8; 32]>::try_from(&route[96..128]).unwrap(),
            slice_count,
            slice_size,
            count,
            msg_sign,
            ack: 0,
            reserved: [0u8; 16]
        }
    }
}

impl Serialize<256> for DataHead {
    fn serialize(&self) -> [u8; 256] {
        let mut serialized = vec![];
        serialized.append(&mut self.virtual_host.to_vec());
        serialized.append(&mut self.channel.to_vec());
        serialized.append(&mut self.version.to_vec());
        serialized.append(&mut self.routing_mod.to_vec());
        serialized.append(&mut self.command.to_vec());
        serialized.append(&mut self.route0.to_vec());
        serialized.append(&mut self.route1.to_vec());
        serialized.append(&mut self.route2.to_vec());
        serialized.append(&mut self.route3.to_vec());
        serialized.append(&mut self.slice_count.to_le_bytes().to_vec());
        serialized.append(&mut self.slice_size.to_le_bytes().to_vec());
        serialized.append(&mut self.count.to_le_bytes().to_vec());
        serialized.append(&mut self.msg_sign.to_le_bytes().to_vec());
        serialized.append(&mut self.ack.to_le_bytes().to_vec());
        serialized.append(&mut self.reserved.to_vec());
        <[u8; 256]>::try_from(serialized).unwrap()
    }
}

impl Deserialize<256> for DataHead {
    type T = DataHead;

    fn deserialize(data: [u8; 256]) -> Self::T {
        let mut deserialized = data.to_vec();
        let virtual_host_sha256 = <[u8; 32]>::try_from(deserialized.drain(0..32).collect::<Vec<_>>()).unwrap();

        let channel_sha256 = <[u8; 32]>::try_from(deserialized.drain(0..32).collect::<Vec<_>>()).unwrap();
        let version = <[u8; 4]>::try_from(deserialized.drain(0..4).collect::<Vec<_>>()).unwrap();
        let routing_mod = <[u8; 4]>::try_from(deserialized.drain(0..4).collect::<Vec<_>>()).unwrap();
        let command = <[u8; 24]>::try_from(deserialized.drain(0..24).collect::<Vec<_>>()).unwrap();

        let route0 = <[u8; 32]>::try_from(deserialized.drain(0..32).collect::<Vec<_>>()).unwrap();

        let route1 = <[u8; 32]>::try_from(deserialized.drain(0..32).collect::<Vec<_>>()).unwrap();

        let route2 = <[u8; 32]>::try_from(deserialized.drain(0..32).collect::<Vec<_>>()).unwrap();

        let route3 = <[u8; 32]>::try_from(deserialized.drain(0..32).collect::<Vec<_>>()).unwrap();

        let slice_count = <u32>::from_le_bytes(<[u8; 4]>::try_from(deserialized.drain(0..4).collect::<Vec<_>>()).unwrap());
        let slice_size = <u32>::from_le_bytes(<[u8; 4]>::try_from(deserialized.drain(0..4).collect::<Vec<_>>()).unwrap());
        let count = <u32>::from_le_bytes(<[u8; 4]>::try_from(deserialized.drain(0..4).collect::<Vec<_>>()).unwrap());
        let msg_sign = <u16>::from_le_bytes(<[u8; 2]>::try_from(deserialized.drain(0..2).collect::<Vec<_>>()).unwrap());
        let ack = <u16>::from_le_bytes(<[u8; 2]>::try_from(deserialized.drain(0..2).collect::<Vec<_>>()).unwrap());
        let reserved = <[u8; 16]>::try_from(deserialized.drain(0..16).collect::<Vec<_>>()).unwrap();

        DataHead {
            virtual_host: virtual_host_sha256,
            channel: channel_sha256,
            version,
            routing_mod,
            command,
            route0,
            route1,
            route2,
            route3,
            slice_count,
            slice_size,
            count,
            msg_sign,
            ack,
            reserved
        }
    }
}

pub struct Acknowledge {
    virtual_host_sha256: [u8; 32],

    channel_sha256: [u8; 32],

    version: [u8; 4],
    msg_sign: u16,
    ack: u16,
    reserved: [u8; 24]
}

impl Acknowledge {
    pub fn new(virtual_host: VirtualHost, sender: [u8; 32], ack: u16, msg_sign: u16) -> Acknowledge {
        Acknowledge {
            virtual_host_sha256: <[u8; 32]>::try_from(virtual_host.name[0..32].as_bytes()).unwrap(),
            channel_sha256: sender,
            version: [1u8, 0u8, 0u8, 0u8],
            msg_sign,
            ack,
            reserved: [0u8; 24]
        }
    }
}

impl Serialize<96> for Acknowledge {
    fn serialize(&self) -> [u8; 96] {
        let mut serialized = vec![];
        serialized.append(&mut self.virtual_host_sha256.to_vec());
        serialized.append(&mut self.channel_sha256.to_vec());
        serialized.append(&mut self.version.to_vec());
        serialized.append(&mut self.msg_sign.to_le_bytes().to_vec());
        serialized.append(&mut self.ack.to_le_bytes().to_vec());
        serialized.append(&mut self.reserved.to_vec());
        <[u8; 96]>::try_from(serialized).unwrap()
    }
}

impl Deserialize<96> for Acknowledge {
    type T = Acknowledge;

    fn deserialize(data: [u8; 96]) -> Self::T {
        let mut deserialized = data.to_vec();
        let virtual_host_sha256 = <[u8; 32]>::try_from(deserialized.drain(0..32).collect::<Vec<_>>()).unwrap();

        let channel_sha256 = <[u8; 32]>::try_from(deserialized.drain(0..32).collect::<Vec<_>>()).unwrap();
        let version = <[u8; 4]>::try_from(deserialized.drain(0..4).collect::<Vec<_>>()).unwrap();
        let msg_sign = <u16>::from_le_bytes(<[u8; 2]>::try_from(deserialized.drain(0..2).collect::<Vec<_>>()).unwrap());
        let ack = <u16>::from_le_bytes(<[u8; 2]>::try_from(deserialized.drain(0..2).collect::<Vec<_>>()).unwrap());
        let reserved = <[u8; 24]>::try_from(deserialized.drain(0..24).collect::<Vec<_>>()).unwrap();

        Acknowledge {
            virtual_host_sha256,
            channel_sha256,
            version,
            msg_sign,
            ack,
            reserved
        }
    }
}

pub struct Data {
    virtual_host: VirtualHost,
    channel: [u8; 32],
    data: [u8; 448]
}

impl Data {
    pub fn new(virtual_host: VirtualHost, channel: [u8; 32], data: [u8; 448]) -> Data {
        Data {
            virtual_host,
            channel,
            data
        }
    }
}

impl Serialize<512> for Data {
    fn serialize(&self) -> [u8; 512] {
        let mut serialized = vec![];
        serialized.append(&mut self.virtual_host.name.as_bytes().to_vec());
        serialized.append(&mut self.channel.to_vec());
        serialized.append(&mut self.data.to_vec());
        <[u8; 512]>::try_from(serialized).unwrap()
    }
}

impl Deserialize<512> for Data {
    type T = Data;

    fn deserialize(data: [u8; 512]) -> Self::T {
        let mut deserialized = data.to_vec();
        let virtual_host = VirtualHost::new(
            String::from_utf8(
                deserialized.drain(0..32)
                    .collect::<Vec<_>>()).unwrap()
        );

        let channel = <[u8; 32]>::try_from(deserialized.drain(0..32).collect::<Vec<_>>()).unwrap();
        let data = <[u8; 448]>::try_from(deserialized.drain(0..448).collect::<Vec<_>>()).unwrap();

        Data {
            virtual_host,
            channel,
            data
        }
    }
}