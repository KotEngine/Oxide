use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use super::Item;

pub struct StandardItem {
    pub id: u16,
    pub meta: u8,
    pub name: &'static str,
    pub max_stack_size: u32,
    pub is_tool: bool,
    pub is_armor: bool,
    pub is_food: bool,
    pub tool_type: crate::block::ToolType,
    pub durability: u32,
    pub attack_damage: f32,
    pub mining_speed: f32,
}

impl StandardItem {
    pub fn simple(id: u16, name: &'static str, max_stack: u32) -> Self {
        Self {
            id,
            meta: 0,
            name,
            max_stack_size: max_stack,
            is_tool: false,
            is_armor: false,
            is_food: false,
            tool_type: crate::block::ToolType::None,
            durability: 0,
            attack_damage: 1.0,
            mining_speed: 1.0,
        }
    }

    pub fn tool(id: u16, name: &'static str, tt: crate::block::ToolType, dur: u32, speed: f32, damage: f32) -> Self {
        Self {
            id,
            meta: 0,
            name,
            max_stack_size: 1,
            is_tool: true,
            is_armor: false,
            is_food: false,
            tool_type: tt,
            durability: dur,
            attack_damage: damage,
            mining_speed: speed,
        }
    }

    pub fn armor(id: u16, name: &'static str, dur: u32) -> Self {
        Self {
            id,
            meta: 0,
            name,
            max_stack_size: 1,
            is_tool: false,
            is_armor: true,
            is_food: false,
            tool_type: crate::block::ToolType::None,
            durability: dur,
            attack_damage: 0.0,
            mining_speed: 1.0,
        }
    }

    pub fn food(id: u16, name: &'static str) -> Self {
        Self {
            id,
            meta: 0,
            name,
            max_stack_size: 64,
            is_tool: false,
            is_armor: false,
            is_food: true,
            tool_type: crate::block::ToolType::None,
            durability: 0,
            attack_damage: 1.0,
            mining_speed: 1.0,
        }
    }
}

impl Item for StandardItem {
    fn id(&self) -> u16 { self.id }
    fn meta(&self) -> u8 { self.meta }
    fn name(&self) -> &'static str { self.name }
    fn max_stack_size(&self) -> u32 { self.max_stack_size }
    fn is_tool(&self) -> bool { self.is_tool }
    fn is_armor(&self) -> bool { self.is_armor }
    fn is_food(&self) -> bool { self.is_food }
    fn tool_type(&self) -> crate::block::ToolType { self.tool_type }
    fn durability(&self) -> u32 { self.durability }
    fn attack_damage(&self) -> f32 { self.attack_damage }
    fn mining_speed(&self) -> f32 { self.mining_speed }
}

pub struct ItemRegistry {
    items: RwLock<HashMap<u16, Arc<dyn Item>>>,
}

impl ItemRegistry {
    pub fn new() -> Self {
        Self { items: RwLock::new(HashMap::new()) }
    }
    pub fn register(&self, id: u16, item: Arc<dyn Item>) {
        self.items.write().insert(id, item);
    }
    pub fn get(&self, id: u16) -> Option<Arc<dyn Item>> {
        self.items.read().get(&id).cloned()
    }
    pub fn count(&self) -> usize {
        self.items.read().len()
    }

