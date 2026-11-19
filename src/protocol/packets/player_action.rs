use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

pub const ACTION_START_BREAK: i32 = 0;
pub const ACTION_ABORT_BREAK: i32 = 1;
pub const ACTION_STOP_BREAK: i32 = 2;
pub const ACTION_RELEASE_ITEM: i32 = 5;
pub const ACTION_STOP_SLEEPING: i32 = 6;
pub const ACTION_SPAWN_SAME_DIMENSION: i32 = 7;
pub const ACTION_JUMP: i32 = 8;
pub const ACTION_START_SPRINT: i32 = 9;
pub const ACTION_STOP_SPRINT: i32 = 10;
pub const ACTION_START_SNEAK: i32 = 11;
pub const ACTION_STOP_SNEAK: i32 = 12;
pub const ACTION_SPAWN_OVERWORLD: i32 = 13;
pub const ACTION_SPAWN_NETHER: i32 = 14;
pub const ACTION_START_GLIDE: i32 = 15;
pub const ACTION_STOP_GLIDE: i32 = 16;
pub const ACTION_BUILD_DENIED: i32 = 17;
pub const ACTION_CONTINUE_BREAK: i32 = 18;

#[derive(Debug)]
pub struct PlayerActionPacket {
    pub entity_runtime_id: i64,
    pub action: i32,
    pub x: i32,
    pub y: u32,
    pub z: i32,
    pub face: i32,
}

impl PlayerActionPacket {
    pub fn new(entity_runtime_id: i64, action: i32, x: i32, y: u32, z: i32, face: i32) -> Self {
        Self { entity_runtime_id, action, x, y, z, face }
    }
}

impl DataPacket for PlayerActionPacket {
    const NETWORK_ID: u8 = ProtocolInfo::PLAYER_ACTION_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_entity_id(self.entity_runtime_id);
        stream.put_var_int(self.action);
        stream.put_block_coords(self.x, self.y, self.z);
        stream.put_var_int(self.face);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let entity_runtime_id = stream.get_entity_id().unwrap_or(0);
        let action = stream.get_var_int().unwrap_or(0);
        let (x, y, z) = stream.get_block_coords().unwrap_or((0, 0, 0));
        let face = stream.get_var_int().unwrap_or(0);
        Self { entity_runtime_id, action, x, y, z, face }
    }

    fn name(&self) -> &'static str { "PlayerActionPacket" }
}
