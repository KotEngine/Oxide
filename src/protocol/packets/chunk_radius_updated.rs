use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

pub struct ChunkRadiusUpdatedPacket {
    pub radius: i32,
}

impl ChunkRadiusUpdatedPacket {
    pub fn new(radius: i32) -> Self { Self { radius } }
}

impl DataPacket for ChunkRadiusUpdatedPacket {
    const NETWORK_ID: u8 = ProtocolInfo::CHUNK_RADIUS_UPDATED_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_var_int(self.radius);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let radius = stream.get_var_int().unwrap_or(0);
        Self { radius }
    }

    fn name(&self) -> &'static str { "ChunkRadiusUpdatedPacket" }
}
