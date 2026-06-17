use crate::utils::BinaryStream;
use crate::raknet::protocol::message_ids::MessageIdentifiers;

pub struct ConnectedPong {
    pub send_ping_time: i64,
    pub send_pong_time: i64,
}

impl ConnectedPong {
    pub const ID: u8 = MessageIdentifiers::ID_CONNECTED_PONG;

    pub fn new(send_ping_time: i64, send_pong_time: i64) -> Self {
        Self { send_ping_time, send_pong_time }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut stream = BinaryStream::new();
        stream.put_byte(Self::ID);
        stream.put_long(self.send_ping_time);
        stream.put_long(self.send_pong_time);
        stream.into_inner()
    }

    pub fn decode(buffer: Vec<u8>) -> Self {
        let mut stream = BinaryStream::from_bytes(buffer);
        let _ = stream.get_byte();
        let send_ping_time = stream.get_long().unwrap_or(0);
        let send_pong_time = stream.get_long().unwrap_or(0);
        Self { send_ping_time, send_pong_time }
    }
}
