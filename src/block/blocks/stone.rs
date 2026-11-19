use crate::block::{Block, Material, ToolType};

pub struct Stone;

impl Block for Stone {
    fn id(&self) -> u16 { 1 }
    fn name(&self) -> &'static str { "Stone" }
    fn hardness(&self) -> f32 { 1.5 }
    fn blast_resistance(&self) -> f32 { 30.0 }
    fn tool_type(&self) -> ToolType { ToolType::Pickaxe }
    fn material(&self) -> Material { Material::Rock }
}
