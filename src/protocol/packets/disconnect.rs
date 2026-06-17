use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

pub struct DisconnectPacket {
    pub message: String,
}

impl DisconnectPacket {
    pub fn new(message: impl Into<String>) -> Self { Self { message: message.into() } }
}

impl DataPacket for DisconnectPacket {
    const NETWORK_ID: u8 = ProtocolInfo::DISCONNECT_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_bool(false);
        stream.put_string(&self.message);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let _ = stream.get_bool();
        let message = stream.get_string().unwrap_or_default();
        Self { message }
    }

    fn name(&self) -> &'static str { "DisconnectPacket" }
    fn can_be_sent_before_login(&self) -> bool { true }
}
