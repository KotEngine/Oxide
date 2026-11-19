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

// Construction & Minerals
simple_block!(Cobblestone, 4, "Cobblestone", 2.0, Material::Rock);
simple_block!(CobblestoneWall, 139, "Cobblestone Wall", 2.0, Material::Rock);
simple_block!(MossStone, 48, "Moss Stone", 2.0, Material::Rock);
simple_block!(Bricks, 45, "Bricks", 2.0, Material::Rock);
simple_block!(Bookshelf, 47, "Bookshelf", 1.5, Material::Wood);
simple_block!(Obsidian, 49, "Obsidian", 50.0, Material::Rock);
simple_block!(Torch, 50, "Torch", 0.0, Material::Wood);
simple_block!(CraftingTable, 58, "Crafting Table", 2.5, Material::Wood);
simple_block!(Furnace, 61, "Furnace", 3.5, Material::Rock);
simple_block!(BurningFurnace, 62, "Burning Furnace", 3.5, Material::Rock);
simple_block!(BrewingStand, 117, "Brewing Stand", 0.5, Material::Rock);
simple_block!(Cauldron, 118, "Cauldron", 2.0, Material::Iron);
simple_block!(EnchantingTable, 116, "Enchanting Table", 5.0, Material::Rock);
simple_block!(Anvil, 145, "Anvil", 5.0, Material::Iron);
simple_block!(Beacon, 138, "Beacon", 3.0, Material::Glass);
simple_block!(EndPortalFrame, 120, "End Portal Frame", -1.0, Material::Rock);
simple_block!(EndStone, 121, "End Stone", 3.0, Material::Rock);
simple_block!(EndStoneBricks, 206, "End Stone Bricks", 3.0, Material::Rock);
simple_block!(CommandBlock, 137, "Command Block", -1.0, Material::Rock);

// Mineral Storage Blocks
simple_block!(GoldBlock, 41, "Block of Gold", 3.0, Material::Iron);
simple_block!(IronBlock, 42, "Block of Iron", 5.0, Material::Iron);
simple_block!(DiamondBlock, 57, "Block of Diamond", 5.0, Material::Iron);
simple_block!(EmeraldBlock, 133, "Block of Emerald", 5.0, Material::Iron);
simple_block!(LapisBlock, 22, "Lapis Lazuli Block", 3.0, Material::Rock);
simple_block!(RedstoneBlock, 152, "Block of Redstone", 5.0, Material::Iron);
simple_block!(CoalBlock, 173, "Block of Coal", 5.0, Material::Rock);

// Slabs & Decorative
simple_block!(StoneSlab, 44, "Stone Slab", 2.0, Material::Rock);
simple_block!(DoubleStoneSlab, 43, "Double Stone Slab", 2.0, Material::Rock);
simple_block!(WoodSlab, 158, "Wood Slab", 2.0, Material::Wood);
simple_block!(DoubleWoodSlab, 157, "Double Wood Slab", 2.0, Material::Wood);
simple_block!(PurpurSlab, 182, "Purpur Slab", 1.5, Material::Rock);
simple_block!(RedSandstoneSlab, 182, "Red Sandstone Slab", 2.0, Material::Rock);

// Containers & Interactive Blocks
simple_block!(Chest, 54, "Chest", 2.5, Material::Wood);
simple_block!(TrappedChest, 146, "Trapped Chest", 2.5, Material::Wood);
simple_block!(EnderChest, 130, "Ender Chest", 22.5, Material::Rock);
simple_block!(ShulkerBox, 218, "Shulker Box", 2.0, Material::Rock);
simple_block!(Bed, 26, "Bed", 0.2, Material::Cloth);
simple_block!(Ladder, 65, "Ladder", 0.4, Material::Wood);
simple_block!(SignPost, 63, "Sign Post", 1.0, Material::Wood);
simple_block!(WallSign, 68, "Wall Sign", 1.0, Material::Wood);
simple_block!(FlowerPot, 140, "Flower Pot", 0.0, Material::Clay);
simple_block!(ItemFrame, 199, "Item Frame", 0.0, Material::Wood);

// Fences & Gates
simple_block!(Fence, 85, "Oak Fence", 2.0, Material::Wood);
simple_block!(SpruceFence, 188, "Spruce Fence", 2.0, Material::Wood);
simple_block!(BirchFence, 189, "Birch Fence", 2.0, Material::Wood);
simple_block!(JungleFence, 190, "Jungle Fence", 2.0, Material::Wood);
simple_block!(DarkOakFence, 191, "Dark Oak Fence", 2.0, Material::Wood);
simple_block!(AcaciaFence, 192, "Acacia Fence", 2.0, Material::Wood);
simple_block!(NetherBrickFence, 113, "Nether Brick Fence", 2.0, Material::Rock);

