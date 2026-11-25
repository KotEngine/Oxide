pub mod entity;
pub mod player;
pub mod living;
pub mod mobs;
pub mod projectile;
pub mod manager;
pub mod effect;
pub mod loot;
pub mod spawner;
pub mod damage;
pub mod physics;

pub use entity::{Entity, EntityData};
pub use player::Player;
pub use projectile::{ProjectileManager, ProjectileKind};
pub use manager::{EntityManager, ActiveEntity};
pub use effect::{ActiveEffect, EffectContainer, effect_id};
pub use loot::LootTable;
pub use spawner::MobSpawner;
