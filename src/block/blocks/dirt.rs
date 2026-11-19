use crate::block::{Block, Material, ToolType};

pub struct Dirt;

impl Block for Dirt {
    fn id(&self) -> u16 { 3 }
    fn name(&self) -> &'static str { "Dirt" }
    fn hardness(&self) -> f32 { 0.5 }
    fn blast_resistance(&self) -> f32 { 2.5 }
    fn tool_type(&self) -> ToolType { ToolType::Shovel }
    fn material(&self) -> Material { Material::Dirt }
    fn can_be_replaced(&self) -> bool { false }
}