simple_block!(FenceGate, 107, "Oak Fence Gate", 2.0, Material::Wood);
simple_block!(SpruceFenceGate, 183, "Spruce Fence Gate", 2.0, Material::Wood);
simple_block!(BirchFenceGate, 184, "Birch Fence Gate", 2.0, Material::Wood);
simple_block!(JungleFenceGate, 185, "Jungle Fence Gate", 2.0, Material::Wood);
simple_block!(DarkOakFenceGate, 186, "Dark Oak Fence Gate", 2.0, Material::Wood);
simple_block!(AcaciaFenceGate, 187, "Acacia Fence Gate", 2.0, Material::Wood);

// Redstone Components
simple_block!(RedstoneWire, 55, "Redstone Wire", 0.0, Material::Air);
simple_block!(RedstoneTorchOff, 75, "Redstone Torch (Off)", 0.0, Material::Wood);
simple_block!(RedstoneTorchOn, 76, "Redstone Torch (On)", 0.0, Material::Wood);
simple_block!(Lever, 69, "Lever", 0.5, Material::Wood);
simple_block!(StonePressurePlate, 70, "Stone Pressure Plate", 0.5, Material::Rock);
simple_block!(WoodenPressurePlate, 72, "Wooden Pressure Plate", 0.5, Material::Wood);
simple_block!(LightWeightedPressurePlate, 147, "Light Weighted Pressure Plate", 0.5, Material::Iron);
simple_block!(HeavyWeightedPressurePlate, 148, "Heavy Weighted Pressure Plate", 0.5, Material::Iron);
simple_block!(StoneButton, 77, "Stone Button", 0.5, Material::Rock);
simple_block!(WoodenButton, 143, "Wooden Button", 0.5, Material::Wood);
simple_block!(RedstoneRepeaterOff, 93, "Redstone Repeater (Off)", 0.0, Material::Rock);
simple_block!(RedstoneRepeaterOn, 94, "Redstone Repeater (On)", 0.0, Material::Rock);
simple_block!(RedstoneComparatorOff, 149, "Redstone Comparator (Off)", 0.0, Material::Rock);
simple_block!(RedstoneComparatorOn, 150, "Redstone Comparator (On)", 0.0, Material::Rock);
simple_block!(DaylightDetector, 151, "Daylight Detector", 0.2, Material::Wood);
simple_block!(InvertedDaylightDetector, 178, "Inverted Daylight Detector", 0.2, Material::Wood);
simple_block!(RedstoneLampOff, 123, "Redstone Lamp (Off)", 0.3, Material::Glass);
simple_block!(RedstoneLampOn, 124, "Redstone Lamp (On)", 0.3, Material::Glass);
simple_block!(Dispenser, 23, "Dispenser", 3.5, Material::Rock);
simple_block!(Dropper, 125, "Dropper", 3.5, Material::Rock);
simple_block!(Hopper, 154, "Hopper", 3.0, Material::Iron);
simple_block!(Observer, 251, "Observer", 3.0, Material::Rock);
simple_block!(TNT, 46, "TNT", 0.0, Material::Explosive);
simple_block!(TripwireHook, 131, "Tripwire Hook", 0.0, Material::Iron);
simple_block!(Tripwire, 132, "Tripwire", 0.0, Material::Cloth);

// Rails
simple_block!(Rail, 66, "Rail", 0.7, Material::Iron);
simple_block!(PoweredRail, 27, "Powered Rail", 0.7, Material::Iron);
simple_block!(DetectorRail, 28, "Detector Rail", 0.7, Material::Iron);
simple_block!(ActivatorRail, 126, "Activator Rail", 0.7, Material::Iron);

