use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

pub const ACTION_SWING_ARM: i32 = 1;
pub const ACTION_STOP_SLEEP: i32 = 3;
pub const ACTION_CRITICAL_HIT: i32 = 4;
pub const ACTION_ROW_RIGHT: i32 = 128;
pub const ACTION_ROW_LEFT: i32 = 129;

#[derive(Debug, Clone)]
pub struct AnimatePacket {
    pub action: i32,
    pub entity_runtime_id: i64,
    pub rowing_time: f32,
}

impl AnimatePacket {
    pub fn new(action: i32, entity_runtime_id: i64) -> Self {
        Self { action, entity_runtime_id, rowing_time: 0.0 }
    }
}

impl DataPacket for AnimatePacket {
    const NETWORK_ID: u8 = ProtocolInfo::ANIMATE_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_var_int(self.action);
        stream.put_entity_id(self.entity_runtime_id);
        if self.action & 0x80 != 0 {
            stream.put_lfloat(self.rowing_time);
        }
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let action = stream.get_var_int().unwrap_or(0);
        let entity_runtime_id = stream.get_entity_id().unwrap_or(0);
        let rowing_time = if action & 0x80 != 0 {
            stream.get_lfloat().unwrap_or(0.0)
        } else {
            0.0
        };
        Self { action, entity_runtime_id, rowing_time }
    }

    fn name(&self) -> &'static str { "AnimatePacket" }
}
