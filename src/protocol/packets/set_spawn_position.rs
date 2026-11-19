use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

pub const TYPE_PLAYER_SPAWN: i32 = 0;
pub const TYPE_WORLD_SPAWN: i32 = 1;

pub struct SetSpawnPositionPacket {
    pub spawn_type: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub spawn_forced: bool,
}

impl SetSpawnPositionPacket {
    pub fn new_world(x: i32, y: i32, z: i32) -> Self {
        Self { spawn_type: TYPE_WORLD_SPAWN, x, y, z, spawn_forced: false }
    }

    pub fn new_player(x: i32, y: i32, z: i32) -> Self {
        Self { spawn_type: TYPE_PLAYER_SPAWN, x, y, z, spawn_forced: false }
    }
}

impl DataPacket for SetSpawnPositionPacket {
    const NETWORK_ID: u8 = ProtocolInfo::SET_SPAWN_POSITION_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_var_int(self.spawn_type);
        stream.put_block_coords(self.x, self.y as u32, self.z);
        stream.put_bool(self.spawn_forced);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let spawn_type = stream.get_var_int().unwrap_or(0);
        let (x, y, z) = stream.get_block_coords().unwrap_or((0, 0, 0));
        let spawn_forced = stream.get_bool().unwrap_or(false);
        Self { spawn_type, x, y: y as i32, z, spawn_forced }
    }

    fn name(&self) -> &'static str { "SetSpawnPositionPacket" }
}
