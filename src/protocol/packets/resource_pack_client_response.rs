use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

pub const STATUS_REFUSED: u8 = 1;
pub const STATUS_SEND_PACKS: u8 = 2;
pub const STATUS_HAVE_ALL_PACKS: u8 = 3;
pub const STATUS_COMPLETED: u8 = 4;

pub struct ResourcePackClientResponsePacket {
    pub status: u8,
    pub pack_ids: Vec<String>,
}

impl DataPacket for ResourcePackClientResponsePacket {
    const NETWORK_ID: u8 = ProtocolInfo::RESOURCE_PACK_CLIENT_RESPONSE_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_byte(self.status);
        stream.put_lshort(self.pack_ids.len() as u16);
        for id in &self.pack_ids {
            stream.put_string(id);
        }
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let status = stream.get_byte().unwrap_or(0);
        let count = stream.get_lshort().unwrap_or(0);
        let mut pack_ids = Vec::new();
        for _ in 0..count {
            if stream.eof() { break; }
            pack_ids.push(stream.get_string().unwrap_or_default());
        }
        Self { status, pack_ids }
    }

    fn name(&self) -> &'static str { "ResourcePackClientResponsePacket" }
}
