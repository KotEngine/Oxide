use crate::block::{Block, Material, ToolType};

pub struct Glass;

impl Block for Glass {
    fn id(&self) -> u16 { 20 }
    fn name(&self) -> &'static str { "Glass" }
    fn hardness(&self) -> f32 { 0.3 }
    fn blast_resistance(&self) -> f32 { 1.5 }
    fn tool_type(&self) -> ToolType { ToolType::None }
    fn material(&self) -> Material { Material::Glass }
    fn is_transparent(&self) -> bool { true }
}
