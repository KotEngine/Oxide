use crate::block::{Block, Material};

pub struct Water;

impl Block for Water {
    fn id(&self) -> u16 { 8 }
    fn name(&self) -> &'static str { "Water" }
    fn hardness(&self) -> f32 { 100.0 }
    fn blast_resistance(&self) -> f32 { 500.0 }
    fn is_solid(&self) -> bool { false }
    fn is_transparent(&self) -> bool { true }
    fn material(&self) -> Material { Material::Water }
    fn bounding_box(&self) -> Option<crate::math::AABB> { None }
}
