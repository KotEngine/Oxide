use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;
use crate::protocol::types::{ContainerIds, Item};

#[derive(Debug, Clone)]
pub struct ContainerOpenPacket {
    pub window_id: u8,
    pub container_type: i8,
    pub x: i32,
    pub y: u32,
    pub z: i32,
    pub entity_id: i64,
}

impl ContainerOpenPacket {
    pub fn new(window_id: u8, container_type: i8, x: i32, y: u32, z: i32) -> Self {
        Self { window_id, container_type, x, y, z, entity_id: -1 }
    }
}

impl DataPacket for ContainerOpenPacket {
    const NETWORK_ID: u8 = ProtocolInfo::CONTAINER_OPEN_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_byte(self.window_id);
        stream.put_byte(self.container_type as u8);
        stream.put_block_coords(self.x, self.y, self.z);
        stream.put_entity_id(self.entity_id);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let window_id = s.get_byte().unwrap_or(0);
        let container_type = s.get_byte().unwrap_or(0) as i8;
        let (x, y, z) = s.get_block_coords().unwrap_or((0, 0, 0));
        let entity_id = s.get_entity_id().unwrap_or(-1);
        Self { window_id, container_type, x, y: y as u32, z, entity_id }
    }

    fn name(&self) -> &'static str { "ContainerOpenPacket" }
}

#[derive(Debug, Clone)]
pub struct ContainerClosePacket {
    pub window_id: u8,
}

impl DataPacket for ContainerClosePacket {
    const NETWORK_ID: u8 = ProtocolInfo::CONTAINER_CLOSE_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_byte(self.window_id);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        Self { window_id: s.get_byte().unwrap_or(0) }
    }

    fn name(&self) -> &'static str { "ContainerClosePacket" }
}

#[derive(Debug, Clone)]
pub struct ContainerSetSlotPacket {
    pub window_id: u8,
    pub slot: i32,
    pub hotbar_slot: i32,
    pub item: Item,
    pub select_slot: u8,
}

impl ContainerSetSlotPacket {
    pub fn new(window_id: u8, slot: i32, item: Item) -> Self {
        Self { window_id, slot, hotbar_slot: 0, item, select_slot: 0 }
    }
}

impl DataPacket for ContainerSetSlotPacket {
    const NETWORK_ID: u8 = ProtocolInfo::CONTAINER_SET_SLOT_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_byte(self.window_id);
        stream.put_var_int(self.slot);
        stream.put_var_int(self.hotbar_slot);
        self.item.put(stream);
        stream.put_byte(self.select_slot);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let window_id = s.get_byte().unwrap_or(0);
        let slot = s.get_var_int().unwrap_or(0);
        let hotbar_slot = s.get_var_int().unwrap_or(0);
        let mut item = Item::air();
        item.get(&mut s);
        let select_slot = s.get_byte().unwrap_or(0);
        Self { window_id, slot, hotbar_slot, item, select_slot }
    }

    fn name(&self) -> &'static str { "ContainerSetSlotPacket" }
}

#[derive(Debug, Clone)]
pub struct ContainerSetContentPacket {
    pub window_id: u32,
    pub target_eid: i64,
    pub items: Vec<Item>,
    pub hotbar: Vec<i32>,
}

impl ContainerSetContentPacket {
    pub fn new(window_id: u32) -> Self {
        Self { window_id, target_eid: 0, items: Vec::new(), hotbar: Vec::new() }
    }
}

impl DataPacket for ContainerSetContentPacket {
    const NETWORK_ID: u8 = ProtocolInfo::CONTAINER_SET_CONTENT_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_unsigned_var_int(self.window_id);
        stream.put_entity_id(self.target_eid);
        stream.put_unsigned_var_int(self.items.len() as u32);
        for item in &self.items {
            item.put(stream);
        }
        if self.window_id == ContainerIds::INVENTORY as u32 && !self.hotbar.is_empty() {
            stream.put_unsigned_var_int(self.hotbar.len() as u32);
            for v in &self.hotbar {
                stream.put_var_int(*v);
            }
        } else {
            stream.put_unsigned_var_int(0);
        }
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let window_id = s.get_unsigned_var_int().unwrap_or(0);
        let target_eid = s.get_entity_id().unwrap_or(0);
        let count = s.get_unsigned_var_int().unwrap_or(0).min(512);
        let mut items = Vec::with_capacity(count as usize);
        for _ in 0..count {
            if s.feof() { break; }
            let mut item = Item::air();
            item.get(&mut s);
            items.push(item);
        }
        let hotbar_count = s.get_unsigned_var_int().unwrap_or(0).min(512);
        let mut hotbar = Vec::with_capacity(hotbar_count as usize);
        for _ in 0..hotbar_count {
            if s.feof() { break; }
            hotbar.push(s.get_var_int().unwrap_or(0));
        }
        Self { window_id, target_eid, items, hotbar }
    }
    fn name(&self) -> &'static str { "ContainerSetContentPacket" }
}

