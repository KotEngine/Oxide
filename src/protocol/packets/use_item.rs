use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;
use crate::protocol::types::Item;

#[derive(Debug)]
pub struct UseItemPacket {
    pub x: i32,
    pub y: u32,
    pub z: i32,
    pub block_id: u32,
    pub face: i32,
    pub fx: f32,
    pub fy: f32,
    pub fz: f32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    pub slot: i32,
    pub item: Item,
}

impl DataPacket for UseItemPacket {
    const NETWORK_ID: u8 = ProtocolInfo::USE_ITEM_PACKET;

    fn encode_payload(&self, _stream: &mut BinaryStream) {}

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let (x, y, z) = stream.get_block_coords().unwrap_or((0, 0, 0));
        let block_id = stream.get_unsigned_var_int().unwrap_or(0);
        let face = stream.get_var_int().unwrap_or(0);
        let (fx, fy, fz) = stream.get_vector3f().unwrap_or((0.0, 0.0, 0.0));
        let (pos_x, pos_y, pos_z) = stream.get_vector3f().unwrap_or((0.0, 0.0, 0.0));
        let slot = stream.get_var_int().unwrap_or(0);
        let mut item = Item::air();
        item.get(&mut stream);
        Self { x, y, z, block_id, face, fx, fy, fz, pos_x, pos_y, pos_z, slot, item }
    }

    fn name(&self) -> &'static str { "UseItemPacket" }
}
