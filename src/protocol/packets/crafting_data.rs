use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;
use crate::protocol::types::Item;

pub const ENTRY_SHAPELESS: i32 = 0;
pub const ENTRY_SHAPED: i32 = 1;
pub const ENTRY_FURNACE: i32 = 2;
pub const ENTRY_FURNACE_DATA: i32 = 3;
pub const ENTRY_MULTI: i32 = 4;
pub const ENTRY_SHULKER_BOX: i32 = 5;

/// Single recipe: recipe type and serialized recipe payload.
/// `entry_type < 0`: writes varint(-1) with no payload.
#[derive(Debug, Clone)]
pub struct CraftingDataEntry {
    pub entry_type: i32,
    pub data: Vec<u8>,
}

pub struct CraftingDataPacket {
    pub entries: Vec<CraftingDataEntry>,
    pub clean_recipes: bool,
}

impl Default for CraftingDataPacket {
    fn default() -> Self { Self { entries: Vec::new(), clean_recipes: false } }
}

impl CraftingDataPacket {
    pub fn new() -> Self { Self::default() }

    pub fn add_shapeless(&mut self, ingredients: &[Item], result: Item) {
        let mut stream = BinaryStream::new();
        stream.put_unsigned_var_int(ingredients.len() as u32);
        for item in ingredients {
            item.put(&mut stream);
        }
        stream.put_unsigned_var_int(1);
        result.put(&mut stream);
        stream.put_uuid(crate::utils::UUID::random());

        self.entries.push(CraftingDataEntry {
            entry_type: ENTRY_SHAPELESS,
            data: stream.into_inner(),
        });
    }

    pub fn add_shaped(&mut self, width: i32, height: i32, ingredients: &[Item], result: Item) {
        let mut stream = BinaryStream::new();
        stream.put_var_int(width);
        stream.put_var_int(height);
        let total = (width * height) as usize;
        for i in 0..total {
            if i < ingredients.len() {
                ingredients[i].put(&mut stream);
            } else {
                Item::air().put(&mut stream);
            }
        }
        stream.put_unsigned_var_int(1);
        result.put(&mut stream);
        stream.put_uuid(crate::utils::UUID::random());

        self.entries.push(CraftingDataEntry {
            entry_type: ENTRY_SHAPED,
            data: stream.into_inner(),
        });
    }

    pub fn add_furnace_recipe(&mut self, input_id: i32, input_meta: i32, result: Item) {
        let mut stream = BinaryStream::new();
        stream.put_var_int(input_id);
        stream.put_var_int(input_meta);
        result.put(&mut stream);

        self.entries.push(CraftingDataEntry {
            entry_type: ENTRY_FURNACE_DATA,
            data: stream.into_inner(),
        });
    }

