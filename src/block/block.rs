use crate::math::AABB;

pub trait Block: Send + Sync {
    fn id(&self) -> u16;
    fn meta(&self) -> u8 { 0 }
    fn name(&self) -> &'static str;
    fn hardness(&self) -> f32 { 0.0 }
    fn blast_resistance(&self) -> f32 { 0.0 }
    fn is_solid(&self) -> bool { true }
    fn is_transparent(&self) -> bool { false }
    fn is_flammable(&self) -> bool { false }
    fn light_level(&self) -> u8 { 0 }
    fn can_be_replaced(&self) -> bool { false }
    fn bounding_box(&self) -> Option<AABB> {
        if self.is_solid() {
            Some(AABB::from_corners(0.0, 0.0, 0.0, 1.0, 1.0, 1.0))
        } else {
            None
        }
    }
    fn tool_type(&self) -> ToolType { ToolType::None }
    fn material(&self) -> Material { Material::Rock }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolType {
    None,
    Pickaxe,
    Axe,
    Shovel,
    Hoe,
    Sword,
    Shears,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Material {
    Air,
    Grass,
    Dirt,
    Wood,
    Rock,
    Iron,
    Water,
    Lava,
    Leaves,
    Plant,
    Sponge,
    Cloth,
    Sand,
    Glass,
    Explosive,
    Ice,
    PackedIce,
    Snow,
    Clay,
    Cake,
    Quartz,
}

impl Material {
    pub fn is_solid(self) -> bool {
        !matches!(self, Self::Air | Self::Water | Self::Lava)
    }
    pub fn is_liquid(self) -> bool {
        matches!(self, Self::Water | Self::Lava)
    }
}
