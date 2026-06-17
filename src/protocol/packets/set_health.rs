use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

pub struct SetHealthPacket {
    pub health: u32,
}

impl SetHealthPacket {
    pub fn new(health: u32) -> Self { Self { health } }
}

impl DataPacket for SetHealthPacket {
    const NETWORK_ID: u8 = ProtocolInfo::SET_HEALTH_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_unsigned_var_int(self.health);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let health = stream.get_unsigned_var_int().unwrap_or(0);
        Self { health }
    }

    fn name(&self) -> &'static str { "SetHealthPacket" }
}
