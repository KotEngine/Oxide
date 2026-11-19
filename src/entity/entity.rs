use crate::math::{Vec3, AABB};
use crate::protocol::types::EntityMetadata;

pub trait Entity: Send + Sync {
    fn entity_id(&self) -> i64;
    fn position(&self) -> Vec3;
    fn set_position(&mut self, pos: Vec3);
    fn motion(&self) -> Vec3;
    fn set_motion(&mut self, motion: Vec3);
    fn yaw(&self) -> f32;
    fn pitch(&self) -> f32;
    fn set_rotation(&mut self, yaw: f32, pitch: f32);
    fn health(&self) -> f32;
    fn set_health(&mut self, health: f32);
    fn max_health(&self) -> f32 { 20.0 }
    fn is_alive(&self) -> bool { self.health() > 0.0 }
    fn metadata(&self) -> &EntityMetadata;
    fn metadata_mut(&mut self) -> &mut EntityMetadata;
    fn bounding_box(&self) -> AABB {
        let pos = self.position();
        AABB::from_corners(pos.x - 0.3, pos.y, pos.z - 0.3, pos.x + 0.3, pos.y + 1.8, pos.z + 0.3)
    }
    fn entity_type_id(&self) -> i32;
    fn name(&self) -> &str { "Entity" }
}

#[derive(Debug, Clone)]
pub struct EntityData {
    pub id: i64,
    pub type_id: i32,
    pub pos: Vec3,
    pub motion: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub health: f32,
    pub metadata: EntityMetadata,
}

impl Default for EntityData {
    fn default() -> Self {
        Self {
            id: 0,
            type_id: 0,
            pos: Vec3::ZERO,
            motion: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            health: 20.0,
            metadata: EntityMetadata::new(),
        }
    }
}
