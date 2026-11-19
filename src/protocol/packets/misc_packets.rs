use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

pub struct BlockEntityDataPacket {
    pub x: i32,
    pub y: u32,
    pub z: i32,
    pub namedtag: Vec<u8>,
}

impl DataPacket for BlockEntityDataPacket {
    const NETWORK_ID: u8 = ProtocolInfo::BLOCK_ENTITY_DATA_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_block_coords(self.x, self.y, self.z);
        // NBT is written raw without varint length prefix
        stream.put(&self.namedtag);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let (x, y, z) = s.get_block_coords().unwrap_or((0, 0, 0));
        let namedtag = s.get_remaining().unwrap_or_default();
        Self { x, y, z, namedtag }
    }

    fn name(&self) -> &'static str { "BlockEntityDataPacket" }
}

pub struct BlockEventPacket {
    pub x: i32,
    pub y: u32,
    pub z: i32,
    pub case_1: i32,
    pub case_2: i32,
}

impl DataPacket for BlockEventPacket {
    const NETWORK_ID: u8 = ProtocolInfo::BLOCK_EVENT_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_block_coords(self.x, self.y, self.z);
        stream.put_var_int(self.case_1);
        stream.put_var_int(self.case_2);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let (x, y, z) = s.get_block_coords().unwrap_or((0, 0, 0));
        let case_1 = s.get_var_int().unwrap_or(0);
        let case_2 = s.get_var_int().unwrap_or(0);
        Self { x, y: y as u32, z, case_1, case_2 }
    }

    fn name(&self) -> &'static str { "BlockEventPacket" }
}

#[derive(Debug, Clone)]
pub struct BlockPickRequestPacket {
    pub x: i32,
    pub y: u32,
    pub z: i32,
    pub add_user_nbt: bool,
}

impl DataPacket for BlockPickRequestPacket {
    const NETWORK_ID: u8 = ProtocolInfo::BLOCK_PICK_REQUEST_PACKET;

    fn encode_payload(&self, _stream: &mut BinaryStream) {}

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let (x, y, z) = s.get_block_coords().unwrap_or((0, 0, 0));
        let add = s.get_bool().unwrap_or(false);
        Self { x, y, z, add_user_nbt: add }
    }

    fn name(&self) -> &'static str { "BlockPickRequestPacket" }
}

pub struct ExplodePacket {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub radius: f32,
    pub records: Vec<(i32, i32, i32)>,
}

impl ExplodePacket {
    pub fn new(x: f32, y: f32, z: f32, radius: f32) -> Self {
        Self { x, y, z, radius, records: Vec::new() }
    }
}

impl DataPacket for ExplodePacket {
    const NETWORK_ID: u8 = ProtocolInfo::EXPLODE_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_vector3f(self.x, self.y, self.z);
        stream.put_var_int((self.radius * 32.0) as i32);
        stream.put_unsigned_var_int(self.records.len() as u32);
        for r in &self.records {
            stream.put_signed_block_coords(r.0, r.1, r.2);
        }
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let (x, y, z) = stream.get_vector3f().unwrap_or((0.0, 0.0, 0.0));
        let radius = (stream.get_var_int().unwrap_or(0) as f32) / 32.0;
        let count = stream.get_unsigned_var_int().unwrap_or(0) as usize;
        let mut records = Vec::with_capacity(count);
        for _ in 0..count {
            if let Ok(c) = stream.get_signed_block_coords() {
                records.push(c);
            }
        }
        Self { x, y, z, radius, records }
    }

    fn name(&self) -> &'static str { "ExplodePacket" }
}

pub struct ChangeDimensionPacket {
    pub dimension: i32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub respawn: bool,
}

impl ChangeDimensionPacket {
    pub fn new(dimension: i32, x: f32, y: f32, z: f32) -> Self {
        Self { dimension, x, y, z, respawn: false }
    }
}

impl DataPacket for ChangeDimensionPacket {
    const NETWORK_ID: u8 = ProtocolInfo::CHANGE_DIMENSION_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_var_int(self.dimension);
        stream.put_vector3f(self.x, self.y, self.z);
        stream.put_bool(self.respawn);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let dimension = stream.get_var_int().unwrap_or(0);
        let (x, y, z) = stream.get_vector3f().unwrap_or((0.0, 0.0, 0.0));
        let respawn = stream.get_bool().unwrap_or(false);
        Self { dimension, x, y, z, respawn }
    }

    fn name(&self) -> &'static str { "ChangeDimensionPacket" }
}

