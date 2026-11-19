use crate::block::{Block, Material, ToolType};

pub struct Door {
    pub id: u16,
    pub name: &'static str,
}

impl Block for Door {
    fn id(&self) -> u16 { self.id }
    fn name(&self) -> &'static str { self.name }
    fn hardness(&self) -> f32 { 1.0 }
    fn blast_resistance(&self) -> f32 { 5.0 }
    fn tool_type(&self) -> ToolType { ToolType::Axe }
    fn material(&self) -> Material { Material::Wood }
    fn is_solid(&self) -> bool { false }
}

pub fn oak_door() -> Door { Door { id: 64, name: "Oak Door" } }
pub fn iron_door() -> Door { Door { id: 71, name: "Iron Door" } }
pub fn spruce_door() -> Door { Door { id: 193, name: "Spruce Door" } }
pub fn birch_door() -> Door { Door { id: 194, name: "Birch Door" } }
pub fn jungle_door() -> Door { Door { id: 195, name: "Jungle Door" } }
pub fn acacia_door() -> Door { Door { id: 196, name: "Acacia Door" } }
pub fn dark_oak_door() -> Door { Door { id: 197, name: "Dark Oak Door" } }
