use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

#[derive(Debug)]
pub struct RequestChunkRadiusPacket {
    pub radius: i32,
}

impl DataPacket for RequestChunkRadiusPacket {
    const NETWORK_ID: u8 = ProtocolInfo::REQUEST_CHUNK_RADIUS_PACKET;

    fn encode_payload(&self, _stream: &mut BinaryStream) {}

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let radius = stream.get_var_int().unwrap_or(0);
        Self { radius }
    }

    fn name(&self) -> &'static str { "RequestChunkRadiusPacket" }
}
