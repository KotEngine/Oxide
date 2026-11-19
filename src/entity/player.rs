use crate::math::Vec3;
use crate::protocol::types::EntityMetadata;
use crate::utils::UUID;
use super::Entity;

pub struct Player {
    pub username: String,
    pub uuid: UUID,
    pub entity_data: super::EntityData,
    pub gamemode: i32,
    pub on_ground: bool,
    pub address: std::net::SocketAddr,
    pub logged_in: bool,
    pub spawned: bool,
    pub food: f32,
    pub exp: u32,
    pub exp_level: u32,
}

impl Player {
    pub fn new(username: String, uuid: UUID, addr: std::net::SocketAddr) -> Self {
        Self {
            username,
            uuid,
            entity_data: super::EntityData {
                id: 0,
                type_id: 0,
                pos: Vec3::new(0.5, 64.0, 0.5),
                ..Default::default()
            },
            gamemode: 0,
            on_ground: false,
            address: addr,
            logged_in: false,
            spawned: false,
            food: 20.0,
            exp: 0,
            exp_level: 0,
        }
    }
}

impl Entity for Player {
    fn entity_id(&self) -> i64 { self.entity_data.id }
    fn position(&self) -> Vec3 { self.entity_data.pos }
    fn set_position(&mut self, pos: Vec3) { self.entity_data.pos = pos; }
    fn motion(&self) -> Vec3 { self.entity_data.motion }
    fn set_motion(&mut self, motion: Vec3) { self.entity_data.motion = motion; }
    fn yaw(&self) -> f32 { self.entity_data.yaw }
    fn pitch(&self) -> f32 { self.entity_data.pitch }
    fn set_rotation(&mut self, yaw: f32, pitch: f32) {
        self.entity_data.yaw = yaw;
        self.entity_data.pitch = pitch;
    }
    fn health(&self) -> f32 { self.entity_data.health }
    fn set_health(&mut self, health: f32) { self.entity_data.health = health; }
    fn metadata(&self) -> &EntityMetadata { &self.entity_data.metadata }
    fn metadata_mut(&mut self) -> &mut EntityMetadata { &mut self.entity_data.metadata }
    fn entity_type_id(&self) -> i32 { 0 }
    fn name(&self) -> &str { &self.username }
}