    pub fn register_defaults(&self) {
        use crate::block::ToolType;

        // Wood tools
        self.register(270, Arc::new(StandardItem::tool(270, "Wooden Pickaxe", ToolType::Pickaxe, 59, 2.0, 2.0)));
        self.register(271, Arc::new(StandardItem::tool(271, "Wooden Axe", ToolType::Axe, 59, 2.0, 3.0)));
        self.register(273, Arc::new(StandardItem::tool(273, "Wooden Shovel", ToolType::Shovel, 59, 2.0, 1.0)));
        self.register(290, Arc::new(StandardItem::tool(290, "Wooden Hoe", ToolType::Hoe, 59, 2.0, 1.0)));
        self.register(268, Arc::new(StandardItem::tool(268, "Wooden Sword", ToolType::Sword, 59, 1.0, 4.0)));

        // Stone tools
        self.register(274, Arc::new(StandardItem::tool(274, "Stone Pickaxe", ToolType::Pickaxe, 131, 4.0, 3.0)));
        self.register(275, Arc::new(StandardItem::tool(275, "Stone Axe", ToolType::Axe, 131, 4.0, 4.0)));
        self.register(277, Arc::new(StandardItem::tool(277, "Stone Shovel", ToolType::Shovel, 131, 4.0, 2.0)));
        self.register(291, Arc::new(StandardItem::tool(291, "Stone Hoe", ToolType::Hoe, 131, 4.0, 1.0)));
        self.register(272, Arc::new(StandardItem::tool(272, "Stone Sword", ToolType::Sword, 131, 1.0, 5.0)));

        // Iron tools
        self.register(257, Arc::new(StandardItem::tool(257, "Iron Pickaxe", ToolType::Pickaxe, 250, 6.0, 4.0)));
        self.register(258, Arc::new(StandardItem::tool(258, "Iron Axe", ToolType::Axe, 250, 6.0, 5.0)));
        self.register(256, Arc::new(StandardItem::tool(256, "Iron Shovel", ToolType::Shovel, 250, 6.0, 3.0)));
        self.register(292, Arc::new(StandardItem::tool(292, "Iron Hoe", ToolType::Hoe, 250, 6.0, 1.0)));
        self.register(267, Arc::new(StandardItem::tool(267, "Iron Sword", ToolType::Sword, 250, 1.0, 6.0)));

        // Diamond tools
        self.register(278, Arc::new(StandardItem::tool(278, "Diamond Pickaxe", ToolType::Pickaxe, 1561, 8.0, 5.0)));
        self.register(279, Arc::new(StandardItem::tool(279, "Diamond Axe", ToolType::Axe, 1561, 8.0, 6.0)));
        self.register(277, Arc::new(StandardItem::tool(277, "Diamond Shovel", ToolType::Shovel, 1561, 8.0, 4.0)));
        self.register(293, Arc::new(StandardItem::tool(293, "Diamond Hoe", ToolType::Hoe, 1561, 8.0, 1.0)));
        self.register(276, Arc::new(StandardItem::tool(276, "Diamond Sword", ToolType::Sword, 1561, 1.0, 7.0)));

        // Gold tools
        self.register(285, Arc::new(StandardItem::tool(285, "Golden Pickaxe", ToolType::Pickaxe, 32, 12.0, 2.0)));
        self.register(286, Arc::new(StandardItem::tool(286, "Golden Axe", ToolType::Axe, 32, 12.0, 3.0)));
        self.register(284, Arc::new(StandardItem::tool(284, "Golden Shovel", ToolType::Shovel, 32, 12.0, 1.0)));
        self.register(294, Arc::new(StandardItem::tool(294, "Golden Hoe", ToolType::Hoe, 32, 12.0, 1.0)));
        self.register(283, Arc::new(StandardItem::tool(283, "Golden Sword", ToolType::Sword, 32, 1.0, 4.0)));

        // Basic materials
        self.register(280, Arc::new(StandardItem::simple(280, "Stick", 64)));
        self.register(263, Arc::new(StandardItem::simple(263, "Coal", 64)));
        self.register(264, Arc::new(StandardItem::simple(264, "Diamond", 64)));
        self.register(265, Arc::new(StandardItem::simple(265, "Iron Ingot", 64)));
        self.register(266, Arc::new(StandardItem::simple(266, "Gold Ingot", 64)));
        self.register(261, Arc::new(StandardItem::simple(261, "Bow", 1)));
        self.register(262, Arc::new(StandardItem::simple(262, "Arrow", 64)));
        self.register(331, Arc::new(StandardItem::simple(331, "Redstone", 64)));
        self.register(388, Arc::new(StandardItem::simple(388, "Emerald", 64)));
        self.register(351, Arc::new(StandardItem::simple(351, "Dye", 64)));

        // Food
        self.register(260, Arc::new(StandardItem::food(260, "Apple")));
        self.register(297, Arc::new(StandardItem::food(297, "Bread")));
        self.register(319, Arc::new(StandardItem::food(319, "Raw Porkchop")));
        self.register(320, Arc::new(StandardItem::food(320, "Cooked Porkchop")));
        self.register(322, Arc::new(StandardItem::food(322, "Golden Apple")));
        self.register(363, Arc::new(StandardItem::food(363, "Raw Beef")));
        self.register(364, Arc::new(StandardItem::food(364, "Steak")));
        self.register(365, Arc::new(StandardItem::food(365, "Raw Chicken")));
        self.register(366, Arc::new(StandardItem::food(366, "Cooked Chicken")));

        // Armor
        self.register(306, Arc::new(StandardItem::armor(306, "Iron Helmet", 165)));
        self.register(307, Arc::new(StandardItem::armor(307, "Iron Chestplate", 240)));
        self.register(308, Arc::new(StandardItem::armor(308, "Iron Leggings", 225)));
        self.register(309, Arc::new(StandardItem::armor(309, "Iron Boots", 195)));

        self.register(310, Arc::new(StandardItem::armor(310, "Diamond Helmet", 363)));
        self.register(311, Arc::new(StandardItem::armor(311, "Diamond Chestplate", 528)));
        self.register(312, Arc::new(StandardItem::armor(312, "Diamond Leggings", 495)));
        self.register(313, Arc::new(StandardItem::armor(313, "Diamond Boots", 429)));
    }
}

impl Default for ItemRegistry {
    fn default() -> Self {
        let reg = Self::new();
        reg.register_defaults();
        reg
    }
}
