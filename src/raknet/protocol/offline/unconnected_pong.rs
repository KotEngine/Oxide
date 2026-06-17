use crate::utils::BinaryStream;
use crate::raknet::protocol::{message_ids::MessageIdentifiers, packet::OfflineMessage};

pub struct UnconnectedPong {
    pub ping_id: i64,
    pub server_id: i64,
    pub server_name: String,
    pub magic: [u8; 16],
}

impl UnconnectedPong {
    pub const ID: u8 = MessageIdentifiers::ID_UNCONNECTED_PONG;

    pub fn new(ping_id: i64, server_id: i64, server_name: String) -> Self {
        Self { ping_id, server_id, server_name, magic: [0; 16] }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut stream = BinaryStream::new();
        stream.put_byte(Self::ID);
        stream.put_long(self.ping_id);
        stream.put_long(self.server_id);
        OfflineMessage::write_magic(&mut stream);
        stream.put_string(&self.server_name);
        stream.into_inner()
    }

    pub fn decode(buffer: Vec<u8>) -> Self {
        let mut stream = BinaryStream::from_bytes(buffer);
        let _ = stream.get_byte();
        let ping_id = stream.get_long().unwrap_or(0);
        let server_id = stream.get_long().unwrap_or(0);
        let magic = OfflineMessage::read_magic(&mut stream);
        let server_name = stream.get_string().unwrap_or_default();
        Self { ping_id, server_id, server_name, magic }
    }
}
