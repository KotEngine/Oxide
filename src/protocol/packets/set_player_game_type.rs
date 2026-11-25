use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

#[derive(Debug, Clone)]
pub struct SetPlayerGameTypePacket {
    pub gamemode: i32,
}

impl SetPlayerGameTypePacket {
    pub fn new(gamemode: i32) -> Self { Self { gamemode } }
}

impl DataPacket for SetPlayerGameTypePacket {
    const NETWORK_ID: u8 = ProtocolInfo::SET_PLAYER_GAME_TYPE_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_var_int(self.gamemode);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let gamemode = stream.get_var_int().unwrap_or(0);
        Self { gamemode }
    }

    fn name(&self) -> &'static str { "SetPlayerGameTypePacket" }
}
