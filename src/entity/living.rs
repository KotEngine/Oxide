use crate::entity::Entity;

pub trait Living: Entity {
    fn armor(&self) -> &[i32; 4];
    fn attack_damage(&self) -> f32 { 1.0 }
    fn can_attack(&self) -> bool { true }
}