// Nature & Farming
simple_block!(Sapling, 6, "Sapling", 0.0, Material::Plant);
simple_block!(Dandelion, 37, "Dandelion", 0.0, Material::Plant);
simple_block!(Poppy, 38, "Poppy", 0.0, Material::Plant);
simple_block!(BrownMushroom, 39, "Brown Mushroom", 0.0, Material::Plant);
simple_block!(RedMushroom, 40, "Red Mushroom", 0.0, Material::Plant);
simple_block!(Cactus, 81, "Cactus", 0.4, Material::Plant);
simple_block!(SugarCane, 83, "Sugar Cane", 0.0, Material::Plant);
simple_block!(Pumpkin, 86, "Pumpkin", 1.0, Material::Plant);
simple_block!(JackOLantern, 91, "Jack o'Lantern", 1.0, Material::Plant);
simple_block!(MelonBlock, 103, "Melon Block", 1.0, Material::Plant);
simple_block!(Vines, 106, "Vines", 0.2, Material::Plant);
simple_block!(WaterLily, 111, "Lily Pad", 0.0, Material::Plant);
simple_block!(Cocoa, 127, "Cocoa", 0.2, Material::Plant);
simple_block!(WheatBlock, 59, "Wheat Crops", 0.0, Material::Plant);
simple_block!(Farmland, 60, "Farmland", 0.6, Material::Dirt);
simple_block!(CarrotBlock, 141, "Carrot Crops", 0.0, Material::Plant);
simple_block!(PotatoBlock, 142, "Potato Crops", 0.0, Material::Plant);
simple_block!(BeetrootBlock, 244, "Beetroot Crops", 0.0, Material::Plant);
simple_block!(DoublePlant, 175, "Double Plant", 0.0, Material::Plant);
simple_block!(GrassPath, 198, "Grass Path", 0.65, Material::Dirt);

// Nether & End Blocks
simple_block!(Netherrack, 87, "Netherrack", 0.4, Material::Rock);
simple_block!(SoulSand, 88, "Soul Sand", 0.5, Material::Sand);
simple_block!(Glowstone, 89, "Glowstone", 0.3, Material::Glass);
simple_block!(NetherPortal, 90, "Nether Portal", -1.0, Material::Air);
simple_block!(NetherBrick, 112, "Nether Brick", 2.0, Material::Rock);
simple_block!(NetherWartBlockCrop, 115, "Nether Wart", 0.0, Material::Plant);
simple_block!(MagmaBlock, 213, "Magma Block", 0.5, Material::Rock);
simple_block!(NetherWartBlock, 214, "Nether Wart Block", 1.0, Material::Plant);
simple_block!(RedNetherBrick, 215, "Red Nether Brick", 2.0, Material::Rock);
simple_block!(BoneBlock, 216, "Bone Block", 2.0, Material::Rock);
simple_block!(PurpurBlock, 201, "Purpur Block", 1.5, Material::Rock);
simple_block!(PurpurPillar, 202, "Purpur Pillar", 1.5, Material::Rock);
simple_block!(EndRod, 208, "End Rod", 0.0, Material::Glass);

// Ice, Snow, Sandstone, Quartz, Prismarine
simple_block!(SnowLayer, 78, "Snow Layer", 0.1, Material::Snow);
simple_block!(Ice, 79, "Ice", 0.5, Material::Ice);
simple_block!(SnowBlock, 80, "Snow Block", 0.2, Material::Snow);
simple_block!(ClayBlock, 82, "Clay Block", 0.6, Material::Clay);
simple_block!(PackedIce, 174, "Packed Ice", 0.5, Material::PackedIce);
simple_block!(FrostedIce, 212, "Frosted Ice", 0.5, Material::Ice);
simple_block!(Sandstone, 24, "Sandstone", 0.8, Material::Rock);
simple_block!(RedSandstone, 179, "Red Sandstone", 0.8, Material::Rock);
simple_block!(QuartzBlock, 155, "Block of Quartz", 0.8, Material::Rock);
simple_block!(Prismarine, 168, "Prismarine", 1.5, Material::Rock);
simple_block!(SeaLantern, 169, "Sea Lantern", 0.3, Material::Glass);
simple_block!(HayBale, 170, "Hay Bale", 0.5, Material::Plant);
simple_block!(SlimeBlock, 165, "Slime Block", 0.0, Material::Cloth);
simple_block!(IronBars, 101, "Iron Bars", 5.0, Material::Iron);
simple_block!(GlassPane, 102, "Glass Pane", 0.3, Material::Glass);
simple_block!(Sponge, 19, "Sponge", 0.6, Material::Sponge);

// Colored Blocks (Wool, Carpet, Stained Clay, Concrete, Glass)
simple_block!(Wool, 35, "Wool", 0.8, Material::Cloth);
simple_block!(Carpet, 171, "Carpet", 0.1, Material::Cloth);
simple_block!(HardenedClay, 172, "Hardened Clay", 1.25, Material::Clay);
simple_block!(StainedClay, 159, "Stained Clay", 1.25, Material::Clay);
simple_block!(StainedGlass, 95, "Stained Glass", 0.3, Material::Glass);
simple_block!(StainedGlassPane, 160, "Stained Glass Pane", 0.3, Material::Glass);
simple_block!(Concrete, 236, "Concrete", 1.8, Material::Rock);
simple_block!(ConcretePowder, 237, "Concrete Powder", 0.5, Material::Sand);