pub struct SetTitlePacket {
    pub title_type: i32,
    pub text: String,
    pub fade_in_duration: i32,
    pub duration: i32,
    pub fade_out_duration: i32,
}

impl SetTitlePacket {
    pub const TYPE_CLEAR: i32 = 0;
    pub const TYPE_RESET: i32 = 1;
    pub const TYPE_TITLE: i32 = 2;
    pub const TYPE_SUB_TITLE: i32 = 3;
    pub const TYPE_ACTION_BAR: i32 = 4;
    pub const TYPE_TIMES: i32 = 5;

    pub fn new(title_type: i32, text: impl Into<String>) -> Self {
        Self { title_type, text: text.into(), fade_in_duration: 0, duration: 0, fade_out_duration: 0 }
    }
}

impl DataPacket for SetTitlePacket {
    const NETWORK_ID: u8 = ProtocolInfo::SET_TITLE_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_var_int(self.title_type);
        stream.put_string(&self.text);
        stream.put_var_int(self.fade_in_duration);
        stream.put_var_int(self.duration);
        stream.put_var_int(self.fade_out_duration);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let title_type = stream.get_var_int().unwrap_or(0);
        let text = stream.get_string().unwrap_or_default();
        let fade_in_duration = stream.get_var_int().unwrap_or(0);
        let duration = stream.get_var_int().unwrap_or(0);
        let fade_out_duration = stream.get_var_int().unwrap_or(0);
        Self { title_type, text, fade_in_duration, duration, fade_out_duration }
    }

    fn name(&self) -> &'static str { "SetTitlePacket" }
}

pub struct PlaySoundPacket {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub volume: f32,
    pub pitch: f32,
}

impl PlaySoundPacket {
    pub fn new(name: impl Into<String>, x: i32, y: i32, z: i32) -> Self {
        Self { name: name.into(), x, y, z, volume: 1.0, pitch: 1.0 }
    }
}

impl DataPacket for PlaySoundPacket {
    const NETWORK_ID: u8 = ProtocolInfo::PLAY_SOUND_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_string(&self.name);
        stream.put_block_coords(self.x, self.y as u32, self.z);
        stream.put_lfloat(self.volume);
        stream.put_lfloat(self.pitch);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let name = stream.get_string().unwrap_or_default();
        let (x, y, z) = stream.get_block_coords().unwrap_or((0, 0, 0));
        let volume = stream.get_lfloat().unwrap_or(1.0);
        let pitch = stream.get_lfloat().unwrap_or(1.0);
        Self { name, x, y: y as i32, z, volume, pitch }
    }

    fn name(&self) -> &'static str { "PlaySoundPacket" }
}

pub struct StopSoundPacket {
    pub name: String,
    pub stop_all: bool,
}

impl StopSoundPacket {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), stop_all: false }
    }
}

impl DataPacket for StopSoundPacket {
    const NETWORK_ID: u8 = ProtocolInfo::STOP_SOUND_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_string(&self.name);
        stream.put_bool(self.stop_all);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let name = s.get_string().unwrap_or_default();
        let stop_all = s.get_bool().unwrap_or(false);
        Self { name, stop_all }
    }
    fn name(&self) -> &'static str { "StopSoundPacket" }
}

pub struct BossEventPacket {
    pub boss_eid: i64,
    pub player_eid: i64,
    pub event_type: u32,
    pub title: String,
    pub health_percent: f32,
    pub unknown_short: u16,
    pub color: u32,
    pub overlay: u32,
}

impl BossEventPacket {
    pub const TYPE_SHOW: u32 = 0;
    pub const TYPE_REGISTER_PLAYER: u32 = 1;
    pub const TYPE_HIDE: u32 = 2;
    pub const TYPE_UNREGISTER_PLAYER: u32 = 3;
    pub const TYPE_HEALTH_PERCENT: u32 = 4;
    pub const TYPE_TITLE: u32 = 5;
    pub const TYPE_UNKNOWN_6: u32 = 6;
    pub const TYPE_TEXTURE: u32 = 7;
}

