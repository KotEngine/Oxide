use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;
use crate::protocol::types::GameRules;

pub struct StartGamePacket {
    pub entity_unique_id: i64,
    pub entity_runtime_id: i64,
    pub player_gamemode: i32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub seed: i32,
    pub dimension: i32,
    pub generator: i32,
    pub world_gamemode: i32,
    pub difficulty: i32,
    pub spawn_x: i32,
    pub spawn_y: u32,
    pub spawn_z: i32,
    pub has_achievements_disabled: bool,
    pub day_cycle_stop_time: i32,
    pub edu_mode: bool,
    pub rain_level: f32,
    pub lightning_level: f32,
    pub commands_enabled: bool,
    pub is_texture_packs_required: bool,
    pub game_rules: GameRules,
    pub level_id: String,
    pub world_name: String,
    pub premium_world_template_id: String,
    pub unknown_bool: bool,
    pub current_tick: i64,
}

impl StartGamePacket {
    pub fn new(entity_unique_id: i64, entity_runtime_id: i64) -> Self {
        Self {
            entity_unique_id,
            entity_runtime_id,
            player_gamemode: 0,
            x: 0.0, y: 64.0, z: 0.0,
            pitch: 0.0, yaw: 0.0,
            seed: 0,
            dimension: 0,
            generator: 1,
            world_gamemode: 0,
            difficulty: 1,
            spawn_x: 0, spawn_y: 64, spawn_z: 0,
            has_achievements_disabled: true,
            day_cycle_stop_time: -1,
            edu_mode: false,
            rain_level: 0.0,
            lightning_level: 0.0,
            commands_enabled: true,
            is_texture_packs_required: true,
            game_rules: GameRules::default_rules(),
            level_id: String::new(),
            world_name: "Oxide".to_string(),
            premium_world_template_id: String::new(),
            unknown_bool: false,
            current_tick: 0,
        }
    }
}

impl DataPacket for StartGamePacket {
    const NETWORK_ID: u8 = ProtocolInfo::START_GAME_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_entity_id(self.entity_unique_id);
        stream.put_entity_id(self.entity_runtime_id);
        stream.put_var_int(self.player_gamemode);
        stream.put_vector3f(self.x, self.y, self.z);
        stream.put_lfloat(self.pitch);
        stream.put_lfloat(self.yaw);
        stream.put_var_int(self.seed);
        stream.put_var_int(self.dimension);
        stream.put_var_int(self.generator);
        stream.put_var_int(self.world_gamemode);
        stream.put_var_int(self.difficulty);
        stream.put_block_coords(self.spawn_x, self.spawn_y, self.spawn_z);
        stream.put_bool(self.has_achievements_disabled);
        stream.put_var_int(self.day_cycle_stop_time);
        stream.put_bool(self.edu_mode);
        stream.put_lfloat(self.rain_level);
        stream.put_lfloat(self.lightning_level);
        stream.put_bool(self.commands_enabled);
        stream.put_bool(self.is_texture_packs_required);
        self.game_rules.put(stream);
        stream.put_string(&self.level_id);
        stream.put_string(&self.world_name);
        stream.put_string(&self.premium_world_template_id);
        stream.put_bool(self.unknown_bool);
        stream.put_llong(self.current_tick);
    }

    fn decode(_buf: Vec<u8>) -> Self where Self: Sized {
        Self::new(0, 0)
    }

    fn name(&self) -> &'static str { "StartGamePacket" }
}
