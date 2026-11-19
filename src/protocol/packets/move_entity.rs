use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

pub struct MoveEntityPacket {
    pub entity_runtime_id: i64,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub head_yaw: f32,
    pub on_ground: bool,
    pub teleported: bool,
}

impl MoveEntityPacket {
    pub fn new(entity_runtime_id: i64, x: f32, y: f32, z: f32, yaw: f32, pitch: f32) -> Self {
        Self {
            entity_runtime_id, x, y, z, yaw, pitch, head_yaw: yaw,
            on_ground: false, teleported: false,
        }
    }
}

impl DataPacket for MoveEntityPacket {
    const NETWORK_ID: u8 = ProtocolInfo::MOVE_ENTITY_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_entity_id(self.entity_runtime_id);
        stream.put_vector3f(self.x, self.y, self.z);
        // Field ordering: pitch, head_yaw, yaw
        stream.put_byte_rotation(self.pitch);
        stream.put_byte_rotation(self.head_yaw);
        stream.put_byte_rotation(self.yaw);
        stream.put_bool(self.on_ground);
        stream.put_bool(self.teleported);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let entity_runtime_id = stream.get_entity_id().unwrap_or(0);
        let (x, y, z) = stream.get_vector3f().unwrap_or((0.0, 0.0, 0.0));
        let pitch = stream.get_byte_rotation().unwrap_or(0.0);
        let head_yaw = stream.get_byte_rotation().unwrap_or(0.0);
        let yaw = stream.get_byte_rotation().unwrap_or(0.0);
        let on_ground = stream.get_bool().unwrap_or(false);
        let teleported = stream.get_bool().unwrap_or(false);
        Self { entity_runtime_id, x, y, z, yaw, pitch, head_yaw, on_ground, teleported }
    }

    fn name(&self) -> &'static str { "MoveEntityPacket" }
}
