use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

// Level event constants for LevelEventPacket
pub const EVENT_SOUND_CLICK: i32 = 1000;
pub const EVENT_SOUND_CLICK_FAIL: i32 = 1001;
pub const EVENT_SOUND_SHOOT: i32 = 1002;
pub const EVENT_SOUND_DOOR: i32 = 1003;
pub const EVENT_SOUND_FIZZ: i32 = 1004;
pub const EVENT_SOUND_IGNITE: i32 = 1005;
pub const EVENT_SOUND_GHAST: i32 = 1007;
pub const EVENT_SOUND_GHAST_SHOOT: i32 = 1008;
pub const EVENT_SOUND_BLAZE_SHOOT: i32 = 1009;
pub const EVENT_SOUND_DOOR_BUMP: i32 = 1010;
pub const EVENT_SOUND_DOOR_CRASH: i32 = 1012;
pub const EVENT_SOUND_ENDERMAN_TELEPORT: i32 = 1018;
pub const EVENT_SOUND_ANVIL_BREAK: i32 = 1020;
pub const EVENT_SOUND_ANVIL_USE: i32 = 1021;
pub const EVENT_SOUND_ANVIL_FALL: i32 = 1022;
pub const EVENT_SOUND_POP: i32 = 1030;
pub const EVENT_SOUND_PORTAL: i32 = 1032;
pub const EVENT_SOUND_ITEMFRAME_ADD_ITEM: i32 = 1040;
pub const EVENT_SOUND_ITEMFRAME_REMOVE: i32 = 1041;
pub const EVENT_SOUND_ITEMFRAME_PLACE: i32 = 1042;
pub const EVENT_SOUND_ITEMFRAME_REMOVE_ITEM: i32 = 1043;
pub const EVENT_SOUND_ITEMFRAME_ROTATE_ITEM: i32 = 1044;
pub const EVENT_SOUND_CAMERA: i32 = 1050;
pub const EVENT_SOUND_ORB: i32 = 1051;
pub const EVENT_SOUND_TOTEM: i32 = 1052;
pub const EVENT_PARTICLE_SHOOT: i32 = 2000;
pub const EVENT_PARTICLE_DESTROY: i32 = 2001;
pub const EVENT_PARTICLE_SPLASH: i32 = 2002;
pub const EVENT_PARTICLE_EYE_DESPAWN: i32 = 2003;
pub const EVENT_PARTICLE_SPAWN: i32 = 2004;
pub const EVENT_GUARDIAN_CURSE: i32 = 2006;
pub const EVENT_PARTICLE_BLOCK_FORCE_FIELD: i32 = 2008;
pub const EVENT_PARTICLE_PROJECTILE_HIT: i32 = 2009;
pub const EVENT_PARTICLE_ENDERMAN_TELEPORT: i32 = 2013;
pub const EVENT_PARTICLE_PUNCH_BLOCK: i32 = 2014;
pub const EVENT_START_RAIN: i32 = 3001;
pub const EVENT_START_THUNDER: i32 = 3002;
pub const EVENT_STOP_RAIN: i32 = 3003;
pub const EVENT_STOP_THUNDER: i32 = 3004;
pub const EVENT_PAUSE_GAME: i32 = 3005;
pub const EVENT_REDSTONE_TRIGGER: i32 = 3500;
pub const EVENT_CAULDRON_EXPLODE: i32 = 3501;
pub const EVENT_CAULDRON_DYE_ARMOR: i32 = 3502;
pub const EVENT_CAULDRON_CLEAN_ARMOR: i32 = 3503;
pub const EVENT_CAULDRON_FILL_POTION: i32 = 3504;
pub const EVENT_CAULDRON_TAKE_POTION: i32 = 3505;
pub const EVENT_CAULDRON_FILL_WATER: i32 = 3506;
pub const EVENT_CAULDRON_TAKE_WATER: i32 = 3507;
pub const EVENT_CAULDRON_ADD_DYE: i32 = 3508;
pub const EVENT_SET_DATA: i32 = 4000;
pub const EVENT_PLAYERS_SLEEPING: i32 = 9800;
pub const EVENT_ADD_PARTICLE_MASK: i32 = 0x4000;
pub const EVENT_BLOCK_START_BREAK: i32 = 3600;
pub const EVENT_BLOCK_STOP_BREAK: i32 = 3601;

pub struct LevelEventPacket {
    pub evid: i32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub data: i32,
}

impl LevelEventPacket {
    pub fn new(evid: i32, x: f32, y: f32, z: f32, data: i32) -> Self {
        Self { evid, x, y, z, data }
    }
}

