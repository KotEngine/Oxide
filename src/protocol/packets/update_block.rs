use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

pub const FLAG_NONE: u8 = 0b0000;
pub const FLAG_NEIGHBORS: u8 = 0b0001;
pub const FLAG_NETWORK: u8 = 0b0010;
pub const FLAG_NOGRAPHIC: u8 = 0b0100;
pub const FLAG_PRIORITY: u8 = 0b1000;

pub const FLAG_ALL: u8 = FLAG_NEIGHBORS | FLAG_NETWORK;
pub const FLAG_ALL_PRIORITY: u8 = FLAG_ALL | FLAG_PRIORITY;

pub struct UpdateBlockPacket {
    pub x: i32,
    pub z: i32,
    pub y: u32,
    pub block_id: u32,
    pub block_data: u32,
    pub flags: u8,
}

impl UpdateBlockPacket {
    pub fn new(x: i32, y: u32, z: i32, block_id: u32, block_data: u32, flags: u8) -> Self {
        Self { x, z, y, block_id, block_data, flags }
    }
}

impl DataPacket for UpdateBlockPacket {
    const NETWORK_ID: u8 = ProtocolInfo::UPDATE_BLOCK_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_block_coords(self.x, self.y, self.z);
        stream.put_unsigned_var_int(self.block_id);
        stream.put_unsigned_var_int(((self.flags as u32) << 4) | (self.block_data & 0x0f));
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let (x, y, z) = stream.get_block_coords().unwrap_or((0, 0, 0));
        let block_id = stream.get_unsigned_var_int().unwrap_or(0);
        let mask = stream.get_unsigned_var_int().unwrap_or(0);
        let block_data = mask & 0x0f;
        let flags = (mask >> 4) as u8;
        Self { x, z, y, block_id, block_data, flags }
    }

    fn name(&self) -> &'static str { "UpdateBlockPacket" }
}
