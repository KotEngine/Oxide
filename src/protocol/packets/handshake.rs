use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

pub struct ServerToClientHandshakePacket {
    pub public_key: String,
    pub server_token: String,
}

impl DataPacket for ServerToClientHandshakePacket {
    const NETWORK_ID: u8 = ProtocolInfo::SERVER_TO_CLIENT_HANDSHAKE_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_string(&self.public_key);
        stream.put_string(&self.server_token);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        Self {
            public_key: s.get_string().unwrap_or_default(),
            server_token: s.get_string().unwrap_or_default(),
        }
    }

    fn name(&self) -> &'static str { "ServerToClientHandshakePacket" }
}

pub struct ClientToServerHandshakePacket;

impl DataPacket for ClientToServerHandshakePacket {
    const NETWORK_ID: u8 = ProtocolInfo::CLIENT_TO_SERVER_HANDSHAKE_PACKET;

    fn encode_payload(&self, _stream: &mut BinaryStream) {}

    fn decode(_buf: Vec<u8>) -> Self where Self: Sized { Self }

    fn name(&self) -> &'static str { "ClientToServerHandshakePacket" }
}
