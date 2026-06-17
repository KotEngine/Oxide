use crate::utils::BinaryStream;
use crate::raknet::protocol::{message_ids::MessageIdentifiers, packet::OfflineMessage};

pub struct OpenConnectionReply1 {
    pub server_id: i64,
    pub server_security: bool,
    pub mtu_size: u16,
    pub magic: [u8; 16],
}

impl OpenConnectionReply1 {
    pub const ID: u8 = MessageIdentifiers::ID_OPEN_CONNECTION_REPLY_1;

    pub fn new(server_id: i64, mtu_size: u16) -> Self {
        Self { server_id, server_security: false, mtu_size, magic: [0; 16] }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut stream = BinaryStream::new();
        stream.put_byte(Self::ID);
        OfflineMessage::write_magic(&mut stream);
        stream.put_long(self.server_id);
        stream.put_bool(self.server_security);
        stream.put_lshort(self.mtu_size);
        stream.into_inner()
    }

    pub fn decode(buffer: Vec<u8>) -> Self {
        let mut stream = BinaryStream::from_bytes(buffer);
        let _ = stream.get_byte();
        let magic = OfflineMessage::read_magic(&mut stream);
        let server_id = stream.get_long().unwrap_or(0);
        let server_security = stream.get_bool().unwrap_or(false);
        let mtu_size = stream.get_lshort().unwrap_or(0);
        Self { server_id, server_security, mtu_size, magic }
    }
}
