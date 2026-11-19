use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;
use crate::protocol::types::Item;

#[derive(Debug, Clone)]
pub struct MobEquipmentPacket {
    pub entity_runtime_id: i64,
    pub item: Item,
    pub inventory_slot: u8,
    pub hotbar_slot: u8,
    pub window_id: u8,
}

impl MobEquipmentPacket {
    pub fn new(entity_runtime_id: i64, item: Item, slot: u8) -> Self {
        Self { entity_runtime_id, item, inventory_slot: slot, hotbar_slot: slot, window_id: 0 }
    }
}

impl DataPacket for MobEquipmentPacket {
    const NETWORK_ID: u8 = ProtocolInfo::MOB_EQUIPMENT_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_entity_id(self.entity_runtime_id);
        self.item.put(stream);
        stream.put_byte(self.inventory_slot);
        stream.put_byte(self.hotbar_slot);
        stream.put_byte(self.window_id);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let entity_runtime_id = s.get_entity_id().unwrap_or(0);
        let mut item = Item::air();
        item.get(&mut s);
        let inventory_slot = s.get_byte().unwrap_or(0);
        let hotbar_slot = s.get_byte().unwrap_or(0);
        let window_id = s.get_byte().unwrap_or(0);
        Self { entity_runtime_id, item, inventory_slot, hotbar_slot, window_id }
    }

    fn name(&self) -> &'static str { "MobEquipmentPacket" }
}

pub struct MobArmorEquipmentPacket {
    pub entity_runtime_id: i64,
    pub helmet: Item,
    pub chestplate: Item,
    pub leggings: Item,
    pub boots: Item,
}

impl MobArmorEquipmentPacket {
    pub fn new(eid: i64) -> Self {
        Self { entity_runtime_id: eid, helmet: Item::air(), chestplate: Item::air(), leggings: Item::air(), boots: Item::air() }
    }
}

impl DataPacket for MobArmorEquipmentPacket {
    const NETWORK_ID: u8 = ProtocolInfo::MOB_ARMOR_EQUIPMENT_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_entity_id(self.entity_runtime_id);
        self.helmet.put(stream);
        self.chestplate.put(stream);
        self.leggings.put(stream);
        self.boots.put(stream);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let entity_runtime_id = s.get_entity_id().unwrap_or(0);
        let mut helmet = Item::air();
        helmet.get(&mut s);
        let mut chestplate = Item::air();
        chestplate.get(&mut s);
        let mut leggings = Item::air();
        leggings.get(&mut s);
        let mut boots = Item::air();
        boots.get(&mut s);
        Self { entity_runtime_id, helmet, chestplate, leggings, boots }
    }
    fn name(&self) -> &'static str { "MobArmorEquipmentPacket" }
}
