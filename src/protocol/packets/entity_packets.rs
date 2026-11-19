use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

#[derive(Debug, Clone)]
pub struct Attribute {
    pub min: f32,
    pub max: f32,
    pub current: f32,
    pub default: f32,
    pub name: String,
}

impl Attribute {
    pub fn new(name: impl Into<String>, min: f32, max: f32, current: f32) -> Self {
        Self { min, max, current, default: current, name: name.into() }
    }
    pub fn movement_speed(value: f32) -> Self {
        Self { min: 0.0, max: 3.4028235e38, current: value, default: 0.1, name: "minecraft:movement".to_string() }
    }
    pub fn health(value: f32) -> Self {
        Self { min: 0.0, max: 20.0, current: value, default: 20.0, name: "minecraft:health".to_string() }
    }
    pub fn hunger(value: f32) -> Self {
        Self { min: 0.0, max: 20.0, current: value, default: 20.0, name: "minecraft:player.hunger".to_string() }
    }
    pub fn food_saturation(value: f32) -> Self {
        Self { min: 0.0, max: 20.0, current: value, default: 20.0, name: "minecraft:player.saturation".to_string() }
    }
    pub fn experience_level(value: f32) -> Self {
        Self { min: 0.0, max: 24791.0, current: value, default: 0.0, name: "minecraft:player.level".to_string() }
    }
    pub fn experience_progress(value: f32) -> Self {
        Self { min: 0.0, max: 1.0, current: value, default: 0.0, name: "minecraft:player.experience".to_string() }
    }
}

pub struct UpdateAttributesPacket {
    pub entity_runtime_id: i64,
    pub attributes: Vec<Attribute>,
}

impl UpdateAttributesPacket {
    pub fn new(entity_runtime_id: i64) -> Self {
        Self { entity_runtime_id, attributes: Vec::new() }
    }
}

impl DataPacket for UpdateAttributesPacket {
    const NETWORK_ID: u8 = ProtocolInfo::UPDATE_ATTRIBUTES_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_entity_id(self.entity_runtime_id);
        stream.put_unsigned_var_int(self.attributes.len() as u32);
        for attr in &self.attributes {
            stream.put_lfloat(attr.min);
            stream.put_lfloat(attr.max);
            stream.put_lfloat(attr.current);
            stream.put_lfloat(attr.default);
            stream.put_string(&attr.name);
        }
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let entity_runtime_id = stream.get_entity_id().unwrap_or(0);
        let count = stream.get_unsigned_var_int().unwrap_or(0) as usize;
        let mut attributes = Vec::with_capacity(count);
        for _ in 0..count {
            let min = stream.get_lfloat().unwrap_or(0.0);
            let max = stream.get_lfloat().unwrap_or(0.0);
            let current = stream.get_lfloat().unwrap_or(0.0);
            let default = stream.get_lfloat().unwrap_or(0.0);
            let name = stream.get_string().unwrap_or_default();
            attributes.push(Attribute { name, min, max, current, default });
        }
        Self { entity_runtime_id, attributes }
    }

    fn name(&self) -> &'static str { "UpdateAttributesPacket" }
}

pub struct SetEntityMotionPacket {
    pub entity_runtime_id: i64,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl SetEntityMotionPacket {
    pub fn new(entity_runtime_id: i64, x: f32, y: f32, z: f32) -> Self {
        Self { entity_runtime_id, x, y, z }
    }
}

impl DataPacket for SetEntityMotionPacket {
    const NETWORK_ID: u8 = ProtocolInfo::SET_ENTITY_MOTION_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_entity_id(self.entity_runtime_id);
        stream.put_vector3f(self.x, self.y, self.z);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let entity_runtime_id = stream.get_entity_id().unwrap_or(0);
        let (x, y, z) = stream.get_vector3f().unwrap_or((0.0, 0.0, 0.0));
        Self { entity_runtime_id, x, y, z }
    }

    fn name(&self) -> &'static str { "SetEntityMotionPacket" }
}

pub struct SetEntityLinkPacket {
    pub from: i64,
    pub to: i64,
    pub link_type: u8,
}

impl SetEntityLinkPacket {
    pub const TYPE_RIDE: u8 = 1;
    pub const TYPE_PASSENGER: u8 = 2;
}

impl DataPacket for SetEntityLinkPacket {
    const NETWORK_ID: u8 = ProtocolInfo::SET_ENTITY_LINK_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_entity_id(self.from);
        stream.put_entity_id(self.to);
        stream.put_byte(self.link_type);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let from = stream.get_entity_id().unwrap_or(0);
        let to = stream.get_entity_id().unwrap_or(0);
        let link_type = stream.get_byte().unwrap_or(0);
        Self { from, to, link_type }
    }

    fn name(&self) -> &'static str { "SetEntityLinkPacket" }
}

pub struct EntityEventPacket {
    pub entity_runtime_id: i64,
    pub event: u8,
    pub data: i32,
}

