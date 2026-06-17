use crate::utils::BinaryStream;
use crate::raknet::protocol::message_ids::MessageIdentifiers;

pub struct DisconnectionNotification;

impl DisconnectionNotification {
    pub const ID: u8 = MessageIdentifiers::ID_DISCONNECTION_NOTIFICATION;

    pub fn new() -> Self { Self }

    pub fn encode(&self) -> Vec<u8> {
        let mut stream = BinaryStream::new();
        stream.put_byte(Self::ID);
        stream.into_inner()
    }

    pub fn decode(buffer: Vec<u8>) -> Self {
        let mut stream = BinaryStream::from_bytes(buffer);
        let _ = stream.get_byte();
        Self
    }
}
