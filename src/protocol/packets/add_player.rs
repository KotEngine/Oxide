use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;
use crate::utils::UUID;

pub struct AddPlayerPacket {
    pub uuid: UUID,
    pub username: String,
    pub entity_runtime_id: i64,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub speed_x: f32,
    pub speed_y: f32,
    pub speed_z: f32,
    pub yaw: f32,
    pub head_yaw: Option<f32>,
    pub pitch: f32,
    pub item: crate::protocol::types::Item,
    pub metadata: crate::protocol::types::EntityMetadata,
}

impl AddPlayerPacket {
    pub fn new(uuid: UUID, username: String, entity_runtime_id: i64, x: f32, y: f32, z: f32) -> Self {
        Self {
            uuid, username, entity_runtime_id, x, y, z,
            speed_x: 0.0, speed_y: 0.0, speed_z: 0.0,
            yaw: 0.0, head_yaw: None, pitch: 0.0,
            item: crate::protocol::types::Item::air(),
            metadata: crate::protocol::types::EntityMetadata::new(),
        }
    }
}

impl DataPacket for AddPlayerPacket {
    const NETWORK_ID: u8 = ProtocolInfo::ADD_PLAYER_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_uuid(self.uuid);
        stream.put_string(&self.username);
        // Unique entity ID and runtime entity ID (both share the same ID in MCPE 1.1.5)
        stream.put_entity_id(self.entity_runtime_id);
        stream.put_entity_id(self.entity_runtime_id);
        stream.put_vector3f(self.x, self.y, self.z);
        stream.put_vector3f(self.speed_x, self.speed_y, self.speed_z);
        stream.put_lfloat(self.pitch);
        stream.put_lfloat(self.head_yaw.unwrap_or(self.yaw));
        stream.put_lfloat(self.yaw);
        self.item.put(stream);
        self.metadata.put(stream);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let uuid = stream.get_uuid().unwrap_or_else(|_| UUID::nil());
        let username = stream.get_string().unwrap_or_default();
        let _unique_id = stream.get_entity_id().unwrap_or(0);
        let entity_runtime_id = stream.get_entity_id().unwrap_or(0);
        let (x, y, z) = stream.get_vector3f().unwrap_or((0.0, 0.0, 0.0));
        let (speed_x, speed_y, speed_z) = stream.get_vector3f().unwrap_or((0.0, 0.0, 0.0));
        let pitch = stream.get_lfloat().unwrap_or(0.0);
        let head_yaw = stream.get_lfloat().ok();
        let yaw = stream.get_lfloat().unwrap_or(0.0);
        let mut item = crate::protocol::types::Item::air();
        item.get(&mut stream);
        let metadata = crate::protocol::types::EntityMetadata::get(&mut stream);
        Self {
            uuid, username, entity_runtime_id, x, y, z,
            speed_x, speed_y, speed_z, yaw, head_yaw, pitch,
            item, metadata,
        }
    }

    fn name(&self) -> &'static str { "AddPlayerPacket" }
}

pub struct RemoveEntityPacket {
    pub entity_runtime_id: i64,
}

impl RemoveEntityPacket {
    pub fn new(entity_runtime_id: i64) -> Self { Self { entity_runtime_id } }
}

impl DataPacket for RemoveEntityPacket {
    const NETWORK_ID: u8 = ProtocolInfo::REMOVE_ENTITY_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_entity_id(self.entity_runtime_id);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let entity_runtime_id = stream.get_entity_id().unwrap_or(0);
        Self { entity_runtime_id }
    }

    fn name(&self) -> &'static str { "RemoveEntityPacket" }
}
