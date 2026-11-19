use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

pub struct FullChunkDataPacket {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub data: Vec<u8>,
}

impl FullChunkDataPacket {
    pub fn new(chunk_x: i32, chunk_z: i32, data: Vec<u8>) -> Self {
        Self { chunk_x, chunk_z, data }
    }
}

impl DataPacket for FullChunkDataPacket {
    const NETWORK_ID: u8 = ProtocolInfo::FULL_CHUNK_DATA_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_var_int(self.chunk_x);
        stream.put_var_int(self.chunk_z);
        stream.put_byte_string(&self.data);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let chunk_x = stream.get_var_int().unwrap_or(0);
        let chunk_z = stream.get_var_int().unwrap_or(0);
        let data = stream.get_byte_string().unwrap_or_default();
        Self { chunk_x, chunk_z, data }
    }

    fn name(&self) -> &'static str { "FullChunkDataPacket" }
}
