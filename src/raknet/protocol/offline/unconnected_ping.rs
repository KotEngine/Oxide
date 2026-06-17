use crate::utils::BinaryStream;
use crate::raknet::protocol::{message_ids::MessageIdentifiers, packet::OfflineMessage};

pub struct UnconnectedPing {
    pub ping_id: i64,
    pub magic: [u8; 16],
}

impl UnconnectedPing {
    pub const ID: u8 = MessageIdentifiers::ID_UNCONNECTED_PING;

    pub fn new(ping_id: i64) -> Self {
        Self { ping_id, magic: [0; 16] }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut stream = BinaryStream::new();
        stream.put_byte(Self::ID);
        stream.put_long(self.ping_id);
        OfflineMessage::write_magic(&mut stream);
        stream.into_inner()
    }

    pub fn decode(buffer: Vec<u8>) -> Self {
        let mut stream = BinaryStream::from_bytes(buffer);
        let _ = stream.get_byte();
        let ping_id = stream.get_long().unwrap_or(0);
        let magic = OfflineMessage::read_magic(&mut stream);
        Self { ping_id, magic }
    }

    pub fn is_valid(&self) -> bool {
        OfflineMessage::is_valid_magic(&self.magic)
    }
}
