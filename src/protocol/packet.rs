use crate::utils::BinaryStream;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketDirection {
    ClientToServer,
    ServerToClient,
}

pub trait DataPacket: Send + Sync {
    const NETWORK_ID: u8;

    fn encode(&self) -> Vec<u8> {
        let mut stream = BinaryStream::new();
        stream.put_byte(Self::NETWORK_ID);
        self.encode_payload(&mut stream);
        stream.into_inner()
    }

    fn encode_payload(&self, stream: &mut BinaryStream);

    fn decode(buf: Vec<u8>) -> Self where Self: Sized;

    fn can_be_batched(&self) -> bool { true }
    fn can_be_sent_before_login(&self) -> bool { false }
    fn pid(&self) -> u8 { Self::NETWORK_ID }
    fn name(&self) -> &'static str { "UnknownPacket" }
}
