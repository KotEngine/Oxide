use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

pub const ENTRY_SHAPELESS: i32 = 0;
pub const ENTRY_SHAPED: i32 = 1;
pub const ENTRY_FURNACE: i32 = 2;
pub const ENTRY_FURNACE_DATA: i32 = 3;
pub const ENTRY_MULTI: i32 = 4;
pub const ENTRY_SHULKER_BOX: i32 = 5;

/// Single recipe: recipe type and serialized recipe payload.
/// `entry_type < 0`: writes varint(-1) with no payload.
#[derive(Debug, Clone)]
pub struct CraftingDataEntry {
    pub entry_type: i32,
    pub data: Vec<u8>,
}

pub struct CraftingDataPacket {
    pub entries: Vec<CraftingDataEntry>,
    pub clean_recipes: bool,
}

impl Default for CraftingDataPacket {
    fn default() -> Self { Self { entries: Vec::new(), clean_recipes: false } }
}

impl CraftingDataPacket {
    pub fn new() -> Self { Self::default() }
}

impl DataPacket for CraftingDataPacket {
    const NETWORK_ID: u8 = ProtocolInfo::CRAFTING_DATA_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_unsigned_var_int(self.entries.len() as u32);
        for entry in &self.entries {
            if entry.entry_type >= 0 {
                stream.put_var_int(entry.entry_type);
                stream.put(&entry.data);
            } else {
                stream.put_var_int(-1);
            }
        }
        stream.put_bool(self.clean_recipes);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let count = s.get_unsigned_var_int().unwrap_or(0).min(4096);
        let mut entries = Vec::with_capacity(count as usize);
        for _ in 0..count {
            if s.feof() { break; }
            let entry_type = s.get_var_int().unwrap_or(-1);
            if entry_type < 0 {
                entries.push(CraftingDataEntry { entry_type, data: Vec::new() });
                continue;
            }
            let start = s.offset;
            match entry_type {
                ENTRY_SHAPELESS | ENTRY_SHULKER_BOX => {
                    let in_cnt = s.get_unsigned_var_int().unwrap_or(0).min(128);
                    for _ in 0..in_cnt {
                        let mut it = crate::protocol::types::Item::air();
                        it.get(&mut s);
                    }
                    let out_cnt = s.get_unsigned_var_int().unwrap_or(0).min(128);
                    for _ in 0..out_cnt {
                        let mut it = crate::protocol::types::Item::air();
                        it.get(&mut s);
                    }
                    let _ = s.get_uuid();
                }
                ENTRY_SHAPED => {
                    let w = s.get_var_int().unwrap_or(0).max(0).min(16);
                    let h = s.get_var_int().unwrap_or(0).max(0).min(16);
                    let total = (w * h) as usize;
                    for _ in 0..total {
                        let mut it = crate::protocol::types::Item::air();
                        it.get(&mut s);
                    }
                    let out_cnt = s.get_unsigned_var_int().unwrap_or(0).min(128);
                    for _ in 0..out_cnt {
                        let mut it = crate::protocol::types::Item::air();
                        it.get(&mut s);
                    }
                    let _ = s.get_uuid();
                }
                ENTRY_FURNACE => {
                    let _ = s.get_var_int();
                    let mut it = crate::protocol::types::Item::air();
                    it.get(&mut s);
                }
                ENTRY_FURNACE_DATA => {
                    let _ = s.get_var_int();
                    let _ = s.get_var_int();
                    let mut it = crate::protocol::types::Item::air();
                    it.get(&mut s);
                }
                ENTRY_MULTI => {
                    let _ = s.get_uuid();
                }
                _ => {}
            }
            let end = s.offset;
            let data = if end > start && end <= s.buffer.len() {
                s.buffer[start..end].to_vec()
            } else {
                Vec::new()
            };
            entries.push(CraftingDataEntry { entry_type, data });
        }
        let clean_recipes = s.get_bool().unwrap_or(false);
        Self { entries, clean_recipes }
    }

    fn name(&self) -> &'static str { "CraftingDataPacket" }
}