impl DataPacket for BossEventPacket {
    const NETWORK_ID: u8 = ProtocolInfo::BOSS_EVENT_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_entity_id(self.boss_eid);
        stream.put_unsigned_var_int(self.event_type);
        // Fallthrough cases between SHOW, UNKNOWN_6, and TEXTURE actions
        match self.event_type {
            Self::TYPE_REGISTER_PLAYER | Self::TYPE_UNREGISTER_PLAYER => {
                stream.put_entity_id(self.player_eid);
            }
            Self::TYPE_SHOW | Self::TYPE_UNKNOWN_6 | Self::TYPE_TEXTURE => {
                if self.event_type == Self::TYPE_SHOW {
                    stream.put_string(&self.title);
                    stream.put_lfloat(self.health_percent);
                }
                if self.event_type != Self::TYPE_TEXTURE {
                    stream.put_lshort(self.unknown_short);
                }
                stream.put_unsigned_var_int(self.color);
                stream.put_unsigned_var_int(self.overlay);
            }
            Self::TYPE_HEALTH_PERCENT => {
                stream.put_lfloat(self.health_percent);
            }
            Self::TYPE_TITLE => {
                stream.put_string(&self.title);
            }
            _ => {}
        }
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let boss_eid = s.get_entity_id().unwrap_or(0);
        let event_type = s.get_unsigned_var_int().unwrap_or(0);
        let mut player_eid = 0;
        let mut title = String::new();
        let mut health_percent = 0.0;
        let mut unknown_short = 0;
        let mut color = 0;
        let mut overlay = 0;
        match event_type {
            Self::TYPE_REGISTER_PLAYER | Self::TYPE_UNREGISTER_PLAYER => {
                player_eid = s.get_entity_id().unwrap_or(0);
            }
            Self::TYPE_SHOW | Self::TYPE_UNKNOWN_6 | Self::TYPE_TEXTURE => {
                if event_type == Self::TYPE_SHOW {
                    title = s.get_string().unwrap_or_default();
                    health_percent = s.get_lfloat().unwrap_or(0.0);
                }
                if event_type != Self::TYPE_TEXTURE {
                    unknown_short = s.get_lshort().unwrap_or(0);
                }
                color = s.get_unsigned_var_int().unwrap_or(0);
                overlay = s.get_unsigned_var_int().unwrap_or(0);
            }
            Self::TYPE_HEALTH_PERCENT => {
                health_percent = s.get_lfloat().unwrap_or(0.0);
            }
            Self::TYPE_TITLE => {
                title = s.get_string().unwrap_or_default();
            }
            _ => {}
        }
        Self {
            boss_eid,
            player_eid,
            event_type,
            title,
            health_percent,
            unknown_short,
            color,
            overlay,
        }
    }

    fn name(&self) -> &'static str { "BossEventPacket" }
}

pub struct ShowCreditsPacket {
    pub player_eid: i64,
    pub state: i32,
}

impl DataPacket for ShowCreditsPacket {
    const NETWORK_ID: u8 = ProtocolInfo::SHOW_CREDITS_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_entity_id(self.player_eid);
        stream.put_var_int(self.state);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let player_eid = s.get_entity_id().unwrap_or(0);
        let state = s.get_var_int().unwrap_or(0);
        Self { player_eid, state }
    }

    fn name(&self) -> &'static str { "ShowCreditsPacket" }
}

pub struct GameRulesChangedPacket {
    pub game_rules: crate::protocol::types::GameRules,
}

impl Default for GameRulesChangedPacket {
    fn default() -> Self { Self { game_rules: crate::protocol::types::GameRules::default_rules() } }
}

impl DataPacket for GameRulesChangedPacket {
    const NETWORK_ID: u8 = ProtocolInfo::GAME_RULES_CHANGED_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        self.game_rules.put(stream);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let game_rules = crate::protocol::types::GameRules::get(&mut stream);
        Self { game_rules }
    }

    fn name(&self) -> &'static str { "GameRulesChangedPacket" }
}

pub struct SetCommandsEnabledPacket {
    pub enabled: bool,
}

