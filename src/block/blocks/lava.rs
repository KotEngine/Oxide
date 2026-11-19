use crate::block::{Block, Material};

pub struct Lava;

impl Block for Lava {
    fn id(&self) -> u16 { 10 }
    fn name(&self) -> &'static str { "Lava" }
    fn hardness(&self) -> f32 { 100.0 }
    fn blast_resistance(&self) -> f32 { 500.0 }
    fn is_solid(&self) -> bool { false }
    fn is_transparent(&self) -> bool { true }
    fn light_level(&self) -> u8 { 15 }
    fn material(&self) -> Material { Material::Lava }
    fn bounding_box(&self) -> Option<crate::math::AABB> { None }
}
