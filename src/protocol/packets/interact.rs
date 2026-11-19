use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

pub const ACTION_RIGHT_CLICK: u8 = 1;
pub const ACTION_LEFT_CLICK: u8 = 2;
pub const ACTION_LEAVE_VEHICLE: u8 = 3;
pub const ACTION_MOUSEOVER: u8 = 4;
pub const ACTION_OPEN_INVENTORY: u8 = 6;

#[derive(Debug)]
pub struct InteractPacket {
    pub action: u8,
    pub target: i64,
    pub entity_runtime_id: i64,
}

impl DataPacket for InteractPacket {
    const NETWORK_ID: u8 = ProtocolInfo::INTERACT_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_byte(self.action);
        stream.put_entity_id(self.target);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let action = stream.get_byte().unwrap_or(0);
        let target = stream.get_entity_id().unwrap_or(0);
        Self { action, target, entity_runtime_id: 0 }
    }

    fn name(&self) -> &'static str { "InteractPacket" }
}
