pub trait Item: Send + Sync {
    fn id(&self) -> u16;
    fn meta(&self) -> u8 { 0 }
    fn name(&self) -> &'static str;
    fn max_stack_size(&self) -> u32 { 64 }
    fn is_tool(&self) -> bool { false }
    fn is_armor(&self) -> bool { false }
    fn is_food(&self) -> bool { false }
    fn is_block_item(&self) -> bool { false }
    fn tool_type(&self) -> crate::block::ToolType { crate::block::ToolType::None }
    fn durability(&self) -> u32 { 0 }
    fn attack_damage(&self) -> f32 { 1.0 }
    fn mining_speed(&self) -> f32 { 1.0 }
    fn enchantability(&self) -> u32 { 0 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemTier {
    Hand,
    Wood,
    Stone,
    Iron,
    Gold,
    Diamond,
}

impl ItemTier {
    pub fn mining_level(self) -> u32 {
        match self {
            Self::Hand => 0,
            Self::Wood => 0,
            Self::Stone => 1,
            Self::Iron => 2,
            Self::Diamond => 3,
            Self::Gold => 0,
        }
    }
    pub fn durability(self) -> u32 {
        match self {
            Self::Hand => 0,
            Self::Wood => 59,
            Self::Stone => 131,
            Self::Iron => 250,
            Self::Gold => 32,
            Self::Diamond => 1561,
        }
    }
    pub fn mining_speed(self) -> f32 {
        match self {
            Self::Hand => 1.0,
            Self::Wood => 2.0,
            Self::Stone => 4.0,
            Self::Iron => 6.0,
            Self::Gold => 12.0,
            Self::Diamond => 8.0,
        }
    }
    pub fn enchantability(self) -> u32 {
        match self {
            Self::Hand => 0,
            Self::Wood => 15,
            Self::Stone => 5,
            Self::Iron => 14,
            Self::Gold => 22,
            Self::Diamond => 10,
        }
    }
}
