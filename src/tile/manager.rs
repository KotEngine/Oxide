use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use parking_lot::RwLock;
use crate::protocol::types::Item;
use crate::inventory::crafting::CraftingManager;

#[derive(Debug, Clone)]
pub struct ChestData {
    pub slots: Vec<Item>,
    pub viewers: HashSet<SocketAddr>,
    pub custom_name: Option<String>,
}

impl ChestData {
    pub fn new() -> Self {
        Self {
            slots: vec![Item::air(); 27],
            viewers: HashSet::new(),
            custom_name: None,
        }
    }
}

impl Default for ChestData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct FurnaceData {
    pub input: Item,
    pub fuel: Item,
    pub result: Item,
    pub cook_time: u16,
    pub burn_time: u16,
    pub max_burn_time: u16,
    pub viewers: HashSet<SocketAddr>,
}

impl FurnaceData {
    pub fn new() -> Self {
        Self {
            input: Item::air(),
            fuel: Item::air(),
            result: Item::air(),
            cook_time: 0,
            burn_time: 0,
            max_burn_time: 0,
            viewers: HashSet::new(),
        }
    }

    pub fn to_items(&self) -> Vec<Item> {
        vec![self.input.clone(), self.fuel.clone(), self.result.clone()]
    }
}

impl Default for FurnaceData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct SignData {
    pub lines: [String; 4],
    pub creator: String,
}

impl SignData {
    pub fn new() -> Self {
        Self {
            lines: [String::new(), String::new(), String::new(), String::new()],
            creator: String::new(),
        }
    }
}

impl Default for SignData {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TileManager {
    chests: RwLock<HashMap<(i32, i32, i32), ChestData>>,
    furnaces: RwLock<HashMap<(i32, i32, i32), FurnaceData>>,
    signs: RwLock<HashMap<(i32, i32, i32), SignData>>,
}

impl TileManager {
    pub fn new() -> Self {
        Self {
            chests: RwLock::new(HashMap::new()),
            furnaces: RwLock::new(HashMap::new()),
            signs: RwLock::new(HashMap::new()),
        }
    }

    // --- CHEST INTERACTION ---

    pub fn open_chest(&self, addr: SocketAddr, x: i32, y: i32, z: i32) -> (Vec<Item>, bool) {
        let mut chests = self.chests.write();
        let chest = chests.entry((x, y, z)).or_insert_with(ChestData::new);
        let first_viewer = chest.viewers.is_empty();
        chest.viewers.insert(addr);
        (chest.slots.clone(), first_viewer)
    }

    pub fn close_chest(&self, addr: SocketAddr, x: i32, y: i32, z: i32) -> bool {
        let mut chests = self.chests.write();
        if let Some(chest) = chests.get_mut(&(x, y, z)) {
            chest.viewers.remove(&addr);
            return chest.viewers.is_empty();
        }
        false
    }

    pub fn close_all_for_player(&self, addr: SocketAddr) -> Vec<((i32, i32, i32), bool)> {
        let mut closed_chests = Vec::new();
        let mut chests = self.chests.write();
        for (&pos, chest) in chests.iter_mut() {
            if chest.viewers.remove(&addr) && chest.viewers.is_empty() {
                closed_chests.push((pos, true));
            }
        }
        let mut furnaces = self.furnaces.write();
        for (_, furnace) in furnaces.iter_mut() {
            furnace.viewers.remove(&addr);
        }
        closed_chests
    }

    pub fn set_chest_slot(
        &self,
        x: i32,
        y: i32,
        z: i32,
        slot: usize,
        item: Item,
    ) -> Vec<SocketAddr> {
        let mut chests = self.chests.write();
        if let Some(chest) = chests.get_mut(&(x, y, z)) {
            if slot < chest.slots.len() {
                chest.slots[slot] = item;
                return chest.viewers.iter().copied().collect();
            }
        }
        Vec::new()
    }

    pub fn get_chest_items(&self, x: i32, y: i32, z: i32) -> Option<Vec<Item>> {
        self.chests.read().get(&(x, y, z)).map(|c| c.slots.clone())
    }

    // --- FURNACE INTERACTION ---

    pub fn open_furnace(&self, addr: SocketAddr, x: i32, y: i32, z: i32) -> Vec<Item> {
        let mut furnaces = self.furnaces.write();
        let furnace = furnaces.entry((x, y, z)).or_insert_with(FurnaceData::new);
        furnace.viewers.insert(addr);
        furnace.to_items()
    }

    pub fn close_furnace(&self, addr: SocketAddr, x: i32, y: i32, z: i32) {
        let mut furnaces = self.furnaces.write();
        if let Some(f) = furnaces.get_mut(&(x, y, z)) {
            f.viewers.remove(&addr);
        }
    }

    pub fn set_furnace_slot(&self, x: i32, y: i32, z: i32, slot: usize, item: Item) {
        let mut furnaces = self.furnaces.write();
        if let Some(f) = furnaces.get_mut(&(x, y, z)) {
            match slot {
                0 => f.input = item,
                1 => f.fuel = item,
                2 => f.result = item,
                _ => {}
            }
        }
    }

    pub fn get_furnace_items(&self, x: i32, y: i32, z: i32) -> Option<Vec<Item>> {
        self.furnaces.read().get(&(x, y, z)).map(|f| f.to_items())
    }

