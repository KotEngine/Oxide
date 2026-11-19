use crate::block::{Block, Material, ToolType};

pub struct Sand;

impl Block for Sand {
    fn id(&self) -> u16 { 12 }
    fn name(&self) -> &'static str { "Sand" }
    fn hardness(&self) -> f32 { 0.5 }
    fn blast_resistance(&self) -> f32 { 2.5 }
    fn tool_type(&self) -> ToolType { ToolType::Shovel }
    fn material(&self) -> Material { Material::Sand }
}
