use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

pub const MODE_NORMAL: u8 = 0;
pub const MODE_RESET: u8 = 1;
pub const MODE_TELEPORT: u8 = 2;
pub const MODE_PITCH: u8 = 3;

#[derive(Debug)]
pub struct MovePlayerPacket {
    pub entity_runtime_id: i64,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
    pub head_yaw: f32,
    pub pitch: f32,
    pub mode: u8,
    pub on_ground: bool,
    pub entity_runtime_id_2: i64,
    pub teleport_cause: i32,
    pub teleport_item: i32,
}

impl MovePlayerPacket {
    pub fn new(entity_runtime_id: i64, x: f32, y: f32, z: f32, yaw: f32, pitch: f32) -> Self {
        Self {
            entity_runtime_id,
            x, y, z,
            yaw,
            head_yaw: yaw,
            pitch,
            mode: MODE_NORMAL,
            on_ground: false,
            entity_runtime_id_2: 0,
            teleport_cause: 0,
            teleport_item: 0,
        }
    }
}

impl DataPacket for MovePlayerPacket {
    const NETWORK_ID: u8 = ProtocolInfo::MOVE_PLAYER_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_entity_id(self.entity_runtime_id);
        stream.put_vector3f(self.x, self.y, self.z);
        stream.put_lfloat(self.pitch);
        stream.put_lfloat(self.yaw);
        stream.put_lfloat(self.head_yaw);
        stream.put_byte(self.mode);
        stream.put_bool(self.on_ground);
        stream.put_entity_id(self.entity_runtime_id_2);
        if self.mode == MODE_TELEPORT {
            stream.put_lint(self.teleport_cause);
            stream.put_lint(self.teleport_item);
        }
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let entity_runtime_id = stream.get_entity_id().unwrap_or(0);
        let (x, y, z) = stream.get_vector3f().unwrap_or((0.0, 0.0, 0.0));
        let pitch = stream.get_lfloat().unwrap_or(0.0);
        let yaw = stream.get_lfloat().unwrap_or(0.0);
        let head_yaw = stream.get_lfloat().unwrap_or(0.0);
        let mode = stream.get_byte().unwrap_or(0);
        let on_ground = stream.get_bool().unwrap_or(false);
        let entity_runtime_id_2 = stream.get_entity_id().unwrap_or(0);
        let (teleport_cause, teleport_item) = if mode == MODE_TELEPORT {
            (stream.get_lint().unwrap_or(0), stream.get_lint().unwrap_or(0))
        } else {
            (0, 0)
        };
        Self {
            entity_runtime_id, x, y, z, yaw, head_yaw, pitch, mode, on_ground,
            entity_runtime_id_2, teleport_cause, teleport_item,
        }
    }

    fn name(&self) -> &'static str { "MovePlayerPacket" }
}
