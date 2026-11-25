use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;
use crate::protocol::types::Item;

pub struct AddItemPacket {
    pub item: Item,
}

impl DataPacket for AddItemPacket {
    const NETWORK_ID: u8 = ProtocolInfo::ADD_ITEM_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        self.item.put(stream);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let mut item = Item::air();
        item.get(&mut s);
        Self { item }
    }

    fn name(&self) -> &'static str { "AddItemPacket" }
}

pub struct ReplaceItemInSlotPacket {
    pub item: Item,
}

impl DataPacket for ReplaceItemInSlotPacket {
    const NETWORK_ID: u8 = ProtocolInfo::REPLACE_ITEM_IN_SLOT_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        self.item.put(stream);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let mut item = Item::air();
        item.get(&mut s);
        Self { item }
    }

    fn name(&self) -> &'static str { "ReplaceItemInSlotPacket" }
}

pub struct AddHangingEntityPacket {
    pub entity_unique_id: i64,
    pub entity_runtime_id: i64,
    pub x: i32,
    pub y: u32,
    pub z: i32,
    pub unknown: i32,
}

impl DataPacket for AddHangingEntityPacket {
    const NETWORK_ID: u8 = ProtocolInfo::ADD_HANGING_ENTITY_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_entity_id(self.entity_unique_id);
        stream.put_entity_id(self.entity_runtime_id);
        stream.put_block_coords(self.x, self.y, self.z);
        stream.put_var_int(self.unknown);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let entity_unique_id = s.get_entity_id().unwrap_or(0);
        let entity_runtime_id = s.get_entity_id().unwrap_or(0);
        let (x, y, z) = s.get_block_coords().unwrap_or((0, 0, 0));
        let unknown = s.get_var_int().unwrap_or(0);
        Self { entity_unique_id, entity_runtime_id, x, y, z, unknown }
    }

    fn name(&self) -> &'static str { "AddHangingEntityPacket" }
}

pub struct SpawnExperienceOrbPacket {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub amount: i32,
}

impl DataPacket for SpawnExperienceOrbPacket {
    const NETWORK_ID: u8 = ProtocolInfo::SPAWN_EXPERIENCE_ORB_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_vector3f(self.x, self.y, self.z);
        stream.put_var_int(self.amount);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let (x, y, z) = s.get_vector3f().unwrap_or((0.0, 0.0, 0.0));
        let amount = s.get_var_int().unwrap_or(0);
        Self { x, y, z, amount }
    }

    fn name(&self) -> &'static str { "SpawnExperienceOrbPacket" }
}

#[derive(Debug, Clone)]
pub struct ItemFrameDropItemPacket {
    pub x: i32,
    pub y: u32,
    pub z: i32,
}

impl DataPacket for ItemFrameDropItemPacket {
    const NETWORK_ID: u8 = ProtocolInfo::ITEM_FRAME_DROP_ITEM_PACKET;

    // Client to server packet only, payload encode is empty
    fn encode_payload(&self, _stream: &mut BinaryStream) {}

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let (x, y, z) = s.get_block_coords().unwrap_or((0, 0, 0));
        Self { x, y, z }
    }

    fn name(&self) -> &'static str { "ItemFrameDropItemPacket" }
}

pub struct CameraPacket {
    pub camera_unique_id: i64,
    pub player_unique_id: i64,
}

impl DataPacket for CameraPacket {
    const NETWORK_ID: u8 = ProtocolInfo::CAMERA_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_entity_id(self.camera_unique_id);
        stream.put_entity_id(self.player_unique_id);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        Self {
            camera_unique_id: s.get_entity_id().unwrap_or(0),
            player_unique_id: s.get_entity_id().unwrap_or(0),
        }
    }

    fn name(&self) -> &'static str { "CameraPacket" }
}