#[derive(Debug, Clone)]
pub struct ContainerSetDataPacket {
    pub window_id: u8,
    pub property: i32,
    pub value: i32,
}

impl DataPacket for ContainerSetDataPacket {
    const NETWORK_ID: u8 = ProtocolInfo::CONTAINER_SET_DATA_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_byte(self.window_id);
        stream.put_var_int(self.property);
        stream.put_var_int(self.value);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let window_id = s.get_byte().unwrap_or(0);
        let property = s.get_var_int().unwrap_or(0);
        let value = s.get_var_int().unwrap_or(0);
        Self { window_id, property, value }
    }

    fn name(&self) -> &'static str { "ContainerSetDataPacket" }
}

pub const ACTION_GIVE_ITEM: u32 = 0;
pub const ACTION_ENCHANT_ITEM: u32 = 2;

#[derive(Debug, Clone)]
pub struct InventoryActionPacket {
    pub action_id: u32,
    pub item: Item,
    pub enchantment_id: i32,
    pub enchantment_level: i32,
}

impl DataPacket for InventoryActionPacket {
    const NETWORK_ID: u8 = ProtocolInfo::INVENTORY_ACTION_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_unsigned_var_int(self.action_id);
        self.item.put(stream);
        stream.put_var_int(self.enchantment_id);
        stream.put_var_int(self.enchantment_level);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let action_id = s.get_unsigned_var_int().unwrap_or(0);
        let mut item = Item::air();
        item.get(&mut s);
        let enchantment_id = s.get_var_int().unwrap_or(0);
        let enchantment_level = s.get_var_int().unwrap_or(0);
        Self { action_id, item, enchantment_id, enchantment_level }
    }

    fn name(&self) -> &'static str { "InventoryActionPacket" }
}

#[derive(Debug, Clone)]
pub struct DropItemPacket {
    pub item: Item,
    pub unknown: u8,
}

impl DataPacket for DropItemPacket {
    const NETWORK_ID: u8 = ProtocolInfo::DROP_ITEM_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_byte(self.unknown);
        self.item.put(stream);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let unknown = s.get_byte().unwrap_or(0);
        let mut item = Item::air();
        item.get(&mut s);
        Self { item, unknown }
    }

    fn name(&self) -> &'static str { "DropItemPacket" }
}

#[derive(Debug, Clone)]
pub struct CraftingEventPacket {
    pub window_id: u8,
    pub recipe_type: i32,
    pub recipe_id: String,
    pub input: Vec<Item>,
    pub result: Vec<Item>,
}

impl DataPacket for CraftingEventPacket {
    const NETWORK_ID: u8 = ProtocolInfo::CRAFTING_EVENT_PACKET;

    fn encode_payload(&self, _stream: &mut BinaryStream) {}

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let window_id = s.get_byte().unwrap_or(0);
        let recipe_type = s.get_var_int().unwrap_or(0);
        let recipe_id = match s.get_uuid() {
            Ok(u) => u.to_string(),
            Err(_) => String::new(),
        };
        let in_count = s.get_unsigned_var_int().unwrap_or(0).min(128);
        let mut input = Vec::with_capacity(in_count as usize);
        for _ in 0..in_count {
            if s.feof() { break; }
            let mut it = Item::air();
            it.get(&mut s);
            input.push(it);
        }
        let out_count = s.get_unsigned_var_int().unwrap_or(0).min(128);
        let mut result = Vec::with_capacity(out_count as usize);
        for _ in 0..out_count {
            if s.feof() { break; }
            let mut it = Item::air();
            it.get(&mut s);
            result.push(it);
        }
        Self { window_id, recipe_type, recipe_id, input, result }
    }

    fn name(&self) -> &'static str { "CraftingEventPacket" }
}