    pub fn tick_furnaces(&self, crafting: &CraftingManager) {
        let mut furnaces = self.furnaces.write();
        for (_, f) in furnaces.iter_mut() {
            // If burning fuel, decrement burn timer
            if f.burn_time > 0 {
                f.burn_time -= 1;
            }

            // Check if input can be smelted
            let smelt_result = if !f.input.is_air() {
                crafting.match_furnace(&f.input)
            } else {
                None
            };

            if let Some(res) = smelt_result {
                let can_output = f.result.is_air()
                    || (f.result.id == res.result.id
                        && f.result.meta == res.result.meta
                        && f.result.count < 64);

                if can_output {
                    // Ignite fuel if not burning
                    if f.burn_time == 0 && !f.fuel.is_air() {
                        let burn_duration = fuel_burn_time(f.fuel.id);
                        if burn_duration > 0 {
                            f.burn_time = burn_duration;
                            f.max_burn_time = burn_duration;
                            if f.fuel.count > 1 {
                                f.fuel.count -= 1;
                            } else {
                                f.fuel = Item::air();
                            }
                        }
                    }

                    // Progress cooking if fuel is active
                    if f.burn_time > 0 {
                        f.cook_time += 1;
                        if f.cook_time >= 200 {
                            // Smelt finished (10 seconds)
                            f.cook_time = 0;
                            // Decrement input
                            if f.input.count > 1 {
                                f.input.count -= 1;
                            } else {
                                f.input = Item::air();
                            }
                            // Add result
                            if f.result.is_air() {
                                f.result = res.result.clone();
                            } else {
                                f.result.count += 1;
                            }
                        }
                    } else if f.cook_time > 0 {
                        // Slowly lose cook progress when no fuel
                        f.cook_time = f.cook_time.saturating_sub(2);
                    }
                } else {
                    f.cook_time = 0;
                }
            } else {
                f.cook_time = 0;
            }
        }
    }

    // --- SIGN INTERACTION ---

    pub fn set_sign_text(&self, x: i32, y: i32, z: i32, lines: [String; 4], creator: String) {
        let mut signs = self.signs.write();
        signs.insert((x, y, z), SignData { lines, creator });
    }

    pub fn get_sign_text(&self, x: i32, y: i32, z: i32) -> Option<[String; 4]> {
        self.signs.read().get(&(x, y, z)).map(|s| s.lines.clone())
    }

    /// Build binary NBT for BlockEntityDataPacket
    pub fn build_sign_nbt(&self, x: i32, y: i32, z: i32) -> Vec<u8> {
        let lines = self
            .signs
            .read()
            .get(&(x, y, z))
            .map(|s| s.lines.clone())
            .unwrap_or_else(|| [String::new(), String::new(), String::new(), String::new()]);

        let mut buf = Vec::new();
        // Root compound
        buf.push(0x0a); // TAG_Compound
        buf.push(0x00); // Name length high
        buf.push(0x00); // Name length low (empty root name)

        // TAG_String "id" = "Sign"
        put_nbt_string(&mut buf, "id", "Sign");
        // TAG_Int "x", "y", "z"
        put_nbt_int(&mut buf, "x", x);
        put_nbt_int(&mut buf, "y", y);
        put_nbt_int(&mut buf, "z", z);

        // Lines: "Text1" .. "Text4"
        for (i, line) in lines.iter().enumerate() {
            put_nbt_string(&mut buf, &format!("Text{}", i + 1), line);
        }

        // TAG_End
        buf.push(0x00);
        buf
    }

    // --- REMOVE TILE AND DROP ITEMS ---

    pub fn remove_tile(&self, x: i32, y: i32, z: i32) -> Vec<Item> {
        let mut drops = Vec::new();
        if let Some(chest) = self.chests.write().remove(&(x, y, z)) {
            for item in chest.slots {
                if !item.is_air() {
                    drops.push(item);
                }
            }
        }
        if let Some(furnace) = self.furnaces.write().remove(&(x, y, z)) {
            if !furnace.input.is_air() { drops.push(furnace.input); }
            if !furnace.fuel.is_air() { drops.push(furnace.fuel); }
            if !furnace.result.is_air() { drops.push(furnace.result); }
        }
        self.signs.write().remove(&(x, y, z));
        drops
    }
}

impl Default for TileManager {
    fn default() -> Self {
        Self::new()
    }
}

fn fuel_burn_time(item_id: i32) -> u16 {
    match item_id {
        263 => 1600, // Coal / Charcoal (80s, 8 items)
        17 | 162 => 300, // Logs (15s, 1.5 items)
        5 => 300, // Planks
        280 => 100, // Sticks (5s, 0.5 items)
        325 => 20000, // Lava bucket (1000s, 100 items)
        85 | 188 | 189 | 190 | 191 | 192 => 300, // Fences
        54 | 58 => 300, // Chest, Crafting Table
        6 => 100, // Saplings
        _ => 0,
    }
}

fn put_nbt_string(buf: &mut Vec<u8>, name: &str, val: &str) {
    buf.push(0x08); // TAG_String
    let name_bytes = name.as_bytes();
    buf.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
    buf.extend_from_slice(name_bytes);

    let val_bytes = val.as_bytes();
    buf.extend_from_slice(&(val_bytes.len() as u16).to_le_bytes());
    buf.extend_from_slice(val_bytes);
}

fn put_nbt_int(buf: &mut Vec<u8>, name: &str, val: i32) {
    buf.push(0x03); // TAG_Int
    let name_bytes = name.as_bytes();
    buf.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
    buf.extend_from_slice(name_bytes);
    buf.extend_from_slice(&val.to_le_bytes());
}
