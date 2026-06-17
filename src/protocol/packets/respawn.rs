use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

pub struct RespawnPacket {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub respawn: bool,
}

impl RespawnPacket {
    pub fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z, respawn: false } }
}

impl DataPacket for RespawnPacket {
    const NETWORK_ID: u8 = ProtocolInfo::RESPAWN_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_vector3f(self.x, self.y, self.z);
        stream.put_bool(self.respawn);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let (x, y, z) = stream.get_vector3f().unwrap_or((0.0, 0.0, 0.0));
        let respawn = stream.get_bool().unwrap_or(false);
        Self { x, y, z, respawn }
    }

    fn name(&self) -> &'static str { "RespawnPacket" }
}
