use crate::utils::BinaryStream;
use crate::raknet::protocol::{message_ids::MessageIdentifiers, packet::OfflineMessage};

pub struct IncompatibleProtocolVersion {
    pub protocol: u8,
    pub server_id: i64,
    pub magic: [u8; 16],
}

impl IncompatibleProtocolVersion {
    pub const ID: u8 = MessageIdentifiers::ID_INCOMPATIBLE_PROTOCOL_VERSION;

    pub fn new(protocol: u8, server_id: i64) -> Self {
        Self { protocol, server_id, magic: [0; 16] }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut stream = BinaryStream::new();
        stream.put_byte(Self::ID);
        stream.put_byte(self.protocol);
        OfflineMessage::write_magic(&mut stream);
        stream.put_long(self.server_id);
        stream.into_inner()
    }

    pub fn decode(buffer: Vec<u8>) -> Self {
        let mut stream = BinaryStream::from_bytes(buffer);
        let _ = stream.get_byte();
        let protocol = stream.get_byte().unwrap_or(0);
        let magic = OfflineMessage::read_magic(&mut stream);
        let server_id = stream.get_long().unwrap_or(0);
        Self { protocol, server_id, magic }
    }
}
