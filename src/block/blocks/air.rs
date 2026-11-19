use crate::block::{Block, Material};

pub struct Air;

impl Block for Air {
    fn id(&self) -> u16 { 0 }
    fn name(&self) -> &'static str { "Air" }
    fn is_solid(&self) -> bool { false }
    fn is_transparent(&self) -> bool { true }
    fn can_be_replaced(&self) -> bool { true }
    fn bounding_box(&self) -> Option<crate::math::AABB> { None }
    fn material(&self) -> Material { Material::Air }
    fn light_level(&self) -> u8 { 0 }
}