impl DataPacket for SetCommandsEnabledPacket {
    const NETWORK_ID: u8 = ProtocolInfo::SET_COMMANDS_ENABLED_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_bool(self.enabled);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let enabled = stream.get_bool().unwrap_or(false);
        Self { enabled }
    }
    fn name(&self) -> &'static str { "SetCommandsEnabledPacket" }
}

pub struct HurtArmorPacket {
    pub health: i32,
}

impl DataPacket for HurtArmorPacket {
    const NETWORK_ID: u8 = ProtocolInfo::HURT_ARMOR_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_var_int(self.health);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let health = stream.get_var_int().unwrap_or(0);
        Self { health }
    }
    fn name(&self) -> &'static str { "HurtArmorPacket" }
}

pub struct EntityFallPacket {
    pub entity_runtime_id: i64,
    pub fall_distance: f32,
    pub unknown: bool,
}

impl DataPacket for EntityFallPacket {
    const NETWORK_ID: u8 = ProtocolInfo::ENTITY_FALL_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_entity_id(self.entity_runtime_id);
        stream.put_lfloat(self.fall_distance);
        stream.put_bool(self.unknown);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let entity_runtime_id = s.get_entity_id().unwrap_or(0);
        let fall_distance = s.get_lfloat().unwrap_or(0.0);
        let unknown = s.get_bool().unwrap_or(false);
        Self { entity_runtime_id, fall_distance, unknown }
    }

    fn name(&self) -> &'static str { "EntityFallPacket" }
}

pub struct RiderJumpPacket {
    pub unknown: i32,
}

impl DataPacket for RiderJumpPacket {
    const NETWORK_ID: u8 = ProtocolInfo::RIDER_JUMP_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_var_int(self.unknown);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        Self { unknown: s.get_var_int().unwrap_or(0) }
    }

    fn name(&self) -> &'static str { "RiderJumpPacket" }
}

pub struct AddPaintingPacket {
    pub entity_runtime_id: i64,
    pub x: i32,
    pub y: u32,
    pub z: i32,
    pub direction: i32,
    pub title: String,
}

impl DataPacket for AddPaintingPacket {
    const NETWORK_ID: u8 = ProtocolInfo::ADD_PAINTING_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        // Unique entity ID and runtime entity ID (both share the same ID in MCPE 1.1.5)
        stream.put_entity_id(self.entity_runtime_id);
        stream.put_entity_id(self.entity_runtime_id);
        stream.put_block_coords(self.x, self.y, self.z);
        stream.put_var_int(self.direction);
        stream.put_string(&self.title);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let _unique_id = s.get_entity_id().unwrap_or(0);
        let entity_runtime_id = s.get_entity_id().unwrap_or(0);
        let (x, y, z) = s.get_block_coords().unwrap_or((0, 0, 0));
        let direction = s.get_var_int().unwrap_or(0);
        let title = s.get_string().unwrap_or_default();
        Self { entity_runtime_id, x, y: y as u32, z, direction, title }
    }

    fn name(&self) -> &'static str { "AddPaintingPacket" }
}

#[derive(Debug, Clone)]
pub struct PlayerInputPacket {
    pub motion_x: f32,
    pub motion_y: f32,
    pub jumping: bool,
    pub sneaking: bool,
}

impl DataPacket for PlayerInputPacket {
    const NETWORK_ID: u8 = ProtocolInfo::PLAYER_INPUT_PACKET;

    // Client to server packet only, payload encode is empty
    fn encode_payload(&self, _stream: &mut BinaryStream) {}

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let (x, y, _) = s.get_vector3f().unwrap_or((0.0, 0.0, 0.0));
        let j = s.get_bool().unwrap_or(false);
        let sn = s.get_bool().unwrap_or(false);
        Self { motion_x: x, motion_y: y, jumping: j, sneaking: sn }
    }

    fn name(&self) -> &'static str { "PlayerInputPacket" }
}

pub struct TransferPacket {
    pub ip: String,
    pub port: u16,
}

impl DataPacket for TransferPacket {
    const NETWORK_ID: u8 = ProtocolInfo::TRANSFER_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_string(&self.ip);
        stream.put_lshort(self.port);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let ip = s.get_string().unwrap_or_default();
        let port = s.get_lshort().unwrap_or(0);
        Self { ip, port }
    }
    fn name(&self) -> &'static str { "TransferPacket" }
}

