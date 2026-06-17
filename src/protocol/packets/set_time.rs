use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

pub struct SetTimePacket {
    pub time: i32,
}

impl SetTimePacket {
    pub fn new(time: i32) -> Self { Self { time } }
}

impl DataPacket for SetTimePacket {
    const NETWORK_ID: u8 = ProtocolInfo::SET_TIME_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_var_int(self.time);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let time = stream.get_var_int().unwrap_or(0);
        Self { time }
    }

    fn name(&self) -> &'static str { "SetTimePacket" }
}
