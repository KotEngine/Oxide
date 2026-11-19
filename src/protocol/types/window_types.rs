/// Window types protocol constants
pub struct WindowTypes;

impl WindowTypes {
    pub const INVENTORY: i8 = -1;
    pub const CONTAINER: i8 = 0;
    pub const WORKBENCH: i8 = 1;
    pub const FURNACE: i8 = 2;
    pub const ENCHANTMENT: i8 = 3;
    pub const BREWING_STAND: i8 = 4;
    pub const ANVIL: i8 = 5;
    pub const DISPENSER: i8 = 6;
    pub const DROPPER: i8 = 7;
    pub const HOPPER: i8 = 8;
    pub const CAULDRON: i8 = 9;
    pub const MINECART_CHEST: i8 = 10;
    pub const MINECART_HOPPER: i8 = 11;
    pub const HORSE: i8 = 12;
    pub const BEACON: i8 = 13;
    pub const STRUCTURE_EDITOR: i8 = 14;
    pub const TRADING: i8 = 15;
    pub const COMMAND_BLOCK: i8 = 16;
    pub const JUKEBOX: i8 = 17;
}

/// Container ID protocol constants
pub struct ContainerIds;

impl ContainerIds {
    pub const NONE: i8 = -1;
    pub const INVENTORY: i8 = 0;
    pub const FIRST: i8 = 1;
    pub const LAST: i8 = 100;
    pub const OFFHAND: i8 = 119;
    pub const ARMOR: i8 = 120;
    pub const CREATIVE: i8 = 121;
    pub const HOTBAR: i8 = 122;
    pub const FIXED_INVENTORY: i8 = 123;
}

/// Dimension ID protocol constants
pub struct DimensionIds;

impl DimensionIds {
    pub const OVERWORLD: i32 = 0;
    pub const NETHER: i32 = 1;
    pub const THE_END: i32 = 2;
}
