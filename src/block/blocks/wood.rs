use crate::block::{Block, Material, ToolType};

pub struct Wood;

impl Block for Wood {
    fn id(&self) -> u16 { 17 }
    fn name(&self) -> &'static str { "Wood" }
    fn hardness(&self) -> f32 { 2.0 }
    fn blast_resistance(&self) -> f32 { 10.0 }
    fn tool_type(&self) -> ToolType { ToolType::Axe }
    fn material(&self) -> Material { Material::Wood }
    fn is_flammable(&self) -> bool { true }
}
