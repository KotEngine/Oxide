use crate::block::{Block, Material, ToolType};

pub struct Leaves;

impl Block for Leaves {
    fn id(&self) -> u16 { 18 }
    fn name(&self) -> &'static str { "Leaves" }
    fn hardness(&self) -> f32 { 0.2 }
    fn blast_resistance(&self) -> f32 { 1.0 }
    fn tool_type(&self) -> ToolType { ToolType::Shears }
    fn material(&self) -> Material { Material::Leaves }
    fn is_transparent(&self) -> bool { true }
    fn is_flammable(&self) -> bool { true }
}
