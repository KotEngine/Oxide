use crate::block::{Block, Material, ToolType};

pub struct GrassBlock;

impl Block for GrassBlock {
    fn id(&self) -> u16 { 2 }
    fn name(&self) -> &'static str { "Grass" }
    fn hardness(&self) -> f32 { 0.6 }
    fn blast_resistance(&self) -> f32 { 3.0 }
    fn tool_type(&self) -> ToolType { ToolType::Shovel }
    fn material(&self) -> Material { Material::Grass }
    fn is_flammable(&self) -> bool { true }
}
