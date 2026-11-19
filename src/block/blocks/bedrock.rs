use crate::block::{Block, Material, ToolType};

pub struct Bedrock;

impl Block for Bedrock {
    fn id(&self) -> u16 { 7 }
    fn name(&self) -> &'static str { "Bedrock" }
    fn hardness(&self) -> f32 { -1.0 }
    fn blast_resistance(&self) -> f32 { 6000000.0 }
    fn tool_type(&self) -> ToolType { ToolType::None }
    fn material(&self) -> Material { Material::Rock }
}