    pub fn with_default_recipes() -> Self {
        let mut pk = Self::new();

        // 1. Wood planks from logs
        for meta in 0..=5 {
            let log_id = if meta >= 4 { 162 } else { 17 };
            let log_meta = if meta >= 4 { meta - 4 } else { meta };
            pk.add_shapeless(&[Item::new(log_id, log_meta, 1)], Item::new(5, meta, 4));
        }

        // 2. Sticks
        pk.add_shaped(1, 2, &[Item::new(5, 0, 1), Item::new(5, 0, 1)], Item::new(280, 0, 4));

        // 3. Crafting Table
        pk.add_shaped(2, 2, &[
            Item::new(5, 0, 1), Item::new(5, 0, 1),
            Item::new(5, 0, 1), Item::new(5, 0, 1),
        ], Item::new(58, 0, 1));

        // 4. Tools (Wood, Stone, Iron, Diamond, Gold)
        let tool_materials = [
            (5, 270, 271, 269, 290, 268),  // Wood
            (4, 274, 275, 273, 291, 272),  // Stone
            (265, 257, 258, 256, 292, 267),// Iron
            (264, 278, 279, 277, 293, 276),// Diamond
            (266, 285, 286, 284, 294, 283),// Gold
        ];

        let stick = Item::new(280, 0, 1);
        for &(mat_id, pick, axe, shovel, hoe, sword) in &tool_materials {
            let m = Item::new(mat_id, 0, 1);
            let air = Item::air();

            // Pickaxe (3x3)
            pk.add_shaped(3, 3, &[
                m.clone(), m.clone(), m.clone(),
                air.clone(), stick.clone(), air.clone(),
                air.clone(), stick.clone(), air.clone(),
            ], Item::new(pick, 0, 1));

            // Axe (3x3)
            pk.add_shaped(3, 3, &[
                m.clone(), m.clone(), air.clone(),
                m.clone(), stick.clone(), air.clone(),
                air.clone(), stick.clone(), air.clone(),
            ], Item::new(axe, 0, 1));

            // Shovel (1x3)
            pk.add_shaped(1, 3, &[
                m.clone(),
                stick.clone(),
                stick.clone(),
            ], Item::new(shovel, 0, 1));

            // Hoe (2x3)
            pk.add_shaped(2, 3, &[
                m.clone(), m.clone(),
                air.clone(), stick.clone(),
                air.clone(), stick.clone(),
            ], Item::new(hoe, 0, 1));

            // Sword (1x3)
            pk.add_shaped(1, 3, &[
                m.clone(),
                m.clone(),
                stick.clone(),
            ], Item::new(sword, 0, 1));
        }

        // 5. Armors (Iron, Diamond, Gold)
        let armor_materials = [
            (265, 306, 307, 308, 309), // Iron
            (264, 310, 311, 312, 313), // Diamond
            (266, 314, 315, 316, 317), // Gold
        ];
        for &(mat_id, helm, chest, legs, boots) in &armor_materials {
            let m = Item::new(mat_id, 0, 1);
            let air = Item::air();

            // Helmet (3x2)
            pk.add_shaped(3, 2, &[
                m.clone(), m.clone(), m.clone(),
                m.clone(), air.clone(), m.clone(),
            ], Item::new(helm, 0, 1));

            // Chestplate (3x3)
            pk.add_shaped(3, 3, &[
                m.clone(), air.clone(), m.clone(),
                m.clone(), m.clone(), m.clone(),
                m.clone(), m.clone(), m.clone(),
            ], Item::new(chest, 0, 1));

            // Leggings (3x3)
            pk.add_shaped(3, 3, &[
                m.clone(), m.clone(), m.clone(),
                m.clone(), air.clone(), m.clone(),
                m.clone(), air.clone(), m.clone(),
            ], Item::new(legs, 0, 1));

            // Boots (3x2)
            pk.add_shaped(3, 2, &[
                m.clone(), air.clone(), m.clone(),
                m.clone(), air.clone(), m.clone(),
            ], Item::new(boots, 0, 1));
        }

        // 6. Common Utilities & Blocks
        let cobble = Item::new(4, 0, 1);
        let plank = Item::new(5, 0, 1);
        let iron = Item::new(265, 0, 1);
        let air = Item::air();

        // Furnace (3x3)
        pk.add_shaped(3, 3, &[
            cobble.clone(), cobble.clone(), cobble.clone(),
            cobble.clone(), air.clone(), cobble.clone(),
            cobble.clone(), cobble.clone(), cobble.clone(),
        ], Item::new(61, 0, 1));

        // Chest (3x3)
        pk.add_shaped(3, 3, &[
            plank.clone(), plank.clone(), plank.clone(),
            plank.clone(), air.clone(), plank.clone(),
            plank.clone(), plank.clone(), plank.clone(),
        ], Item::new(54, 0, 1));

        // Torches (1x2)
        pk.add_shaped(1, 2, &[Item::new(263, 0, 1), stick.clone()], Item::new(50, 0, 4));

        // Redstone Torch (1x2)
        pk.add_shaped(1, 2, &[Item::new(331, 0, 1), stick.clone()], Item::new(76, 0, 1));

        // Lever (1x2)
        pk.add_shaped(1, 2, &[stick.clone(), cobble.clone()], Item::new(69, 0, 1));

        // Wooden Door (2x3)
        pk.add_shaped(2, 3, &[
            plank.clone(), plank.clone(),
            plank.clone(), plank.clone(),
            plank.clone(), plank.clone(),
        ], Item::new(324, 0, 3));

        // Iron Door (2x3)
        pk.add_shaped(2, 3, &[
            iron.clone(), iron.clone(),
            iron.clone(), iron.clone(),
            iron.clone(), iron.clone(),
        ], Item::new(330, 0, 3));

        // Trapdoor (3x2)
        pk.add_shaped(3, 2, &[
            plank.clone(), plank.clone(), plank.clone(),
            plank.clone(), plank.clone(), plank.clone(),
        ], Item::new(96, 0, 2));

        // Fence (3x2)
        pk.add_shaped(3, 2, &[
            plank.clone(), stick.clone(), plank.clone(),
            plank.clone(), stick.clone(), plank.clone(),
        ], Item::new(85, 0, 3));

        // Fence Gate (3x2)
        pk.add_shaped(3, 2, &[
            stick.clone(), plank.clone(), stick.clone(),
            stick.clone(), plank.clone(), stick.clone(),
        ], Item::new(107, 0, 1));

        // Ladder (3x3)
        pk.add_shaped(3, 3, &[
            stick.clone(), air.clone(), stick.clone(),
            stick.clone(), stick.clone(), stick.clone(),
            stick.clone(), air.clone(), stick.clone(),
        ], Item::new(65, 0, 3));

        // Bed (3x2)
        let wool = Item::new(35, 0, 1);
        pk.add_shaped(3, 2, &[
            wool.clone(), wool.clone(), wool.clone(),
            plank.clone(), plank.clone(), plank.clone(),
        ], Item::new(355, 0, 1));

        // Bucket (3x2)
        pk.add_shaped(3, 2, &[
            iron.clone(), air.clone(), iron.clone(),
            air.clone(), iron.clone(), air.clone(),
        ], Item::new(325, 0, 1));

        // Shears (2x2)
        pk.add_shaped(2, 2, &[
            air.clone(), iron.clone(),
            iron.clone(), air.clone(),
        ], Item::new(359, 0, 1));

        // Bow (3x3)
        let string_item = Item::new(287, 0, 1);
        pk.add_shaped(3, 3, &[
            air.clone(), stick.clone(), string_item.clone(),
            stick.clone(), air.clone(), string_item.clone(),
            air.clone(), stick.clone(), string_item.clone(),
        ], Item::new(261, 0, 1));

        // Arrow (1x3)
        pk.add_shaped(1, 3, &[
            Item::new(318, 0, 1), // Flint
            stick.clone(),
            Item::new(288, 0, 1), // Feather
        ], Item::new(262, 0, 4));

        // Flint and Steel (2x2)
        pk.add_shaped(2, 2, &[
            iron.clone(), air.clone(),
            air.clone(), Item::new(318, 0, 1),
        ], Item::new(259, 0, 1));

        // Anvil (3x3)
        let iron_block = Item::new(42, 0, 1);
        pk.add_shaped(3, 3, &[
            iron_block.clone(), iron_block.clone(), iron_block.clone(),
            air.clone(), iron.clone(), air.clone(),
            iron.clone(), iron.clone(), iron.clone(),
        ], Item::new(145, 0, 1));

        // Bread (3x1)
        let wheat = Item::new(296, 0, 1);
        pk.add_shaped(3, 1, &[wheat.clone(), wheat.clone(), wheat.clone()], Item::new(297, 0, 1));

        // Golden Apple (3x3)
        let gold_ingot = Item::new(266, 0, 1);
        pk.add_shaped(3, 3, &[
            gold_ingot.clone(), gold_ingot.clone(), gold_ingot.clone(),
            gold_ingot.clone(), Item::new(260, 0, 1), gold_ingot.clone(),
            gold_ingot.clone(), gold_ingot.clone(), gold_ingot.clone(),
        ], Item::new(322, 0, 1));

        // Mineral Blocks Compression and Decompression
        for (item_id, block_id) in [
            (265, 42),  // Iron
            (266, 41),  // Gold
            (264, 57),  // Diamond
            (388, 133), // Emerald
            (331, 152), // Redstone
            (351, 22),  // Lapis
            (263, 173), // Coal
        ] {
            let it = Item::new(item_id, 0, 1);
            pk.add_shaped(3, 3, &[
                it.clone(), it.clone(), it.clone(),
                it.clone(), it.clone(), it.clone(),
                it.clone(), it.clone(), it.clone(),
            ], Item::new(block_id, 0, 1));
            pk.add_shapeless(&[Item::new(block_id, 0, 1)], Item::new(item_id, 0, 9));
        }

        // 7. Furnace Smelting Recipes
        for &(in_id, in_meta, out_id, out_meta) in &[
            (15, 0, 265, 0),  // Iron Ore -> Iron Ingot
            (14, 0, 266, 0),  // Gold Ore -> Gold Ingot
            (4, 0, 1, 0),     // Cobblestone -> Stone
            (12, 0, 20, 0),   // Sand -> Glass
            (363, 0, 364, 0), // Raw Beef -> Steak
            (319, 0, 320, 0), // Raw Porkchop -> Cooked Porkchop
            (365, 0, 366, 0), // Raw Chicken -> Cooked Chicken
            (392, 0, 393, 0), // Potato -> Baked Potato
            (337, 0, 336, 0), // Clay -> Brick
            (17, 0, 263, 1),  // Log -> Charcoal
            (81, 0, 351, 2),  // Cactus -> Cactus Green
        ] {
            pk.add_furnace_recipe(in_id, in_meta, Item::new(out_id, out_meta, 1));
        }

        pk
    }
}

