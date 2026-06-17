use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;
use crate::protocol::types::Item;

pub struct CraftingDataPacket {
    pub entries: Vec<Vec<u8>>,
    pub clean_recipes: bool,
}

impl Default for CraftingDataPacket {
    fn default() -> Self { Self { entries: Vec::new(), clean_recipes: true } }
}

impl CraftingDataPacket {
    pub fn new() -> Self { Self::default() }
}

impl DataPacket for CraftingDataPacket {
    const NETWORK_ID: u8 = ProtocolInfo::CRAFTING_DATA_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_unsigned_var_int(self.entries.len() as u32);
        for entry in &self.entries {
            stream.put_string(std::str::from_utf8(entry).unwrap_or(""));
        }
        stream.put_bool(self.clean_recipes);
    }

    fn decode(_buf: Vec<u8>) -> Self where Self: Sized {
        Self::default()
    }

    fn name(&self) -> &'static str { "CraftingDataPacket" }
}

impl Drop for CraftingDataPacket {
    fn drop(&mut self) {
        let _ = Item::air;
    }
}