impl DataPacket for LevelEventPacket {
    const NETWORK_ID: u8 = ProtocolInfo::LEVEL_EVENT_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_var_int(self.evid);
        stream.put_vector3f(self.x, self.y, self.z);
        stream.put_var_int(self.data);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let evid = s.get_var_int().unwrap_or(0);
        let (x, y, z) = s.get_vector3f().unwrap_or((0.0, 0.0, 0.0));
        let data = s.get_var_int().unwrap_or(0);
        Self { evid, x, y, z, data }
    }

    fn name(&self) -> &'static str { "LevelEventPacket" }
}

// Level sound event constants for LevelSoundEventPacket
pub const SOUND_ITEM_USE_ON: u8 = 0;
pub const SOUND_HIT: u8 = 1;
pub const SOUND_STEP: u8 = 2;
pub const SOUND_JUMP: u8 = 3;
pub const SOUND_BREAK: u8 = 4;
pub const SOUND_PLACE: u8 = 5;
pub const SOUND_HEAVY_STEP: u8 = 6;
pub const SOUND_GALLOP: u8 = 7;
pub const SOUND_FALL: u8 = 8;
pub const SOUND_AMBIENT: u8 = 9;
pub const SOUND_AMBIENT_BABY: u8 = 10;
pub const SOUND_AMBIENT_IN_WATER: u8 = 11;
pub const SOUND_BREATHE: u8 = 12;
pub const SOUND_DEATH: u8 = 13;
pub const SOUND_DEATH_IN_WATER: u8 = 14;
pub const SOUND_DEATH_TO_ZOMBIE: u8 = 15;
pub const SOUND_HURT: u8 = 16;
pub const SOUND_HURT_IN_WATER: u8 = 17;
pub const SOUND_MAD: u8 = 18;
pub const SOUND_BOOST: u8 = 19;
pub const SOUND_BOW: u8 = 20;
pub const SOUND_SQUISH_BIG: u8 = 21;
pub const SOUND_SQUISH_SMALL: u8 = 22;
pub const SOUND_FALL_BIG: u8 = 23;
pub const SOUND_FALL_SMALL: u8 = 24;
pub const SOUND_SPLASH: u8 = 25;
pub const SOUND_FIZZ: u8 = 26;
pub const SOUND_FLAP: u8 = 27;
pub const SOUND_SWIM: u8 = 28;
pub const SOUND_DRINK: u8 = 29;
pub const SOUND_EAT: u8 = 30;
pub const SOUND_TAKEOFF: u8 = 31;
pub const SOUND_SHAKE: u8 = 32;
pub const SOUND_PLOP: u8 = 33;
pub const SOUND_LAND: u8 = 34;
pub const SOUND_SADDLE: u8 = 35;
pub const SOUND_ARMOR: u8 = 36;
pub const SOUND_ADD_CHEST: u8 = 37;
pub const SOUND_THROW: u8 = 38;
pub const SOUND_ATTACK: u8 = 39;
pub const SOUND_ATTACK_NODAMAGE: u8 = 40;
pub const SOUND_WARN: u8 = 41;
pub const SOUND_SHEAR: u8 = 42;
pub const SOUND_MILK: u8 = 43;
pub const SOUND_THUNDER: u8 = 44;
pub const SOUND_EXPLODE: u8 = 45;
pub const SOUND_FIRE: u8 = 46;
pub const SOUND_IGNITE: u8 = 47;
pub const SOUND_FUSE: u8 = 48;
pub const SOUND_STARE: u8 = 49;
pub const SOUND_SPAWN: u8 = 50;
pub const SOUND_SHOOT: u8 = 51;
pub const SOUND_BREAK_BLOCK: u8 = 52;
pub const SOUND_REMEDY: u8 = 53;
pub const SOUND_UNFECT: u8 = 54;
pub const SOUND_LEVELUP: u8 = 55;
pub const SOUND_BOW_HIT: u8 = 56;
pub const SOUND_BULLET_HIT: u8 = 57;
pub const SOUND_EXTINGUISH_FIRE: u8 = 58;
pub const SOUND_ITEM_FIZZ: u8 = 59;
pub const SOUND_CHEST_OPEN: u8 = 60;
pub const SOUND_CHEST_CLOSED: u8 = 61;
pub const SOUND_SHULKERBOX_OPEN: u8 = 62;
pub const SOUND_SHULKERBOX_CLOSED: u8 = 63;
pub const SOUND_POWER_ON: u8 = 64;
pub const SOUND_POWER_OFF: u8 = 65;
pub const SOUND_ATTACH: u8 = 66;
pub const SOUND_DETACH: u8 = 67;
pub const SOUND_DENY: u8 = 68;
pub const SOUND_TRIPOD: u8 = 69;
pub const SOUND_POP: u8 = 70;
pub const SOUND_DROP_SLOT: u8 = 71;
pub const SOUND_NOTE: u8 = 72;
pub const SOUND_THORNS: u8 = 73;
pub const SOUND_PISTON_IN: u8 = 74;
pub const SOUND_PISTON_OUT: u8 = 75;
pub const SOUND_PORTAL: u8 = 76;
pub const SOUND_WATER: u8 = 77;
pub const SOUND_LAVA_POP: u8 = 78;
pub const SOUND_LAVA: u8 = 79;
pub const SOUND_BURP: u8 = 80;
pub const SOUND_BUCKET_FILL_WATER: u8 = 81;
pub const SOUND_BUCKET_FILL_LAVA: u8 = 82;
pub const SOUND_BUCKET_EMPTY_WATER: u8 = 83;
pub const SOUND_BUCKET_EMPTY_LAVA: u8 = 84;
pub const SOUND_GUARDIAN_FLOP: u8 = 85;
pub const SOUND_ELDERGUARDIAN_CURSE: u8 = 86;
pub const SOUND_MOB_WARNING: u8 = 87;
pub const SOUND_MOB_WARNING_BABY: u8 = 88;
pub const SOUND_TELEPORT: u8 = 89;
pub const SOUND_SHULKER_OPEN: u8 = 90;
pub const SOUND_SHULKER_CLOSE: u8 = 91;
pub const SOUND_HAGGLE: u8 = 92;
pub const SOUND_HAGGLE_YES: u8 = 93;
pub const SOUND_HAGGLE_NO: u8 = 94;
pub const SOUND_HAGGLE_IDLE: u8 = 95;
pub const SOUND_CHORUSGROW: u8 = 96;
pub const SOUND_CHORUSDEATH: u8 = 97;
pub const SOUND_GLASS: u8 = 98;
pub const SOUND_CAST_SPELL: u8 = 99;
pub const SOUND_PREPARE_ATTACK: u8 = 100;
pub const SOUND_PREPARE_SUMMON: u8 = 101;
pub const SOUND_PREPARE_WOLOLO: u8 = 102;
pub const SOUND_FANG: u8 = 103;
pub const SOUND_CHARGE: u8 = 104;
pub const SOUND_CAMERA_TAKE_PICTURE: u8 = 105;
pub const SOUND_DEFAULT: u8 = 106;
pub const SOUND_UNDEFINED: u8 = 107;

