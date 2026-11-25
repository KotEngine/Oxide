use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use parking_lot::RwLock;
use crate::protocol::types::Item;
use crate::inventory::crafting::CraftingManager;
use crate::utils::BinaryStream;

pub type TileKey = (i32, i32, i32, i32); // (dimension, x, y, z)

#[derive(Debug, Clone)]
pub struct ChestData {
    pub slots: Vec<Item>,
    pub viewers: HashSet<SocketAddr>,
    pub custom_name: Option<String>,
    pub paired: Option<(i32, i32, i32)>,
    pub is_trapped: bool,
}

impl ChestData {
    pub fn new() -> Self {
        Self {
            slots: vec![Item::air(); 27],
            viewers: HashSet::new(),
            custom_name: None,
            paired: None,
            is_trapped: false,
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
    pub stored_xp: f32,
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
            stored_xp: 0.0,
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
pub struct BrewingData {
    pub ingredient: Item,
    pub bottles: [Item; 3],
    pub brew_time: u16,
    pub viewers: HashSet<SocketAddr>,
}

impl BrewingData {
    pub const MAX_BREW_TIME: u16 = 400;

    pub fn new() -> Self {
        Self {
            ingredient: Item::air(),
            bottles: [Item::air(), Item::air(), Item::air()],
            brew_time: Self::MAX_BREW_TIME,
            viewers: HashSet::new(),
        }
    }

    pub fn to_items(&self) -> Vec<Item> {
        vec![
            self.ingredient.clone(),
            self.bottles[0].clone(),
            self.bottles[1].clone(),
            self.bottles[2].clone(),
        ]
    }
}

impl Default for BrewingData {
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

#[derive(Debug, Clone)]
pub struct TempWindow {
    pub window_type: u8,
    pub dim: i32,
    pub pos: (i32, i32, i32),
    pub slots: Vec<Item>,
}

#[derive(Debug, Default)]
pub struct FurnaceTickResult {
    pub block_toggles: Vec<(i32, i32, i32, i32, bool)>,
    pub data_updates: Vec<(SocketAddr, i32, i32)>,
    pub slot_syncs: Vec<(SocketAddr, usize, Item)>,
}

#[derive(Debug, Default)]
pub struct BrewingTickResult {
    pub data_updates: Vec<(SocketAddr, i32, i32)>,
    pub slot_syncs: Vec<(SocketAddr, usize, Item)>,
}

pub struct TileManager {
    chests: RwLock<HashMap<TileKey, ChestData>>,
    furnaces: RwLock<HashMap<TileKey, FurnaceData>>,
    brewing_stands: RwLock<HashMap<TileKey, BrewingData>>,
    signs: RwLock<HashMap<TileKey, SignData>>,
    beds: RwLock<HashMap<TileKey, u8>>,
    enchants: RwLock<HashSet<TileKey>>,
    temp_windows: RwLock<HashMap<SocketAddr, TempWindow>>,
}

fn order_chest_pair(a: (i32, i32, i32), b: (i32, i32, i32)) -> ((i32, i32, i32), (i32, i32, i32)) {
    let score_a = a.0 as i64 + ((a.2 as i64) << 15);
    let score_b = b.0 as i64 + ((b.2 as i64) << 15);
    if score_b > score_a {
        (b, a)
    } else {
        (a, b)
    }
}

impl TileManager {
    pub fn new() -> Self {
        Self {
            chests: RwLock::new(HashMap::new()),
            furnaces: RwLock::new(HashMap::new()),
            brewing_stands: RwLock::new(HashMap::new()),
            signs: RwLock::new(HashMap::new()),
            beds: RwLock::new(HashMap::new()),
            enchants: RwLock::new(HashSet::new()),
            temp_windows: RwLock::new(HashMap::new()),
        }
    }

    // --- CHEST INTERACTION ---

    pub fn try_pair_chest_in_dim(
        &self,
        dim: i32,
        x: i32,
        y: i32,
        z: i32,
        is_trapped: bool,
    ) -> Option<(i32, i32, i32)> {
        let mut chests = self.chests.write();
        let entry = chests.entry((dim, x, y, z)).or_insert_with(ChestData::new);
        entry.is_trapped = is_trapped;
        if entry.paired.is_some() {
            return entry.paired;
        }
        let candidates = [(x + 1, y, z), (x - 1, y, z), (x, y, z + 1), (x, y, z - 1)];
        let mut partner_pos = None;
        for c in candidates {
            if let Some(other) = chests.get(&(dim, c.0, c.1, c.2)) {
                if other.paired.is_none() && other.is_trapped == is_trapped {
                    partner_pos = Some(c);
                    break;
                }
            }
        }
        if let Some(p) = partner_pos {
            if let Some(c1) = chests.get_mut(&(dim, x, y, z)) {
                c1.paired = Some(p);
            }
            if let Some(c2) = chests.get_mut(&(dim, p.0, p.1, p.2)) {
                c2.paired = Some((x, y, z));
            }
        }
        partner_pos
    }

    pub fn open_chest(&self, addr: SocketAddr, x: i32, y: i32, z: i32) -> (Vec<Item>, bool) {
        self.open_chest_in_dim(addr, 0, x, y, z)
    }

    pub fn open_chest_in_dim(
        &self,
        addr: SocketAddr,
        dim: i32,
        x: i32,
        y: i32,
        z: i32,
    ) -> (Vec<Item>, bool) {
        let mut chests = self.chests.write();
        let paired = chests
            .entry((dim, x, y, z))
            .or_insert_with(ChestData::new)
            .paired;

        if let Some(partner) = paired {
            if chests.contains_key(&(dim, partner.0, partner.1, partner.2)) {
                let (first_pos, second_pos) = order_chest_pair((x, y, z), partner);
                let mut combined = Vec::with_capacity(54);
                let first_empty;
                let second_empty;
                {
                    let c1 = chests.get_mut(&(dim, first_pos.0, first_pos.1, first_pos.2)).unwrap();
                    first_empty = c1.viewers.is_empty();
                    c1.viewers.insert(addr);
                    combined.extend_from_slice(&c1.slots);
                }
                {
                    let c2 = chests.get_mut(&(dim, second_pos.0, second_pos.1, second_pos.2)).unwrap();
                    second_empty = c2.viewers.is_empty();
                    c2.viewers.insert(addr);
                    combined.extend_from_slice(&c2.slots);
                }
                return (combined, first_empty && second_empty);
            }
        }

        let chest = chests.entry((dim, x, y, z)).or_insert_with(ChestData::new);
        let first_viewer = chest.viewers.is_empty();
        chest.viewers.insert(addr);
        (chest.slots.clone(), first_viewer)
    }

    pub fn set_chest_slot(&self, x: i32, y: i32, z: i32, slot: usize, item: Item) -> Vec<SocketAddr> {
        self.set_chest_slot_in_dim(0, x, y, z, slot, item)
    }

    pub fn set_chest_slot_in_dim(
        &self,
        dim: i32,
        x: i32,
        y: i32,
        z: i32,
        slot: usize,
        item: Item,
    ) -> Vec<SocketAddr> {
        let mut chests = self.chests.write();
        let paired = match chests.get(&(dim, x, y, z)) {
            Some(c) => c.paired,
            None => return Vec::new(),
        };

        if let Some(partner) = paired {
            if chests.contains_key(&(dim, partner.0, partner.1, partner.2)) && slot < 54 {
                let (first_pos, second_pos) = order_chest_pair((x, y, z), partner);
                let target_pos = if slot < 27 { first_pos } else { second_pos };
                let target_idx = slot % 27;
                if let Some(c) = chests.get_mut(&(dim, target_pos.0, target_pos.1, target_pos.2)) {
                    c.slots[target_idx] = item;
                }
                let mut viewers = HashSet::new();
                if let Some(c1) = chests.get(&(dim, first_pos.0, first_pos.1, first_pos.2)) {
                    viewers.extend(&c1.viewers);
                }
                if let Some(c2) = chests.get(&(dim, second_pos.0, second_pos.1, second_pos.2)) {
                    viewers.extend(&c2.viewers);
                }
                return viewers.into_iter().collect();
            }
        }

        if let Some(chest) = chests.get_mut(&(dim, x, y, z)) {
            if slot < chest.slots.len() {
                chest.slots[slot] = item;
                return chest.viewers.iter().copied().collect();
            }
        }
        Vec::new()
    }

    pub fn close_all_for_player(&self, addr: SocketAddr) -> Vec<((i32, i32, i32), bool)> {
        let mut closed_chests = Vec::new();
        {
            let mut chests = self.chests.write();
            let mut visited_pairs = HashSet::new();
            let keys: Vec<TileKey> = chests.keys().copied().collect();
            for key in keys {
                let mut removed = false;
                let mut is_empty = false;
                let mut paired = None;
                if let Some(c) = chests.get_mut(&key) {
                    if c.viewers.remove(&addr) {
                        removed = true;
                        is_empty = c.viewers.is_empty();
                        paired = c.paired;
                    }
                }
                if removed {
                    if let Some(p) = paired {
                        let pkey = (key.0, p.0, p.1, p.2);
                        let p_empty = if let Some(pc) = chests.get_mut(&pkey) {
                            pc.viewers.remove(&addr);
                            pc.viewers.is_empty()
                        } else {
                            true
                        };
                        if !visited_pairs.contains(&key) && !visited_pairs.contains(&pkey) {
                            visited_pairs.insert(key);
                            visited_pairs.insert(pkey);
                            if is_empty && p_empty {
                                closed_chests.push(((key.1, key.2, key.3), true));
                            }
                        }
                    } else if is_empty {
                        closed_chests.push(((key.1, key.2, key.3), true));
                    }
                }
            }
        }
        for (_, f) in self.furnaces.write().iter_mut() {
            f.viewers.remove(&addr);
        }
        for (_, b) in self.brewing_stands.write().iter_mut() {
            b.viewers.remove(&addr);
        }
        closed_chests
    }

    pub fn get_chest_items(&self, x: i32, y: i32, z: i32) -> Option<Vec<Item>> {
        self.get_chest_items_in_dim(0, x, y, z)
    }

    pub fn get_chest_items_in_dim(&self, dim: i32, x: i32, y: i32, z: i32) -> Option<Vec<Item>> {
        self.chests.read().get(&(dim, x, y, z)).map(|c| c.slots.clone())
    }

    // --- FURNACE INTERACTION ---

    pub fn open_furnace(&self, addr: SocketAddr, x: i32, y: i32, z: i32) -> Vec<Item> {
        self.open_furnace_in_dim(addr, 0, x, y, z)
    }

    pub fn open_furnace_in_dim(&self, addr: SocketAddr, dim: i32, x: i32, y: i32, z: i32) -> Vec<Item> {
        let mut furnaces = self.furnaces.write();
        let furnace = furnaces.entry((dim, x, y, z)).or_insert_with(FurnaceData::new);
        furnace.viewers.insert(addr);
        furnace.to_items()
    }

    pub fn get_furnace_progress_in_dim(&self, dim: i32, x: i32, y: i32, z: i32) -> (i32, i32) {
        if let Some(f) = self.furnaces.read().get(&(dim, x, y, z)) {
            let burn_ticks = if f.burn_time > 0 && f.max_burn_time > 0 {
                ((f.burn_time as f32 / f.max_burn_time as f32) * 200.0).ceil() as i32
            } else {
                0
            };
            (f.cook_time as i32, burn_ticks)
        } else {
            (0, 0)
        }
    }

    pub fn set_furnace_slot(&self, x: i32, y: i32, z: i32, slot: usize, item: Item) {
        self.set_furnace_slot_in_dim(0, x, y, z, slot, item);
    }

    pub fn set_furnace_slot_in_dim(&self, dim: i32, x: i32, y: i32, z: i32, slot: usize, item: Item) -> Vec<SocketAddr> {
        let mut furnaces = self.furnaces.write();
        if let Some(f) = furnaces.get_mut(&(dim, x, y, z)) {
            match slot {
                0 => f.input = item,
                1 => f.fuel = item,
                2 => f.result = item,
                _ => {}
            }
            return f.viewers.iter().copied().collect();
        }
        Vec::new()
    }

    pub fn take_furnace_result_xp_in_dim(&self, dim: i32, x: i32, y: i32, z: i32) -> u32 {
        let mut furnaces = self.furnaces.write();
        if let Some(f) = furnaces.get_mut(&(dim, x, y, z)) {
            let whole = f.stored_xp.floor() as u32;
            let frac = f.stored_xp - whole as f32;
            let bonus = if frac > 0.0 && rand::random::<f32>() < frac { 1 } else { 0 };
            f.stored_xp = 0.0;
            whole + bonus
        } else {
            0
        }
    }

    pub fn tick_furnaces(&self, crafting: &CraftingManager) {
        let _ = self.tick_furnaces_live(crafting);
    }

    pub fn tick_furnaces_live(&self, crafting: &CraftingManager) -> FurnaceTickResult {
        let mut res_out = FurnaceTickResult::default();
        let mut furnaces = self.furnaces.write();

        for (&(dim, fx, fy, fz), f) in furnaces.iter_mut() {
            let was_burning = f.burn_time > 0;
            let smelt_recipe = if !f.input.is_air() {
                crafting.match_furnace(&f.input)
            } else {
                None
            };

            let can_smelt = if let Some(ref rec) = smelt_recipe {
                f.input.count > 0
                    && (f.result.is_air()
                        || (f.result.id == rec.result.id
                            && f.result.meta == rec.result.meta
                            && f.result.count < 64))
            } else {
                false
            };

            if f.burn_time == 0 && can_smelt && !f.fuel.is_air() {
                let burn_duration = fuel_burn_time(f.fuel.id, f.fuel.meta);
                if burn_duration > 0 {
                    f.burn_time = burn_duration;
                    f.max_burn_time = burn_duration;
                    if f.fuel.id == 325 && f.fuel.meta == 10 {
                        f.fuel = Item::new(325, 0, 1);
                    } else if f.fuel.count > 1 {
                        f.fuel.count -= 1;
                    } else {
                        f.fuel = Item::air();
                    }
                    if !was_burning {
                        res_out.block_toggles.push((dim, fx, fy, fz, true));
                    }
                    for &viewer in &f.viewers {
                        res_out.slot_syncs.push((viewer, 1, f.fuel.clone()));
                    }
                }
            }

            if f.burn_time > 0 {
                f.burn_time -= 1;
                let burn_ticks = if f.max_burn_time > 0 {
                    ((f.burn_time as f32 / f.max_burn_time as f32) * 200.0).ceil() as i32
                } else {
                    0
                };

                if can_smelt {
                    let smelt_res = smelt_recipe.as_ref().unwrap();
                    f.cook_time += 1;
                    if f.cook_time >= 200 {
                        f.cook_time = 0;
                        let input_id = f.input.id;
                        if f.input.count > 1 {
                            f.input.count -= 1;
                        } else {
                            f.input = Item::air();
                        }
                        if f.result.is_air() {
                            f.result = smelt_res.result.clone();
                        } else {
                            f.result.count += 1;
                        }
                        f.stored_xp += smelt_xp_reward(input_id);
                        for &viewer in &f.viewers {
                            res_out.slot_syncs.push((viewer, 0, f.input.clone()));
                            res_out.slot_syncs.push((viewer, 2, f.result.clone()));
                        }
                    }
                } else {
                    f.cook_time = 0;
                }

                for &viewer in &f.viewers {
                    res_out.data_updates.push((viewer, 0, f.cook_time as i32));
                    res_out.data_updates.push((viewer, 1, burn_ticks));
                }

                if f.burn_time == 0 {
                    f.max_burn_time = 0;
                    f.cook_time = 0;
                    res_out.block_toggles.push((dim, fx, fy, fz, false));
                }
            } else {
                if was_burning {
                    res_out.block_toggles.push((dim, fx, fy, fz, false));
                }
                f.cook_time = 0;
            }
        }
        res_out
    }

    // --- BREWING STAND INTERACTION ---

    pub fn open_brewing_in_dim(&self, addr: SocketAddr, dim: i32, x: i32, y: i32, z: i32) -> (Vec<Item>, u16) {
        let mut stands = self.brewing_stands.write();
        let stand = stands.entry((dim, x, y, z)).or_insert_with(BrewingData::new);
        stand.viewers.insert(addr);
        (stand.to_items(), stand.brew_time)
    }

    pub fn set_brewing_slot_in_dim(
        &self,
        dim: i32,
        x: i32,
        y: i32,
        z: i32,
        slot: usize,
        item: Item,
    ) -> Vec<SocketAddr> {
        let mut stands = self.brewing_stands.write();
        if let Some(b) = stands.get_mut(&(dim, x, y, z)) {
            match slot {
                0 => b.ingredient = item,
                1..=3 => b.bottles[slot - 1] = item,
                _ => {}
            }
            return b.viewers.iter().copied().collect();
        }
        Vec::new()
    }

    pub fn tick_brewing_stands(&self, crafting: &CraftingManager) -> BrewingTickResult {
        let mut out = BrewingTickResult::default();
        let mut stands = self.brewing_stands.write();

        for (_, b) in stands.iter_mut() {
            let can_brew = !b.ingredient.is_air()
                && b.ingredient.count > 0
                && is_brewing_ingredient(b.ingredient.id)
                && b.bottles.iter().any(|bot| {
                    !bot.is_air()
                        && matches!(bot.id, 373 | 438)
                        && crafting.match_brewing(&b.ingredient, bot).is_some()
                });

            if can_brew {
                if b.brew_time > 0 {
                    b.brew_time -= 1;
                }
                for &viewer in &b.viewers {
                    out.data_updates.push((viewer, 0, b.brew_time as i32));
                }
                if b.brew_time == 0 {
                    b.brew_time = BrewingData::MAX_BREW_TIME;
                    for i in 0..3 {
                        let bot = &b.bottles[i];
                        if !bot.is_air() && matches!(bot.id, 373 | 438) {
                            if let Some(res) = crafting.match_brewing(&b.ingredient, bot) {
                                b.bottles[i] = res.clone();
                                for &viewer in &b.viewers {
                                    out.slot_syncs.push((viewer, i + 1, res.clone()));
                                }
                            }
                        }
                    }
                    if b.ingredient.count > 1 {
                        b.ingredient.count -= 1;
                    } else {
                        b.ingredient = Item::air();
                    }
                    for &viewer in &b.viewers {
                        out.slot_syncs.push((viewer, 0, b.ingredient.clone()));
                        out.data_updates.push((viewer, 0, 0));
                    }
                }
            } else if b.brew_time != BrewingData::MAX_BREW_TIME {
                b.brew_time = BrewingData::MAX_BREW_TIME;
                for &viewer in &b.viewers {
                    out.data_updates.push((viewer, 0, 0));
                }
            }
        }
        out
    }

    // --- TEMPORARY WINDOWS (ENCHANTING TABLE / ANVIL) ---

    pub fn open_temp_window(&self, addr: SocketAddr, window_type: u8, dim: i32, x: i32, y: i32, z: i32, slots: usize) {
        self.temp_windows.write().insert(addr, TempWindow {
            window_type,
            dim,
            pos: (x, y, z),
            slots: vec![Item::air(); slots],
        });
    }

    pub fn set_temp_slot(&self, addr: SocketAddr, slot: usize, item: Item) {
        if let Some(w) = self.temp_windows.write().get_mut(&addr) {
            if slot < w.slots.len() {
                w.slots[slot] = item;
            }
        }
    }

    pub fn get_temp_window(&self, addr: SocketAddr) -> Option<TempWindow> {
        self.temp_windows.read().get(&addr).cloned()
    }

    pub fn clear_temp_slots(&self, addr: SocketAddr) {
        if let Some(w) = self.temp_windows.write().get_mut(&addr) {
            for s in &mut w.slots {
                *s = Item::air();
            }
        }
    }

    pub fn close_temp_window(&self, addr: SocketAddr) -> Vec<Item> {
        if let Some(w) = self.temp_windows.write().remove(&addr) {
            w.slots.into_iter().filter(|it| !it.is_air() && it.count > 0).collect()
        } else {
            Vec::new()
        }
    }

    // --- SIGN INTERACTION ---

    pub fn set_sign_text(&self, x: i32, y: i32, z: i32, lines: [String; 4], creator: String) {
        self.set_sign_text_in_dim(0, x, y, z, lines, creator);
    }

    pub fn set_sign_text_in_dim(&self, dim: i32, x: i32, y: i32, z: i32, lines: [String; 4], creator: String) {
        self.signs.write().insert((dim, x, y, z), SignData { lines, creator });
    }

    pub fn get_sign_text(&self, x: i32, y: i32, z: i32) -> Option<[String; 4]> {
        self.signs.read().get(&(0, x, y, z)).map(|s| s.lines.clone())
    }

    pub fn ensure_chest(&self, x: i32, y: i32, z: i32) {
        self.ensure_chest_in_dim(0, x, y, z, false);
    }

    pub fn ensure_chest_in_dim(&self, dim: i32, x: i32, y: i32, z: i32, is_trapped: bool) {
        let mut chests = self.chests.write();
        let c = chests.entry((dim, x, y, z)).or_insert_with(ChestData::new);
        c.is_trapped = is_trapped;
    }

    pub fn ensure_furnace(&self, x: i32, y: i32, z: i32) {
        self.ensure_furnace_in_dim(0, x, y, z);
    }

    pub fn ensure_furnace_in_dim(&self, dim: i32, x: i32, y: i32, z: i32) {
        self.furnaces.write().entry((dim, x, y, z)).or_insert_with(FurnaceData::new);
    }

    pub fn ensure_brewing_in_dim(&self, dim: i32, x: i32, y: i32, z: i32) {
        self.brewing_stands.write().entry((dim, x, y, z)).or_insert_with(BrewingData::new);
    }

    pub fn ensure_enchant_in_dim(&self, dim: i32, x: i32, y: i32, z: i32) {
        self.enchants.write().insert((dim, x, y, z));
    }

    pub fn ensure_sign(&self, x: i32, y: i32, z: i32) {
        self.ensure_sign_in_dim(0, x, y, z);
    }

    pub fn ensure_sign_in_dim(&self, dim: i32, x: i32, y: i32, z: i32) {
        self.signs.write().entry((dim, x, y, z)).or_insert_with(SignData::new);
    }

    pub fn set_bed_color(&self, x: i32, y: i32, z: i32, color: u8) {
        self.set_bed_color_in_dim(0, x, y, z, color);
    }

    pub fn set_bed_color_in_dim(&self, dim: i32, x: i32, y: i32, z: i32, color: u8) {
        self.beds.write().insert((dim, x, y, z), color & 0x0f);
    }

    // --- NETWORK NBT BUILDERS (write(true) wire encoding) ---

    pub fn build_sign_nbt(&self, x: i32, y: i32, z: i32) -> Vec<u8> {
        self.build_sign_nbt_in_dim(0, x, y, z)
    }

    pub fn build_sign_nbt_in_dim(&self, dim: i32, x: i32, y: i32, z: i32) -> Vec<u8> {
        let lines = self
            .signs
            .read()
            .get(&(dim, x, y, z))
            .map(|s| s.lines.clone())
            .unwrap_or_else(|| [String::new(), String::new(), String::new(), String::new()]);

        let mut s = BinaryStream::new();
        put_net_root_start(&mut s);
        put_net_string(&mut s, "id", "Sign");
        put_net_int(&mut s, "x", x);
        put_net_int(&mut s, "y", y);
        put_net_int(&mut s, "z", z);
        for (i, line) in lines.iter().enumerate() {
            put_net_string(&mut s, &format!("Text{}", i + 1), line);
        }
        s.put_byte(0x00);
        s.buffer
    }

    pub fn build_chest_nbt(&self, x: i32, y: i32, z: i32) -> Vec<u8> {
        self.build_chest_nbt_in_dim(0, x, y, z)
    }

    pub fn build_chest_nbt_in_dim(&self, dim: i32, x: i32, y: i32, z: i32) -> Vec<u8> {
        let mut s = BinaryStream::new();
        put_net_root_start(&mut s);
        put_net_string(&mut s, "id", "Chest");
        put_net_int(&mut s, "x", x);
        put_net_int(&mut s, "y", y);
        put_net_int(&mut s, "z", z);
        if let Some(chest) = self.chests.read().get(&(dim, x, y, z)) {
            if let Some(ref cname) = chest.custom_name {
                put_net_string(&mut s, "CustomName", cname);
            }
            if let Some((px, _py, pz)) = chest.paired {
                put_net_int(&mut s, "pairx", px);
                put_net_int(&mut s, "pairz", pz);
            }
        }
        s.put_byte(0x00);
        s.buffer
    }

    pub fn build_furnace_nbt(&self, x: i32, y: i32, z: i32) -> Vec<u8> {
        self.build_furnace_nbt_in_dim(0, x, y, z)
    }

    pub fn build_furnace_nbt_in_dim(&self, dim: i32, x: i32, y: i32, z: i32) -> Vec<u8> {
        let (burn_time, cook_time) = self
            .furnaces
            .read()
            .get(&(dim, x, y, z))
            .map(|f| (f.burn_time as i16, f.cook_time as i16))
            .unwrap_or((0, 0));

        let mut s = BinaryStream::new();
        put_net_root_start(&mut s);
        put_net_string(&mut s, "id", "Furnace");
        put_net_int(&mut s, "x", x);
        put_net_int(&mut s, "y", y);
        put_net_int(&mut s, "z", z);
        put_net_short(&mut s, "BurnTime", burn_time);
        put_net_short(&mut s, "CookTime", cook_time);
        s.put_byte(0x00);
        s.buffer
    }

    pub fn build_brewing_nbt_in_dim(&self, dim: i32, x: i32, y: i32, z: i32) -> Vec<u8> {
        let brew_time = self
            .brewing_stands
            .read()
            .get(&(dim, x, y, z))
            .map(|b| b.brew_time as i16)
            .unwrap_or(BrewingData::MAX_BREW_TIME as i16);

        let mut s = BinaryStream::new();
        put_net_root_start(&mut s);
        put_net_string(&mut s, "id", "BrewingStand");
        put_net_int(&mut s, "x", x);
        put_net_int(&mut s, "y", y);
        put_net_int(&mut s, "z", z);
        put_net_short(&mut s, "CookTime", brew_time);
        s.put_byte(0x00);
        s.buffer
    }

    pub fn build_enchant_nbt_in_dim(&self, _dim: i32, x: i32, y: i32, z: i32) -> Vec<u8> {
        let mut s = BinaryStream::new();
        put_net_root_start(&mut s);
        put_net_string(&mut s, "id", "EnchantTable");
        put_net_int(&mut s, "x", x);
        put_net_int(&mut s, "y", y);
        put_net_int(&mut s, "z", z);
        s.put_byte(0x00);
        s.buffer
    }

    pub fn build_bed_nbt(&self, x: i32, y: i32, z: i32, color: u8) -> Vec<u8> {
        let mut s = BinaryStream::new();
        put_net_root_start(&mut s);
        put_net_string(&mut s, "id", "Bed");
        put_net_int(&mut s, "x", x);
        put_net_int(&mut s, "y", y);
        put_net_int(&mut s, "z", z);
        put_net_byte(&mut s, "color", color as i8);
        s.put_byte(0x00);
        s.buffer
    }

    pub fn serialize_chunk_tiles(&self, cx: i32, cz: i32) -> Vec<Vec<u8>> {
        self.serialize_chunk_tiles_in_dim(0, cx, cz)
    }

    pub fn serialize_chunk_tiles_in_dim(&self, dim: i32, cx: i32, cz: i32) -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        for &(tdim, x, y, z) in self.chests.read().keys() {
            if tdim == dim && (x >> 4) == cx && (z >> 4) == cz {
                out.push(self.build_chest_nbt_in_dim(dim, x, y, z));
            }
        }
        for &(tdim, x, y, z) in self.furnaces.read().keys() {
            if tdim == dim && (x >> 4) == cx && (z >> 4) == cz {
                out.push(self.build_furnace_nbt_in_dim(dim, x, y, z));
            }
        }
        for &(tdim, x, y, z) in self.brewing_stands.read().keys() {
            if tdim == dim && (x >> 4) == cx && (z >> 4) == cz {
                out.push(self.build_brewing_nbt_in_dim(dim, x, y, z));
            }
        }
        for &(tdim, x, y, z) in self.enchants.read().iter() {
            if tdim == dim && (x >> 4) == cx && (z >> 4) == cz {
                out.push(self.build_enchant_nbt_in_dim(dim, x, y, z));
            }
        }
        for &(tdim, x, y, z) in self.signs.read().keys() {
            if tdim == dim && (x >> 4) == cx && (z >> 4) == cz {
                out.push(self.build_sign_nbt_in_dim(dim, x, y, z));
            }
        }
        for (&(tdim, x, y, z), &color) in self.beds.read().iter() {
            if tdim == dim && (x >> 4) == cx && (z >> 4) == cz {
                out.push(self.build_bed_nbt(x, y, z, color));
            }
        }
        out
    }

    // --- REMOVE TILE AND DROP ITEMS ---

    pub fn remove_tile(&self, x: i32, y: i32, z: i32) -> Vec<Item> {
        self.remove_tile_in_dim(0, x, y, z).0
    }

    /// Removes tile at `(dim, x, y, z)`, returning dropped items and an optional unpair coordinate to broadcast.
    pub fn remove_tile_in_dim(
        &self,
        dim: i32,
        x: i32,
        y: i32,
        z: i32,
    ) -> (Vec<Item>, Option<(i32, i32, i32)>) {
        let mut drops = Vec::new();
        let paired_pos = {
            let mut chests = self.chests.write();
            if let Some(chest) = chests.remove(&(dim, x, y, z)) {
                for item in chest.slots {
                    if !item.is_air() {
                        drops.push(item);
                    }
                }
                chest.paired
            } else {
                None
            }
        };
        if let Some(ppos) = paired_pos {
            if let Some(partner) = self.chests.write().get_mut(&(dim, ppos.0, ppos.1, ppos.2)) {
                partner.paired = None;
            }
        }
        if let Some(furnace) = self.furnaces.write().remove(&(dim, x, y, z)) {
            if !furnace.input.is_air() { drops.push(furnace.input); }
            if !furnace.fuel.is_air() { drops.push(furnace.fuel); }
            if !furnace.result.is_air() { drops.push(furnace.result); }
        }
        if let Some(stand) = self.brewing_stands.write().remove(&(dim, x, y, z)) {
            if !stand.ingredient.is_air() { drops.push(stand.ingredient); }
            for bot in stand.bottles {
                if !bot.is_air() { drops.push(bot); }
            }
        }
        self.enchants.write().remove(&(dim, x, y, z));
        self.signs.write().remove(&(dim, x, y, z));
        self.beds.write().remove(&(dim, x, y, z));
        (drops, paired_pos)
    }

    // --- DISK SERIALIZATION ---

    pub fn export_tiles_bytes(&self) -> Vec<u8> {
        let mut s = BinaryStream::new();
        s.put(b"OXTL");
        s.put_byte(1);

        {
            let chests = self.chests.read();
            s.put_unsigned_var_int(chests.len() as u32);
            for (&(dim, x, y, z), c) in chests.iter() {
                s.put_var_int(dim);
                s.put_var_int(x);
                s.put_var_int(y);
                s.put_var_int(z);
                s.put_bool(c.is_trapped);
                match c.custom_name {
                    Some(ref cname) => {
                        s.put_bool(true);
                        s.put_string(cname);
                    }
                    None => s.put_bool(false),
                }
                match c.paired {
                    Some((px, py, pz)) => {
                        s.put_bool(true);
                        s.put_var_int(px);
                        s.put_var_int(py);
                        s.put_var_int(pz);
                    }
                    None => s.put_bool(false),
                }
                s.put_unsigned_var_int(c.slots.len() as u32);
                for it in &c.slots {
                    crate::level::storage::put_item_binary(&mut s, it);
                }
            }
        }

        {
            let furnaces = self.furnaces.read();
            s.put_unsigned_var_int(furnaces.len() as u32);
            for (&(dim, x, y, z), f) in furnaces.iter() {
                s.put_var_int(dim);
                s.put_var_int(x);
                s.put_var_int(y);
                s.put_var_int(z);
                s.put_lshort(f.cook_time);
                s.put_lshort(f.burn_time);
                s.put_lshort(f.max_burn_time);
                s.put_lfloat(f.stored_xp);
                crate::level::storage::put_item_binary(&mut s, &f.input);
                crate::level::storage::put_item_binary(&mut s, &f.fuel);
                crate::level::storage::put_item_binary(&mut s, &f.result);
            }
        }

        {
            let stands = self.brewing_stands.read();
            s.put_unsigned_var_int(stands.len() as u32);
            for (&(dim, x, y, z), b) in stands.iter() {
                s.put_var_int(dim);
                s.put_var_int(x);
                s.put_var_int(y);
                s.put_var_int(z);
                s.put_lshort(b.brew_time);
                crate::level::storage::put_item_binary(&mut s, &b.ingredient);
                for bot in &b.bottles {
                    crate::level::storage::put_item_binary(&mut s, bot);
                }
            }
        }

        {
            let enchants = self.enchants.read();
            s.put_unsigned_var_int(enchants.len() as u32);
            for &(dim, x, y, z) in enchants.iter() {
                s.put_var_int(dim);
                s.put_var_int(x);
                s.put_var_int(y);
                s.put_var_int(z);
            }
        }

        {
            let signs = self.signs.read();
            s.put_unsigned_var_int(signs.len() as u32);
            for (&(dim, x, y, z), sd) in signs.iter() {
                s.put_var_int(dim);
                s.put_var_int(x);
                s.put_var_int(y);
                s.put_var_int(z);
                s.put_string(&sd.creator);
                for line in &sd.lines {
                    s.put_string(line);
                }
            }
        }

        {
            let beds = self.beds.read();
            s.put_unsigned_var_int(beds.len() as u32);
            for (&(dim, x, y, z), &color) in beds.iter() {
                s.put_var_int(dim);
                s.put_var_int(x);
                s.put_var_int(y);
                s.put_var_int(z);
                s.put_byte(color);
            }
        }

        s.into_inner()
    }

    pub fn import_tiles_bytes(&self, bytes: &[u8]) -> bool {
        let mut s = BinaryStream::from_bytes(bytes.to_vec());
        if s.get(4).ok().as_deref() != Some(b"OXTL") {
            return false;
        }
        if s.get_byte().unwrap_or(0) != 1 {
            return false;
        }

        let chest_cnt = s.get_unsigned_var_int().unwrap_or(0) as usize;
        {
            let mut chests = self.chests.write();
            for _ in 0..chest_cnt {
                let dim = s.get_var_int().unwrap_or(0);
                let x = s.get_var_int().unwrap_or(0);
                let y = s.get_var_int().unwrap_or(0);
                let z = s.get_var_int().unwrap_or(0);
                let is_trapped = s.get_bool().unwrap_or(false);
                let custom_name = if s.get_bool().unwrap_or(false) {
                    Some(s.get_string().unwrap_or_default())
                } else {
                    None
                };
                let paired = if s.get_bool().unwrap_or(false) {
                    let px = s.get_var_int().unwrap_or(0);
                    let py = s.get_var_int().unwrap_or(0);
                    let pz = s.get_var_int().unwrap_or(0);
                    Some((px, py, pz))
                } else {
                    None
                };
                let slot_cnt = s.get_unsigned_var_int().unwrap_or(27) as usize;
                let mut slots = Vec::with_capacity(slot_cnt.max(27));
                for _ in 0..slot_cnt {
                    slots.push(crate::level::storage::get_item_binary(&mut s));
                }
                slots.resize(27, Item::air());
                chests.insert(
                    (dim, x, y, z),
                    ChestData {
                        slots,
                        viewers: HashSet::new(),
                        custom_name,
                        paired,
                        is_trapped,
                    },
                );
            }
        }

        let fur_cnt = s.get_unsigned_var_int().unwrap_or(0) as usize;
        {
            let mut furnaces = self.furnaces.write();
            for _ in 0..fur_cnt {
                let dim = s.get_var_int().unwrap_or(0);
                let x = s.get_var_int().unwrap_or(0);
                let y = s.get_var_int().unwrap_or(0);
                let z = s.get_var_int().unwrap_or(0);
                let cook_time = s.get_lshort().unwrap_or(0);
                let burn_time = s.get_lshort().unwrap_or(0);
                let max_burn_time = s.get_lshort().unwrap_or(0);
                let stored_xp = s.get_lfloat().unwrap_or(0.0);
                let input = crate::level::storage::get_item_binary(&mut s);
                let fuel = crate::level::storage::get_item_binary(&mut s);
                let result = crate::level::storage::get_item_binary(&mut s);
                furnaces.insert(
                    (dim, x, y, z),
                    FurnaceData {
                        input,
                        fuel,
                        result,
                        cook_time,
                        burn_time,
                        max_burn_time,
                        stored_xp,
                        viewers: HashSet::new(),
                    },
                );
            }
        }

        let brew_cnt = s.get_unsigned_var_int().unwrap_or(0) as usize;
        {
            let mut stands = self.brewing_stands.write();
            for _ in 0..brew_cnt {
                let dim = s.get_var_int().unwrap_or(0);
                let x = s.get_var_int().unwrap_or(0);
                let y = s.get_var_int().unwrap_or(0);
                let z = s.get_var_int().unwrap_or(0);
                let brew_time = s.get_lshort().unwrap_or(BrewingData::MAX_BREW_TIME);
                let ingredient = crate::level::storage::get_item_binary(&mut s);
                let b0 = crate::level::storage::get_item_binary(&mut s);
                let b1 = crate::level::storage::get_item_binary(&mut s);
                let b2 = crate::level::storage::get_item_binary(&mut s);
                stands.insert(
                    (dim, x, y, z),
                    BrewingData {
                        ingredient,
                        bottles: [b0, b1, b2],
                        brew_time,
                        viewers: HashSet::new(),
                    },
                );
            }
        }

        let ench_cnt = s.get_unsigned_var_int().unwrap_or(0) as usize;
        {
            let mut enchants = self.enchants.write();
            for _ in 0..ench_cnt {
                let dim = s.get_var_int().unwrap_or(0);
                let x = s.get_var_int().unwrap_or(0);
                let y = s.get_var_int().unwrap_or(0);
                let z = s.get_var_int().unwrap_or(0);
                enchants.insert((dim, x, y, z));
            }
        }

        let sign_cnt = s.get_unsigned_var_int().unwrap_or(0) as usize;
        {
            let mut signs = self.signs.write();
            for _ in 0..sign_cnt {
                let dim = s.get_var_int().unwrap_or(0);
                let x = s.get_var_int().unwrap_or(0);
                let y = s.get_var_int().unwrap_or(0);
                let z = s.get_var_int().unwrap_or(0);
                let creator = s.get_string().unwrap_or_default();
                let l0 = s.get_string().unwrap_or_default();
                let l1 = s.get_string().unwrap_or_default();
                let l2 = s.get_string().unwrap_or_default();
                let l3 = s.get_string().unwrap_or_default();
                signs.insert(
                    (dim, x, y, z),
                    SignData {
                        lines: [l0, l1, l2, l3],
                        creator,
                    },
                );
            }
        }

        let bed_cnt = s.get_unsigned_var_int().unwrap_or(0) as usize;
        {
            let mut beds = self.beds.write();
            for _ in 0..bed_cnt {
                let dim = s.get_var_int().unwrap_or(0);
                let x = s.get_var_int().unwrap_or(0);
                let y = s.get_var_int().unwrap_or(0);
                let z = s.get_var_int().unwrap_or(0);
                let color = s.get_byte().unwrap_or(14);
                beds.insert((dim, x, y, z), color);
            }
        }

        true
    }
}

impl Default for TileManager {
    fn default() -> Self {
        Self::new()
    }
}

pub fn parse_sign_nbt_lines(namedtag: &[u8]) -> Option<[String; 4]> {
    let mut s = BinaryStream::from_bytes(namedtag.to_vec());
    if s.get_byte().ok()? != 0x0a {
        return None;
    }
    let root_name_len = s.get_unsigned_var_int().ok()? as usize;
    let _ = s.get(root_name_len).ok()?;

    let mut lines = [String::new(), String::new(), String::new(), String::new()];
    let mut is_sign = false;
    while !s.feof() {
        let tag_type = match s.get_byte() {
            Ok(t) => t,
            Err(_) => break,
        };
        if tag_type == 0x00 {
            break;
        }
        let name_len = s.get_unsigned_var_int().ok()? as usize;
        let name_bytes = s.get(name_len).ok()?;
        let name = String::from_utf8_lossy(&name_bytes).to_string();
        match tag_type {
            0x01 => { let _ = s.get_byte(); }
            0x02 => { let _ = s.get_lshort(); }
            0x03 => { let _ = s.get_var_int(); }
            0x04 => { let _ = s.get_var_long(); }
            0x05 => { let _ = s.get_lfloat(); }
            0x06 => { let _ = s.get(8); }
            0x08 => {
                let vlen = s.get_unsigned_var_int().ok()? as usize;
                let vbytes = s.get(vlen).ok()?;
                let val = String::from_utf8_lossy(&vbytes).to_string();
                match name.as_str() {
                    "id" => is_sign = val == "Sign",
                    "Text1" => lines[0] = val,
                    "Text2" => lines[1] = val,
                    "Text3" => lines[2] = val,
                    "Text4" => lines[3] = val,
                    _ => {}
                }
            }
            _ => break,
        }
    }
    if is_sign || lines.iter().any(|l| !l.is_empty()) {
        Some(lines)
    } else {
        None
    }
}

fn is_brewing_ingredient(item_id: i32) -> bool {
    matches!(
        item_id,
        372 | 348 | 331 | 376 | 378 | 353 | 382 | 375 | 370 | 377 | 396 | 414 | 462 | 289
    )
}

fn fuel_burn_time(item_id: i32, item_meta: i32) -> u16 {
    match item_id {
        263 => 1600,             // Coal / Charcoal
        173 => 16000,            // Coal Block
        369 => 2400,             // Blaze Rod
        17 | 162 => 300,         // Logs
        5 => 300,                // Planks
        280 => 100,              // Sticks
        325 if item_meta == 10 => 20000, // Lava bucket
        85 | 188 | 189 | 190 | 191 | 192 => 300, // Fences
        54 | 58 | 146 => 300,    // Chest, Crafting Table
        6 => 100,                // Saplings
        268 | 269 | 270 | 271 | 290 => 200, // Wooden tools
        _ => 0,
    }
}

fn smelt_xp_reward(input_id: i32) -> f32 {
    match input_id {
        56 | 129 | 14 => 1.0,    // Diamond, Emerald, Gold Ore
        15 | 73 => 0.7,          // Iron, Redstone Ore
        319 | 363 | 365 | 411 | 423 | 349 | 460 | 392 => 0.35, // Foods & kelp/potato
        337 | 82 => 0.35,        // Clay
        74 | 153 | 21 => 0.2,    // Lapis / Quartz Ore
        81 => 0.2,               // Cactus
        17 | 162 => 0.15,        // Wood -> Charcoal
        4 | 12 | 87 => 0.1,      // Stone, Glass, Nether Brick
        _ => 0.1,
    }
}

fn put_net_root_start(s: &mut BinaryStream) {
    s.put_byte(0x0a); // TAG_Compound
    s.put_unsigned_var_int(0); // Empty root name length
}

fn put_net_tag_name(s: &mut BinaryStream, tag_type: u8, name: &str) {
    s.put_byte(tag_type);
    let bytes = name.as_bytes();
    s.put_unsigned_var_int(bytes.len() as u32);
    s.put(bytes);
}

fn put_net_string(s: &mut BinaryStream, name: &str, val: &str) {
    put_net_tag_name(s, 0x08, name);
    let v = val.as_bytes();
    s.put_unsigned_var_int(v.len() as u32);
    s.put(v);
}

fn put_net_byte(s: &mut BinaryStream, name: &str, val: i8) {
    put_net_tag_name(s, 0x01, name);
    s.put_byte(val as u8);
}

fn put_net_short(s: &mut BinaryStream, name: &str, val: i16) {
    put_net_tag_name(s, 0x02, name);
    s.put(&val.to_le_bytes());
}

fn put_net_int(s: &mut BinaryStream, name: &str, val: i32) {
    put_net_tag_name(s, 0x03, name);
    s.put_var_int(val);
}