impl EntityEventPacket {
    pub const JUMP: u8 = 1;
    pub const HURT_ANIMATION: u8 = 2;
    pub const DEATH_ANIMATION: u8 = 3;
    pub const TAME_FAIL: u8 = 6;
    pub const TAME_SUCCESS: u8 = 7;
    pub const SHAKE_WET: u8 = 8;
    pub const USE_ITEM: u8 = 9;
    pub const EAT_GRASS_ANIMATION: u8 = 10;
    pub const FISH_HOOK_BUBBLE: u8 = 11;
    pub const FISH_HOOK_POSITION: u8 = 12;
    pub const FISH_HOOK_HOOK: u8 = 13;
    pub const FISH_HOOK_TEASE: u8 = 14;
    pub const SQUID_INK_CLOUD: u8 = 15;
    pub const AMBIENT_SOUND: u8 = 16;
    pub const RESPAWN: u8 = 18;
    pub const XP: u8 = 34;
    pub const ARROW_SHAKE: u8 = 39;
    pub const EATING_ITEM: u8 = 57;
    pub const CONSUME_TOTEM: u8 = 65;
}

impl DataPacket for EntityEventPacket {
    const NETWORK_ID: u8 = ProtocolInfo::ENTITY_EVENT_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_entity_id(self.entity_runtime_id);
        stream.put_byte(self.event);
        stream.put_var_int(self.data);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        Self {
            entity_runtime_id: s.get_entity_id().unwrap_or(0),
            event: s.get_byte().unwrap_or(0),
            data: s.get_var_int().unwrap_or(0),
        }
    }

    fn name(&self) -> &'static str { "EntityEventPacket" }
}

pub struct MobEffectPacket {
    pub entity_runtime_id: i64,
    pub event_id: u8,
    pub effect_id: i32,
    pub amplifier: i32,
    pub particles: bool,
    pub duration: i32,
}

pub const EVENT_ADD: u8 = 1;
pub const EVENT_MODIFY: u8 = 2;
pub const EVENT_REMOVE: u8 = 3;

impl DataPacket for MobEffectPacket {
    const NETWORK_ID: u8 = ProtocolInfo::MOB_EFFECT_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_entity_id(self.entity_runtime_id);
        stream.put_byte(self.event_id);
        stream.put_var_int(self.effect_id);
        stream.put_var_int(self.amplifier);
        stream.put_bool(self.particles);
        stream.put_var_int(self.duration);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let entity_runtime_id = s.get_entity_id().unwrap_or(0);
        let event_id = s.get_byte().unwrap_or(0);
        let effect_id = s.get_var_int().unwrap_or(0);
        let amplifier = s.get_var_int().unwrap_or(0);
        let particles = s.get_bool().unwrap_or(false);
        let duration = s.get_var_int().unwrap_or(0);
        Self { entity_runtime_id, event_id, effect_id, amplifier, particles, duration }
    }

    fn name(&self) -> &'static str { "MobEffectPacket" }
}

pub struct AddItemEntityPacket {
    pub entity_runtime_id: i64,
    pub item: crate::protocol::types::Item,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub speed_x: f32,
    pub speed_y: f32,
    pub speed_z: f32,
}

impl AddItemEntityPacket {
    pub fn new(eid: i64, item: crate::protocol::types::Item, x: f32, y: f32, z: f32) -> Self {
        Self {
            entity_runtime_id: eid, item, x, y, z,
            speed_x: 0.0, speed_y: 0.0, speed_z: 0.0,
        }
    }
}

impl DataPacket for AddItemEntityPacket {
    const NETWORK_ID: u8 = ProtocolInfo::ADD_ITEM_ENTITY_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        // Unique entity ID and runtime entity ID (both share the same ID in MCPE 1.1.5)
        stream.put_entity_id(self.entity_runtime_id);
        stream.put_entity_id(self.entity_runtime_id);
        self.item.put(stream);
        stream.put_vector3f(self.x, self.y, self.z);
        stream.put_vector3f(self.speed_x, self.speed_y, self.speed_z);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let _unique_id = stream.get_entity_id().unwrap_or(0);
        let entity_runtime_id = stream.get_entity_id().unwrap_or(0);
        let mut item = crate::protocol::types::Item::air();
        item.get(&mut stream);
        let (x, y, z) = stream.get_vector3f().unwrap_or((0.0, 0.0, 0.0));
        let (speed_x, speed_y, speed_z) = stream.get_vector3f().unwrap_or((0.0, 0.0, 0.0));
        Self {
            entity_runtime_id, item, x, y, z,
            speed_x, speed_y, speed_z,
        }
    }

    fn name(&self) -> &'static str { "AddItemEntityPacket" }
}

pub struct TakeItemEntityPacket {
    pub target: i64,
    pub entity_id: i64,
}

impl DataPacket for TakeItemEntityPacket {
    const NETWORK_ID: u8 = ProtocolInfo::TAKE_ITEM_ENTITY_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_entity_id(self.target);
        stream.put_entity_id(self.entity_id);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let target = s.get_entity_id().unwrap_or(0);
        let entity_id = s.get_entity_id().unwrap_or(0);
        Self { target, entity_id }
    }

    fn name(&self) -> &'static str { "TakeItemEntityPacket" }
}
