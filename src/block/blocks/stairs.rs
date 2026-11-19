use crate::block::{Block, Material, ToolType};
use crate::math::AABB;

pub struct Stairs {
    pub id: u16,
    pub name: &'static str,
}

impl Block for Stairs {
    fn id(&self) -> u16 { self.id }
    fn name(&self) -> &'static str { self.name }
    fn hardness(&self) -> f32 { 2.0 }
    fn blast_resistance(&self) -> f32 { 15.0 }
    fn tool_type(&self) -> ToolType { ToolType::Pickaxe }
    fn material(&self) -> Material { Material::Rock }
    fn bounding_box(&self) -> Option<AABB> { Some(AABB::from_corners(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)) }
}

pub fn oak_stairs() -> Stairs { Stairs { id: 53, name: "Oak Stairs" } }
pub fn stone_stairs() -> Stairs { Stairs { id: 67, name: "Stone Stairs" } }
pub fn cobblestone_stairs() -> Stairs { Stairs { id: 4, name: "Cobblestone Stairs" } }
pub fn brick_stairs() -> Stairs { Stairs { id: 108, name: "Brick Stairs" } }
pub fn nether_brick_stairs() -> Stairs { Stairs { id: 114, name: "Nether Brick Stairs" } }
pub fn sandstone_stairs() -> Stairs { Stairs { id: 128, name: "Sandstone Stairs" } }
pub fn quartz_stairs() -> Stairs { Stairs { id: 156, name: "Quartz Stairs" } }
pub fn purpur_stairs() -> Stairs { Stairs { id: 234, name: "Purpur Stairs" } }
