use crate::block::{Block, Material, ToolType};

pub struct WoodPlanks;

impl Block for WoodPlanks {
    fn id(&self) -> u16 { 5 }
    fn name(&self) -> &'static str { "Wood Planks" }
    fn hardness(&self) -> f32 { 2.0 }
    fn blast_resistance(&self) -> f32 { 15.0 }
    fn tool_type(&self) -> ToolType { ToolType::Axe }
    fn material(&self) -> Material { Material::Wood }
    fn is_flammable(&self) -> bool { true }
}