impl DataPacket for CraftingDataPacket {
    const NETWORK_ID: u8 = ProtocolInfo::CRAFTING_DATA_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_unsigned_var_int(self.entries.len() as u32);
        for entry in &self.entries {
            if entry.entry_type >= 0 {
                stream.put_var_int(entry.entry_type);
                stream.put(&entry.data);
            } else {
                stream.put_var_int(-1);
            }
        }
        stream.put_bool(self.clean_recipes);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let count = s.get_unsigned_var_int().unwrap_or(0).min(4096);
        let mut entries = Vec::with_capacity(count as usize);
        for _ in 0..count {
            if s.feof() { break; }
            let entry_type = s.get_var_int().unwrap_or(-1);
            if entry_type < 0 {
                entries.push(CraftingDataEntry { entry_type, data: Vec::new() });
                continue;
            }
            let start = s.offset;
            match entry_type {
                ENTRY_SHAPELESS | ENTRY_SHULKER_BOX => {
                    let in_cnt = s.get_unsigned_var_int().unwrap_or(0).min(128);
                    for _ in 0..in_cnt {
                        let mut it = crate::protocol::types::Item::air();
                        it.get(&mut s);
                    }
                    let out_cnt = s.get_unsigned_var_int().unwrap_or(0).min(128);
                    for _ in 0..out_cnt {
                        let mut it = crate::protocol::types::Item::air();
                        it.get(&mut s);
                    }
                    let _ = s.get_uuid();
                }
                ENTRY_SHAPED => {
                    let w = s.get_var_int().unwrap_or(0).max(0).min(16);
                    let h = s.get_var_int().unwrap_or(0).max(0).min(16);
                    let total = (w * h) as usize;
                    for _ in 0..total {
                        let mut it = crate::protocol::types::Item::air();
                        it.get(&mut s);
                    }
                    let out_cnt = s.get_unsigned_var_int().unwrap_or(0).min(128);
                    for _ in 0..out_cnt {
                        let mut it = crate::protocol::types::Item::air();
                        it.get(&mut s);
                    }
                    let _ = s.get_uuid();
                }
                ENTRY_FURNACE => {
                    let _ = s.get_var_int();
                    let mut it = crate::protocol::types::Item::air();
                    it.get(&mut s);
                }
                ENTRY_FURNACE_DATA => {
                    let _ = s.get_var_int();
                    let _ = s.get_var_int();
                    let mut it = crate::protocol::types::Item::air();
                    it.get(&mut s);
                }
                ENTRY_MULTI => {
                    let _ = s.get_uuid();
                }
                _ => {}
            }
            let end = s.offset;
            let data = if end > start && end <= s.buffer.len() {
                s.buffer[start..end].to_vec()
            } else {
                Vec::new()
            };
            entries.push(CraftingDataEntry { entry_type, data });
        }
        let clean_recipes = s.get_bool().unwrap_or(false);
        Self { entries, clean_recipes }
    }

    fn name(&self) -> &'static str { "CraftingDataPacket" }
}
