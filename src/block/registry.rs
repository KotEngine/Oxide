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
        self.register(117, Arc::new(BrewingStand));
        self.register(118, Arc::new(Cauldron));
        self.register(116, Arc::new(EnchantingTable));
        self.register(145, Arc::new(Anvil));
        self.register(138, Arc::new(Beacon));
        self.register(120, Arc::new(EndPortalFrame));
        self.register(121, Arc::new(EndStone));
        self.register(137, Arc::new(CommandBlock));

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
