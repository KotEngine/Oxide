use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

pub struct AvailableCommandsPacket {
    pub commands_data: Vec<u8>,
}

impl Default for AvailableCommandsPacket {
    fn default() -> Self { Self { commands_data: Vec::new() } }
}

impl DataPacket for AvailableCommandsPacket {
    const NETWORK_ID: u8 = ProtocolInfo::AVAILABLE_COMMANDS_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_string(std::str::from_utf8(&self.commands_data).unwrap_or(""));
    }

    fn decode(_buf: Vec<u8>) -> Self where Self: Sized {
        Self::default()
    }

    fn name(&self) -> &'static str { "AvailableCommandsPacket" }
}
