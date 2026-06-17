use crate::utils::BinaryStream;
use crate::raknet::protocol::message_ids::MessageIdentifiers;

pub struct ConnectionRequest {
    pub client_id: i64,
    pub send_ping_time: i64,
    pub use_security: bool,
}

impl ConnectionRequest {
    pub const ID: u8 = MessageIdentifiers::ID_CONNECTION_REQUEST;

    pub fn new(client_id: i64, send_ping_time: i64) -> Self {
        Self { client_id, send_ping_time, use_security: false }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut stream = BinaryStream::new();
        stream.put_byte(Self::ID);
        stream.put_long(self.client_id);
        stream.put_long(self.send_ping_time);
        stream.put_bool(self.use_security);
        stream.into_inner()
    }

    pub fn decode(buffer: Vec<u8>) -> Self {
        let mut stream = BinaryStream::from_bytes(buffer);
        let _ = stream.get_byte();
        let client_id = stream.get_long().unwrap_or(0);
        let send_ping_time = stream.get_long().unwrap_or(0);
        let use_security = stream.get_bool().unwrap_or(false);
        Self { client_id, send_ping_time, use_security }
    }
}
