use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

pub struct ResourcePackDataInfoPacket {
    pub pack_id: String,
    pub max_chunk_size: i32,
    pub chunk_count: i32,
    pub compressed_pack_size: i64,
    pub sha256: String,
}

impl DataPacket for ResourcePackDataInfoPacket {
    const NETWORK_ID: u8 = ProtocolInfo::RESOURCE_PACK_DATA_INFO_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_string(&self.pack_id);
        stream.put_lint(self.max_chunk_size);
        stream.put_lint(self.chunk_count);
        stream.put_llong(self.compressed_pack_size);
        stream.put_string(&self.sha256);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        Self {
            pack_id: s.get_string().unwrap_or_default(),
            max_chunk_size: s.get_lint().unwrap_or(0),
            chunk_count: s.get_lint().unwrap_or(0),
            compressed_pack_size: s.get_llong().unwrap_or(0),
            sha256: s.get_string().unwrap_or_default(),
        }
    }

    fn name(&self) -> &'static str { "ResourcePackDataInfoPacket" }
}

pub struct ResourcePackChunkDataPacket {
    pub pack_id: String,
    pub chunk_index: i32,
    pub progress: i64,
    pub data: Vec<u8>,
}

impl DataPacket for ResourcePackChunkDataPacket {
    const NETWORK_ID: u8 = ProtocolInfo::RESOURCE_PACK_CHUNK_DATA_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_string(&self.pack_id);
        stream.put_lint(self.chunk_index);
        stream.put_llong(self.progress);
        // Length as little-endian int, followed by raw chunk payload
        stream.put_lint(self.data.len() as i32);
        stream.put(&self.data);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let pack_id = s.get_string().unwrap_or_default();
        let chunk_index = s.get_lint().unwrap_or(0);
        let progress = s.get_llong().unwrap_or(0);
        let len = s.get_lint().unwrap_or(0).max(0) as usize;
        let data = s.get(len).unwrap_or_default();
        Self { pack_id, chunk_index, progress, data }
    }

    fn name(&self) -> &'static str { "ResourcePackChunkDataPacket" }
}

pub struct ResourcePackChunkRequestPacket {
    pub pack_id: String,
    pub chunk_index: i32,
}

impl DataPacket for ResourcePackChunkRequestPacket {
    const NETWORK_ID: u8 = ProtocolInfo::RESOURCE_PACK_CHUNK_REQUEST_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_string(&self.pack_id);
        stream.put_lint(self.chunk_index);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        Self {
            pack_id: s.get_string().unwrap_or_default(),
            chunk_index: s.get_lint().unwrap_or(0),
        }
    }

    fn name(&self) -> &'static str { "ResourcePackChunkRequestPacket" }
}
