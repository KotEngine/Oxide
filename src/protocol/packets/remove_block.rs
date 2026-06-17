use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

pub struct RemoveBlockPacket {
    pub x: i32,
    pub y: u32,
    pub z: i32,
}

impl RemoveBlockPacket {
    pub fn new(x: i32, y: u32, z: i32) -> Self { Self { x, y, z } }
}

impl DataPacket for RemoveBlockPacket {
    const NETWORK_ID: u8 = ProtocolInfo::REMOVE_BLOCK_PACKET;

    fn encode_payload(&self, _stream: &mut BinaryStream) {}

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let (x, y, z) = stream.get_block_coords().unwrap_or((0, 0, 0));
        Self { x, y, z }
    }

    fn name(&self) -> &'static str { "RemoveBlockPacket" }
}
