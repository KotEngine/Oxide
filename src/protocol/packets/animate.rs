use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

pub struct AnimatePacket {
    pub action: u32,
    pub entity_runtime_id: i64,
}

impl AnimatePacket {
    pub fn new(action: u32, entity_runtime_id: i64) -> Self { Self { action, entity_runtime_id } }
}

impl DataPacket for AnimatePacket {
    const NETWORK_ID: u8 = ProtocolInfo::ANIMATE_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_unsigned_var_int(self.action);
        stream.put_entity_id(self.entity_runtime_id);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let action = stream.get_unsigned_var_int().unwrap_or(0);
        let entity_runtime_id = stream.get_entity_id().unwrap_or(0);
        Self { action, entity_runtime_id }
    }

    fn name(&self) -> &'static str { "AnimatePacket" }
}
