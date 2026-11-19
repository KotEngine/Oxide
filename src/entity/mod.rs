pub mod entity;
pub mod player;
pub mod living;
pub mod mobs;
pub mod projectile;
pub mod manager;

pub use entity::{Entity, EntityData};
pub use player::Player;
pub use projectile::{ProjectileManager, ProjectileKind};
pub use manager::{EntityManager, ActiveEntity};
