use crate::utils::BinaryStream;
use crate::raknet::protocol::{constants::DEFAULT_PROTOCOL_VERSION, message_ids::MessageIdentifiers, packet::OfflineMessage};

pub struct OpenConnectionRequest1 {
    pub protocol: u8,
    pub mtu_size: usize,
    pub magic: [u8; 16],
}

impl OpenConnectionRequest1 {
    pub const ID: u8 = MessageIdentifiers::ID_OPEN_CONNECTION_REQUEST_1;

    pub fn new(protocol: u8, mtu_size: usize) -> Self {
        Self { protocol, mtu_size, magic: [0; 16] }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut stream = BinaryStream::new();
        stream.put_byte(Self::ID);
        OfflineMessage::write_magic(&mut stream);
        stream.put_byte(self.protocol);
        let pad = self.mtu_size.saturating_sub(stream.len());
        stream.put(&vec![0u8; pad]);
        stream.into_inner()
    }

    pub fn decode(buffer: Vec<u8>) -> Self {
        let len = buffer.len();
        let mut stream = BinaryStream::from_bytes(buffer);
        let _ = stream.get_byte();
        let magic = OfflineMessage::read_magic(&mut stream);
        let protocol = stream.get_byte().unwrap_or(DEFAULT_PROTOCOL_VERSION);
        Self { protocol, mtu_size: len, magic }
    }

    pub fn is_valid(&self) -> bool {
        self.magic == OfflineMessage::magic_const()
    }
}
