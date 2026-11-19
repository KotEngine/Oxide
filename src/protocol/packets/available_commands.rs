use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

pub struct AvailableCommandsPacket {
    /// JSON string containing command definitions
    pub commands: String,
    /// Second JSON field for extra command metadata
    pub unknown: String,
}

impl Default for AvailableCommandsPacket {
    fn default() -> Self {
        Self { commands: "{}".to_string(), unknown: "{}".to_string() }
    }
}

impl DataPacket for AvailableCommandsPacket {
    const NETWORK_ID: u8 = ProtocolInfo::AVAILABLE_COMMANDS_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_string(&self.commands);
        stream.put_string(&self.unknown);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        Self {
            commands: s.get_string().unwrap_or_default(),
            unknown: s.get_string().unwrap_or_default(),
        }
    }

    fn name(&self) -> &'static str { "AvailableCommandsPacket" }
}