pub struct LevelSoundEventPacket {
    pub sound: u8,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub extra_data: i32,
    pub pitch: i32,
    pub is_baby_mob: bool,
    pub disable_relative_volume: bool,
}

impl LevelSoundEventPacket {
    pub fn new(sound: u8, x: f32, y: f32, z: f32) -> Self {
        Self {
            sound, x, y, z,
            extra_data: -1,
            pitch: 1,
            is_baby_mob: false,
            disable_relative_volume: false,
        }
    }

    pub fn new_with_extra(sound: u8, x: f32, y: f32, z: f32, extra_data: i32) -> Self {
        Self {
            sound, x, y, z,
            extra_data,
            pitch: 1,
            is_baby_mob: false,
            disable_relative_volume: false,
        }
    }
}

impl DataPacket for LevelSoundEventPacket {
    const NETWORK_ID: u8 = ProtocolInfo::LEVEL_SOUND_EVENT_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_byte(self.sound);
        stream.put_vector3f(self.x, self.y, self.z);
        stream.put_var_int(self.extra_data);
        stream.put_var_int(self.pitch);
        stream.put_bool(self.is_baby_mob);
        stream.put_bool(self.disable_relative_volume);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let sound = s.get_byte().unwrap_or(0);
        let (x, y, z) = s.get_vector3f().unwrap_or((0.0, 0.0, 0.0));
        let extra_data = s.get_var_int().unwrap_or(-1);
        let pitch = s.get_var_int().unwrap_or(1);
        let is_baby_mob = s.get_bool().unwrap_or(false);
        let disable_relative_volume = s.get_bool().unwrap_or(false);
        Self { sound, x, y, z, extra_data, pitch, is_baby_mob, disable_relative_volume }
    }

    fn name(&self) -> &'static str { "LevelSoundEventPacket" }
}
