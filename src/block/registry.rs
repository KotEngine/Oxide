use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use super::Block;

pub struct BlockRegistry {
    blocks: RwLock<HashMap<u16, Arc<dyn Block>>>,
}

impl BlockRegistry {
    pub fn new() -> Self {
        Self { blocks: RwLock::new(HashMap::new()) }
    }

    pub fn register(&self, id: u16, block: Arc<dyn Block>) {
        self.blocks.write().insert(id, block);
    }

    pub fn get(&self, id: u16) -> Option<Arc<dyn Block>> {
        self.blocks.read().get(&id).cloned()
    }

    pub fn count(&self) -> usize {
        self.blocks.read().len()
    }

    pub fn register_defaults(&self) {
        use crate::block::blocks::*;
        use crate::block::blocks::misc::*;
        use crate::block::blocks::ores::*;
        use crate::block::blocks::doors::*;
        use crate::block::blocks::stairs::*;

        self.register(0, Arc::new(Air));
        self.register(1, Arc::new(Stone));
        self.register(2, Arc::new(GrassBlock));
        self.register(3, Arc::new(Dirt));
        self.register(7, Arc::new(Bedrock));
        self.register(12, Arc::new(Sand));
        self.register(17, Arc::new(Wood));
        self.register(18, Arc::new(Leaves));
        self.register(5, Arc::new(WoodPlanks));
        self.register(20, Arc::new(Glass));
        self.register(8, Arc::new(Water));
        self.register(9, Arc::new(Water));
        self.register(10, Arc::new(Lava));
        self.register(11, Arc::new(Lava));

        // Misc
        self.register(4, Arc::new(Cobblestone));
        self.register(139, Arc::new(CobblestoneWall));
        self.register(48, Arc::new(MossStone));
        self.register(45, Arc::new(Bricks));
        self.register(47, Arc::new(Bookshelf));
        self.register(49, Arc::new(Obsidian));
        self.register(50, Arc::new(Torch));
        self.register(58, Arc::new(CraftingTable));
        self.register(61, Arc::new(Furnace));
        self.register(62, Arc::new(BurningFurnace));
        self.register(117, Arc::new(BrewingStand));
        self.register(118, Arc::new(Cauldron));
        self.register(116, Arc::new(EnchantingTable));
        self.register(145, Arc::new(Anvil));
        self.register(138, Arc::new(Beacon));
        self.register(120, Arc::new(EndPortalFrame));
        self.register(121, Arc::new(EndStone));
        self.register(206, Arc::new(EndStoneBricks));
        self.register(137, Arc::new(CommandBlock));

        // Mineral Storage
        self.register(41, Arc::new(GoldBlock));
        self.register(42, Arc::new(IronBlock));
        self.register(57, Arc::new(DiamondBlock));
        self.register(133, Arc::new(EmeraldBlock));
        self.register(22, Arc::new(LapisBlock));
        self.register(152, Arc::new(RedstoneBlock));
        self.register(173, Arc::new(CoalBlock));

        // Slabs
        self.register(44, Arc::new(StoneSlab));
        self.register(43, Arc::new(DoubleStoneSlab));
        self.register(158, Arc::new(WoodSlab));
        self.register(157, Arc::new(DoubleWoodSlab));
        self.register(182, Arc::new(RedSandstoneSlab));

        // Containers & Interactive
        self.register(54, Arc::new(Chest));
        self.register(146, Arc::new(TrappedChest));
        self.register(130, Arc::new(EnderChest));
        self.register(218, Arc::new(ShulkerBox));
        self.register(26, Arc::new(Bed));
        self.register(65, Arc::new(Ladder));
        self.register(63, Arc::new(SignPost));
        self.register(68, Arc::new(WallSign));
        self.register(140, Arc::new(FlowerPot));
        self.register(199, Arc::new(ItemFrame));

        // Fences & Gates
        self.register(85, Arc::new(Fence));
        self.register(188, Arc::new(SpruceFence));
        self.register(189, Arc::new(BirchFence));
        self.register(190, Arc::new(JungleFence));
        self.register(191, Arc::new(DarkOakFence));
        self.register(192, Arc::new(AcaciaFence));
        self.register(113, Arc::new(NetherBrickFence));
        self.register(107, Arc::new(FenceGate));
        self.register(183, Arc::new(SpruceFenceGate));
        self.register(184, Arc::new(BirchFenceGate));
        self.register(185, Arc::new(JungleFenceGate));
        self.register(186, Arc::new(DarkOakFenceGate));
        self.register(187, Arc::new(AcaciaFenceGate));

        // Redstone
        self.register(55, Arc::new(RedstoneWire));
        self.register(75, Arc::new(RedstoneTorchOff));
        self.register(76, Arc::new(RedstoneTorchOn));
        self.register(69, Arc::new(Lever));
        self.register(70, Arc::new(StonePressurePlate));
        self.register(72, Arc::new(WoodenPressurePlate));
        self.register(147, Arc::new(LightWeightedPressurePlate));
        self.register(148, Arc::new(HeavyWeightedPressurePlate));
        self.register(77, Arc::new(StoneButton));
        self.register(143, Arc::new(WoodenButton));
        self.register(93, Arc::new(RedstoneRepeaterOff));
        self.register(94, Arc::new(RedstoneRepeaterOn));
        self.register(149, Arc::new(RedstoneComparatorOff));
        self.register(150, Arc::new(RedstoneComparatorOn));
        self.register(151, Arc::new(DaylightDetector));
        self.register(178, Arc::new(InvertedDaylightDetector));
        self.register(123, Arc::new(RedstoneLampOff));
        self.register(124, Arc::new(RedstoneLampOn));
        self.register(23, Arc::new(Dispenser));
        self.register(125, Arc::new(Dropper));
        self.register(154, Arc::new(Hopper));
        self.register(251, Arc::new(Observer));
        self.register(46, Arc::new(TNT));
        self.register(131, Arc::new(TripwireHook));
        self.register(132, Arc::new(Tripwire));

        // Rails
        self.register(66, Arc::new(Rail));
        self.register(27, Arc::new(PoweredRail));
        self.register(28, Arc::new(DetectorRail));
        self.register(126, Arc::new(ActivatorRail));

        // Nature & Farming
        self.register(6, Arc::new(Sapling));
        self.register(37, Arc::new(Dandelion));
        self.register(38, Arc::new(Poppy));
        self.register(39, Arc::new(BrownMushroom));
        self.register(40, Arc::new(RedMushroom));
        self.register(81, Arc::new(Cactus));
        self.register(83, Arc::new(SugarCane));
        self.register(86, Arc::new(Pumpkin));
        self.register(91, Arc::new(JackOLantern));
        self.register(103, Arc::new(MelonBlock));
        self.register(106, Arc::new(Vines));
        self.register(111, Arc::new(WaterLily));
        self.register(127, Arc::new(Cocoa));
        self.register(59, Arc::new(WheatBlock));
        self.register(60, Arc::new(Farmland));
        self.register(141, Arc::new(CarrotBlock));
        self.register(142, Arc::new(PotatoBlock));
        self.register(244, Arc::new(BeetrootBlock));
        self.register(175, Arc::new(DoublePlant));
        self.register(198, Arc::new(GrassPath));

        // Nether & End
        self.register(87, Arc::new(Netherrack));
        self.register(88, Arc::new(SoulSand));
        self.register(89, Arc::new(Glowstone));
        self.register(90, Arc::new(NetherPortal));
        self.register(112, Arc::new(NetherBrick));
        self.register(115, Arc::new(NetherWartBlockCrop));
        self.register(213, Arc::new(MagmaBlock));
        self.register(214, Arc::new(NetherWartBlock));
        self.register(215, Arc::new(RedNetherBrick));
        self.register(216, Arc::new(BoneBlock));
        self.register(201, Arc::new(PurpurBlock));
        self.register(202, Arc::new(PurpurPillar));
        self.register(208, Arc::new(EndRod));

        // Ice, Sandstone, Prismarine, etc.
        self.register(78, Arc::new(SnowLayer));
        self.register(79, Arc::new(Ice));
        self.register(80, Arc::new(SnowBlock));
        self.register(82, Arc::new(ClayBlock));
        self.register(174, Arc::new(PackedIce));
        self.register(212, Arc::new(FrostedIce));
        self.register(24, Arc::new(Sandstone));
        self.register(179, Arc::new(RedSandstone));
        self.register(155, Arc::new(QuartzBlock));
        self.register(168, Arc::new(Prismarine));
        self.register(169, Arc::new(SeaLantern));
        self.register(170, Arc::new(HayBale));
        self.register(165, Arc::new(SlimeBlock));
        self.register(101, Arc::new(IronBars));
        self.register(102, Arc::new(GlassPane));
        self.register(19, Arc::new(Sponge));

        // Colored Blocks
        self.register(35, Arc::new(Wool));
        self.register(171, Arc::new(Carpet));
        self.register(172, Arc::new(HardenedClay));
        self.register(159, Arc::new(StainedClay));
        self.register(95, Arc::new(StainedGlass));
        self.register(160, Arc::new(StainedGlassPane));
        self.register(236, Arc::new(Concrete));
        self.register(237, Arc::new(ConcretePowder));

        // Ores
        self.register(16, Arc::new(CoalOre));
        self.register(15, Arc::new(IronOre));
        self.register(14, Arc::new(GoldOre));
        self.register(56, Arc::new(DiamondOre));
        self.register(129, Arc::new(EmeraldOre));
        self.register(21, Arc::new(LapisOre));
        self.register(73, Arc::new(RedstoneOre));
        self.register(153, Arc::new(QuartzOre));

        // Doors
        self.register(64, Arc::new(oak_door()));
        self.register(71, Arc::new(iron_door()));
        self.register(193, Arc::new(spruce_door()));
        self.register(194, Arc::new(birch_door()));
        self.register(195, Arc::new(jungle_door()));
        self.register(196, Arc::new(acacia_door()));
        self.register(197, Arc::new(dark_oak_door()));

        // Stairs
        self.register(53, Arc::new(oak_stairs()));
        self.register(67, Arc::new(stone_stairs()));
        self.register(108, Arc::new(brick_stairs()));
        self.register(114, Arc::new(nether_brick_stairs()));
        self.register(128, Arc::new(sandstone_stairs()));
        self.register(156, Arc::new(quartz_stairs()));
        self.register(234, Arc::new(purpur_stairs()));
    }
}

impl Default for BlockRegistry {
    fn default() -> Self {
        let reg = Self::new();
        reg.register_defaults();
        reg
    }
}
