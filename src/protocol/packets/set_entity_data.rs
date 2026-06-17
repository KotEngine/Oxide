use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;
use crate::protocol::types::EntityMetadata;

pub struct SetEntityDataPacket {
    pub entity_runtime_id: i64,
    pub metadata: EntityMetadata,
}

impl SetEntityDataPacket {
    pub fn new(entity_runtime_id: i64, metadata: EntityMetadata) -> Self {
        Self { entity_runtime_id, metadata }
    }
}

impl DataPacket for SetEntityDataPacket {
    const NETWORK_ID: u8 = ProtocolInfo::SET_ENTITY_DATA_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_entity_id(self.entity_runtime_id);
        self.metadata.put(stream);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let entity_runtime_id = stream.get_entity_id().unwrap_or(0);
        let metadata = EntityMetadata::get(&mut stream);
        Self { entity_runtime_id, metadata }
    }

    fn name(&self) -> &'static str { "SetEntityDataPacket" }
}
