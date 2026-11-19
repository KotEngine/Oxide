use crate::block::{Block, Material, ToolType};

macro_rules! ore_block {
    ($name:ident, $id:expr, $str_name:expr, $hard:expr) => {
        pub struct $name;
        impl Block for $name {
            fn id(&self) -> u16 { $id }
            fn name(&self) -> &'static str { $str_name }
            fn hardness(&self) -> f32 { $hard }
            fn blast_resistance(&self) -> f32 { 15.0 }
            fn tool_type(&self) -> ToolType { ToolType::Pickaxe }
            fn material(&self) -> Material { Material::Rock }
        }
    };
}

ore_block!(CoalOre, 16, "Coal Ore", 3.0);
ore_block!(IronOre, 15, "Iron Ore", 3.0);
ore_block!(GoldOre, 14, "Gold Ore", 3.0);
ore_block!(DiamondOre, 56, "Diamond Ore", 3.0);
ore_block!(EmeraldOre, 129, "Emerald Ore", 3.0);
ore_block!(LapisOre, 21, "Lapis Ore", 3.0);
ore_block!(RedstoneOre, 73, "Redstone Ore", 3.0);
ore_block!(QuartzOre, 153, "Quartz Ore", 3.0);
