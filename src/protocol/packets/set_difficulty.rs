use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

pub struct SetDifficultyPacket {
    pub difficulty: u32,
}

impl SetDifficultyPacket {
    pub fn new(difficulty: u32) -> Self { Self { difficulty } }
}

impl DataPacket for SetDifficultyPacket {
    const NETWORK_ID: u8 = ProtocolInfo::SET_DIFFICULTY_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_unsigned_var_int(self.difficulty);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let difficulty = stream.get_unsigned_var_int().unwrap_or(0);
        Self { difficulty }
    }

    fn name(&self) -> &'static str { "SetDifficultyPacket" }
}
