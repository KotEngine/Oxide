use crate::block::{Block, Material};

macro_rules! simple_block {
    ($name:ident, $id:expr, $str_name:expr, $hard:expr, $mat:expr) => {
        pub struct $name;
        impl Block for $name {
            fn id(&self) -> u16 { $id }
            fn name(&self) -> &'static str { $str_name }
            fn hardness(&self) -> f32 { $hard }
            fn material(&self) -> Material { $mat }
        }
    };
}

simple_block!(Cobblestone, 4, "Cobblestone", 2.0, Material::Rock);
simple_block!(CobblestoneWall, 139, "Cobblestone Wall", 2.0, Material::Rock);
simple_block!(MossStone, 48, "Moss Stone", 2.0, Material::Rock);
simple_block!(Bricks, 45, "Bricks", 2.0, Material::Rock);
simple_block!(Bookshelf, 47, "Bookshelf", 1.5, Material::Wood);
simple_block!(Obsidian, 49, "Obsidian", 50.0, Material::Rock);
simple_block!(Torch, 50, "Torch", 0.0, Material::Wood);
simple_block!(CraftingTable, 58, "Crafting Table", 2.5, Material::Wood);
simple_block!(Furnace, 61, "Furnace", 3.5, Material::Rock);
simple_block!(BrewingStand, 117, "Brewing Stand", 0.5, Material::Rock);
simple_block!(Cauldron, 118, "Cauldron", 2.0, Material::Iron);
simple_block!(EnchantingTable, 116, "Enchanting Table", 5.0, Material::Rock);
simple_block!(Anvil, 145, "Anvil", 5.0, Material::Iron);
simple_block!(Beacon, 138, "Beacon", 3.0, Material::Glass);
simple_block!(EndPortalFrame, 120, "End Portal Frame", -1.0, Material::Rock);
simple_block!(EndStone, 121, "End Stone", 3.0, Material::Rock);
simple_block!(CommandBlock, 137, "Command Block", -1.0, Material::Rock);
