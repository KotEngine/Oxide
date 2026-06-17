use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

pub const LOGIN_SUCCESS: i32 = 0;
pub const LOGIN_FAILED_CLIENT: i32 = 1;
pub const LOGIN_FAILED_SERVER: i32 = 2;
pub const PLAYER_SPAWN: i32 = 3;
pub const LOGIN_FAILED_INVALID_TENANT: i32 = 4;
pub const LOGIN_FAILED_VANILLA_EDU: i32 = 5;
pub const LOGIN_FAILED_EDU_VANILLA: i32 = 6;

pub struct PlayStatusPacket {
    pub status: i32,
}

impl PlayStatusPacket {
    pub fn new(status: i32) -> Self { Self { status } }
}

impl DataPacket for PlayStatusPacket {
    const NETWORK_ID: u8 = ProtocolInfo::PLAY_STATUS_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_int(self.status);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let status = stream.get_int().unwrap_or(0);
        Self { status }
    }

    fn name(&self) -> &'static str { "PlayStatusPacket" }
    fn can_be_sent_before_login(&self) -> bool { true }
}
