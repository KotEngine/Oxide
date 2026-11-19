use crate::utils::BinaryStream;
use crate::raknet::protocol::{message_ids::MessageIdentifiers, packet::{OfflineMessage, write_address, read_address}};
use crate::raknet::InternetAddress;

pub struct OpenConnectionRequest2 {
    pub client_id: i64,
    pub server_address: Option<InternetAddress>,
    pub mtu_size: u16,
    pub magic: [u8; 16],
}

impl OpenConnectionRequest2 {
    pub const ID: u8 = MessageIdentifiers::ID_OPEN_CONNECTION_REQUEST_2;

    pub fn new(client_id: i64, server_address: InternetAddress, mtu_size: u16) -> Self {
        Self { client_id, server_address: Some(server_address), mtu_size, magic: [0; 16] }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut stream = BinaryStream::new();
        stream.put_byte(Self::ID);
        OfflineMessage::write_magic(&mut stream);
        if let Some(addr) = &self.server_address {
            write_address(&mut stream, addr);
        }
        stream.put_short(self.mtu_size);
        stream.put_long(self.client_id);
        stream.into_inner()
    }

    pub fn decode(buffer: Vec<u8>) -> Self {
        let mut stream = BinaryStream::from_bytes(buffer);
        let _ = stream.get_byte();
        let magic = OfflineMessage::read_magic(&mut stream);
        let server_address = read_address(&mut stream);
        let mtu_size = stream.get_short().unwrap_or(0);
        let client_id = stream.get_long().unwrap_or(0);
        Self { client_id, server_address, mtu_size, magic }
    }

    pub fn is_valid(&self) -> bool {
        self.magic == OfflineMessage::magic_const()
    }
}
