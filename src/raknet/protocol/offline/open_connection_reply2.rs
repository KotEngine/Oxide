use crate::utils::BinaryStream;
use crate::raknet::protocol::{message_ids::MessageIdentifiers, packet::{OfflineMessage, write_address, read_address}};
use crate::raknet::InternetAddress;

pub struct OpenConnectionReply2 {
    pub server_id: i64,
    pub client_address: Option<InternetAddress>,
    pub mtu_size: u16,
    pub server_security: bool,
    pub magic: [u8; 16],
}

impl OpenConnectionReply2 {
    pub const ID: u8 = MessageIdentifiers::ID_OPEN_CONNECTION_REPLY_2;

    pub fn new(server_id: i64, client_address: InternetAddress, mtu_size: u16) -> Self {
        Self { server_id, client_address: Some(client_address), mtu_size, server_security: false, magic: [0; 16] }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut stream = BinaryStream::new();
        stream.put_byte(Self::ID);
        OfflineMessage::write_magic(&mut stream);
        stream.put_long(self.server_id);
        if let Some(addr) = &self.client_address {
            write_address(&mut stream, addr);
        }
        stream.put_lshort(self.mtu_size);
        stream.put_bool(self.server_security);
        stream.into_inner()
    }

    pub fn decode(buffer: Vec<u8>) -> Self {
        let mut stream = BinaryStream::from_bytes(buffer);
        let _ = stream.get_byte();
        let magic = OfflineMessage::read_magic(&mut stream);
        let server_id = stream.get_long().unwrap_or(0);
        let client_address = read_address(&mut stream);
        let mtu_size = stream.get_lshort().unwrap_or(0);
        let server_security = stream.get_bool().unwrap_or(false);
        Self { server_id, client_address, mtu_size, server_security, magic }
    }
}
