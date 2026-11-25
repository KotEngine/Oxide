use crate::protocol::types::Item;

pub const TYPE_ARMOR_PROTECTION: u16 = 0;
pub const TYPE_ARMOR_FIRE_PROTECTION: u16 = 1;
pub const TYPE_ARMOR_FALL_PROTECTION: u16 = 2;
pub const TYPE_ARMOR_EXPLOSION_PROTECTION: u16 = 3;
pub const TYPE_ARMOR_PROJECTILE_PROTECTION: u16 = 4;
pub const TYPE_ARMOR_THORNS: u16 = 5;
pub const TYPE_WATER_BREATHING: u16 = 6;
pub const TYPE_WATER_SPEED: u16 = 7;
pub const TYPE_WATER_AFFINITY: u16 = 8;
pub const TYPE_WEAPON_SHARPNESS: u16 = 9;
pub const TYPE_WEAPON_SMITE: u16 = 10;
pub const TYPE_WEAPON_ARTHROPODS: u16 = 11;
pub const TYPE_WEAPON_KNOCKBACK: u16 = 12;
pub const TYPE_WEAPON_FIRE_ASPECT: u16 = 13;
pub const TYPE_WEAPON_LOOTING: u16 = 14;
pub const TYPE_MINING_EFFICIENCY: u16 = 15;
pub const TYPE_MINING_SILK_TOUCH: u16 = 16;
pub const TYPE_MINING_DURABILITY: u16 = 17;
pub const TYPE_MINING_FORTUNE: u16 = 18;
pub const TYPE_BOW_POWER: u16 = 19;
pub const TYPE_BOW_KNOCKBACK: u16 = 20;
pub const TYPE_BOW_FLAME: u16 = 21;
pub const TYPE_BOW_INFINITY: u16 = 22;
pub const TYPE_FISHING_FORTUNE: u16 = 23;
pub const TYPE_FISHING_LURE: u16 = 24;
pub const TYPE_FROST_WALKER: u16 = 25;
pub const TYPE_MENDING: u16 = 26;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Enchantment {
    pub id: u16,
    pub name: &'static str,
    pub max_level: u16,
    pub rarity: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnchantmentInstance {
    pub id: u16,
    pub level: u16,
}

impl EnchantmentInstance {
    pub fn new(id: u16, level: u16) -> Self {
        Self { id, level }
    }
}

pub const ALL_ENCHANTMENTS: &[Enchantment] = &[
    Enchantment { id: TYPE_ARMOR_PROTECTION, name: "Protection", max_level: 4, rarity: 10 },
    Enchantment { id: TYPE_ARMOR_FIRE_PROTECTION, name: "Fire Protection", max_level: 4, rarity: 5 },
    Enchantment { id: TYPE_ARMOR_FALL_PROTECTION, name: "Feather Falling", max_level: 4, rarity: 5 },
    Enchantment { id: TYPE_ARMOR_EXPLOSION_PROTECTION, name: "Blast Protection", max_level: 4, rarity: 2 },
    Enchantment { id: TYPE_ARMOR_PROJECTILE_PROTECTION, name: "Projectile Protection", max_level: 4, rarity: 5 },
    Enchantment { id: TYPE_ARMOR_THORNS, name: "Thorns", max_level: 3, rarity: 1 },
    Enchantment { id: TYPE_WATER_BREATHING, name: "Respiration", max_level: 3, rarity: 2 },
    Enchantment { id: TYPE_WATER_SPEED, name: "Depth Strider", max_level: 3, rarity: 2 },
    Enchantment { id: TYPE_WATER_AFFINITY, name: "Aqua Affinity", max_level: 1, rarity: 2 },
    Enchantment { id: TYPE_WEAPON_SHARPNESS, name: "Sharpness", max_level: 5, rarity: 10 },
    Enchantment { id: TYPE_WEAPON_SMITE, name: "Smite", max_level: 5, rarity: 5 },
    Enchantment { id: TYPE_WEAPON_ARTHROPODS, name: "Bane of Arthropods", max_level: 5, rarity: 5 },
    Enchantment { id: TYPE_WEAPON_KNOCKBACK, name: "Knockback", max_level: 2, rarity: 5 },
    Enchantment { id: TYPE_WEAPON_FIRE_ASPECT, name: "Fire Aspect", max_level: 2, rarity: 2 },
    Enchantment { id: TYPE_WEAPON_LOOTING, name: "Looting", max_level: 3, rarity: 2 },
    Enchantment { id: TYPE_MINING_EFFICIENCY, name: "Efficiency", max_level: 5, rarity: 10 },
    Enchantment { id: TYPE_MINING_SILK_TOUCH, name: "Silk Touch", max_level: 1, rarity: 1 },
    Enchantment { id: TYPE_MINING_DURABILITY, name: "Unbreaking", max_level: 3, rarity: 5 },
    Enchantment { id: TYPE_MINING_FORTUNE, name: "Fortune", max_level: 3, rarity: 2 },
    Enchantment { id: TYPE_BOW_POWER, name: "Power", max_level: 5, rarity: 10 },
    Enchantment { id: TYPE_BOW_KNOCKBACK, name: "Punch", max_level: 2, rarity: 2 },
    Enchantment { id: TYPE_BOW_FLAME, name: "Flame", max_level: 1, rarity: 2 },
    Enchantment { id: TYPE_BOW_INFINITY, name: "Infinity", max_level: 1, rarity: 1 },
    Enchantment { id: TYPE_FISHING_FORTUNE, name: "Luck of the Sea", max_level: 3, rarity: 2 },
    Enchantment { id: TYPE_FISHING_LURE, name: "Lure", max_level: 3, rarity: 2 },
    Enchantment { id: TYPE_FROST_WALKER, name: "Frost Walker", max_level: 2, rarity: 2 },
    Enchantment { id: TYPE_MENDING, name: "Mending", max_level: 1, rarity: 2 },
];

pub fn get_enchantment(id: u16) -> Option<&'static Enchantment> {
    ALL_ENCHANTMENTS.iter().find(|e| e.id == id)
}

pub fn get_enchantment_by_name(name: &str) -> Option<&'static Enchantment> {
    let lower = name.to_lowercase();
    ALL_ENCHANTMENTS.iter().find(|e| {
        e.name.to_lowercase() == lower
            || e.name.to_lowercase().replace(' ', "") == lower
            || e.name.to_lowercase().replace(' ', "_") == lower
    })
}

// --- FORMULAS & BONUSES ---

pub fn calculate_melee_damage_bonus(enchs: &[EnchantmentInstance], target_undead: bool, target_arthropod: bool) -> f32 {
    let mut bonus = 0.0f32;
    for e in enchs {
        match e.id {
            TYPE_WEAPON_SHARPNESS => {
                bonus += 1.0 + (e.level.saturating_sub(1) as f32) * 0.5;
            }
            TYPE_WEAPON_SMITE if target_undead => {
                bonus += (e.level as f32) * 2.5;
            }
            TYPE_WEAPON_ARTHROPODS if target_arthropod => {
                bonus += (e.level as f32) * 2.5;
            }
            _ => {}
        }
    }
    bonus
}

pub fn calculate_protection_epf(enchs: &[EnchantmentInstance], is_fire: bool, is_fall: bool, is_explosion: bool, is_projectile: bool) -> u32 {
    let mut epf = 0u32;
    for e in enchs {
        match e.id {
            TYPE_ARMOR_PROTECTION => epf += e.level as u32,
            TYPE_ARMOR_FIRE_PROTECTION if is_fire => epf += (e.level as u32) * 2,
            TYPE_ARMOR_FALL_PROTECTION if is_fall => epf += (e.level as u32) * 3,
            TYPE_ARMOR_EXPLOSION_PROTECTION if is_explosion => epf += (e.level as u32) * 2,
            TYPE_ARMOR_PROJECTILE_PROTECTION if is_projectile => epf += (e.level as u32) * 2,
            _ => {}
        }
    }
    epf.min(25) // Vanilla MC cap is 20 or 25 EPF
}

pub fn calculate_damage_reduction_multiplier(epf: u32) -> f32 {
    let reduction = (epf as f32 * 0.04).min(0.80);
    1.0 - reduction
}

pub fn calculate_efficiency_multiplier(level: u16) -> f32 {
    if level == 0 { 1.0 } else { 1.0 + ((level * level + 1) as f32) }
}

pub fn should_consume_durability(unbreaking_lvl: u16, is_armor: bool, rng_val: f32) -> bool {
    if unbreaking_lvl == 0 { return true; }
    if is_armor {
        // Armor: 60% chance to reduce durability (40% ignore)
        rng_val < (0.6 / (unbreaking_lvl as f32 + 1.0) + 0.4)
    } else {
        // Tools/weapons: 100 / (level + 1) % chance
        rng_val < (1.0 / (unbreaking_lvl as f32 + 1.0))
    }
}

pub fn get_knockback_bonus(enchs: &[EnchantmentInstance]) -> f32 {
    for e in enchs {
        if e.id == TYPE_WEAPON_KNOCKBACK || e.id == TYPE_BOW_KNOCKBACK {
            return (e.level as f32) * 0.5;
        }
    }
    0.0
}

pub fn get_fire_aspect_seconds(enchs: &[EnchantmentInstance]) -> u32 {
    for e in enchs {
        if e.id == TYPE_WEAPON_FIRE_ASPECT {
            return (e.level as u32) * 4;
        }
        if e.id == TYPE_BOW_FLAME {
            return 5;
        }
    }
    0
}

pub fn get_fortune_drop_count(enchs: &[EnchantmentInstance], rng_val: f32) -> u32 {
    for e in enchs {
        if e.id == TYPE_MINING_FORTUNE || e.id == TYPE_WEAPON_LOOTING {
            let max_extra = e.level as u32;
            let extra = (rng_val * (max_extra as f32 + 1.0)).floor() as u32;
            return 1 + extra;
        }
    }
    1
}

pub fn has_silk_touch(enchs: &[EnchantmentInstance]) -> bool {
    enchs.iter().any(|e| e.id == TYPE_MINING_SILK_TOUCH)
}

pub fn has_infinity(enchs: &[EnchantmentInstance]) -> bool {
    enchs.iter().any(|e| e.id == TYPE_BOW_INFINITY)
}

// --- NBT PARSING & SERIALIZATION ---

pub fn parse_enchantments_from_item(item: &Item) -> Vec<EnchantmentInstance> {
    parse_enchantments_from_nbt(&item.nbt)
}

pub fn parse_enchantments_from_nbt(nbt: &[u8]) -> Vec<EnchantmentInstance> {
    if nbt.is_empty() { return Vec::new(); }
    let mut list = Vec::new();
    // Search for "id" and "lvl" short tags in compound
    let mut i = 0;
    while i + 8 <= nbt.len() {
        // Tag Short (0x02), name "id" (len 2 LE or BE), 2 bytes value
        if nbt[i] == 0x02
            && i + 7 <= nbt.len()
            && matches!(&nbt[i + 1..i + 5], b"\x02\x00id" | b"\x00\x02id")
        {
            let id = u16::from_le_bytes([nbt[i + 5], nbt[i + 6]]);
            // Next is likely TAG_Short "lvl" (len 3 LE or BE), 2 bytes value
            let next = i + 7;
            if next + 8 <= nbt.len()
                && nbt[next] == 0x02
                && matches!(&nbt[next + 1..next + 6], b"\x03\x00lvl" | b"\x00\x03lvl")
            {
                let lvl = u16::from_le_bytes([nbt[next + 6], nbt[next + 7]]);
                list.push(EnchantmentInstance::new(id, lvl));
                i = next + 8;
                continue;
            }
        }
        i += 1;
    }
    list
}

pub fn apply_enchantment_to_item(item: &mut Item, id: u16, level: u16) {
    let mut enchs = parse_enchantments_from_nbt(&item.nbt);
    enchs.retain(|e| e.id != id);
    enchs.push(EnchantmentInstance::new(id, level));
    let cname = parse_item_custom_name(&item.nbt);
    let rcost = parse_item_repair_cost(&item.nbt);
    item.nbt = serialize_item_nbt(&enchs, cname.as_deref(), rcost);
}

pub fn parse_item_custom_name(nbt: &[u8]) -> Option<String> {
    let mut i = 0;
    while i + 9 <= nbt.len() {
        let is_le = &nbt[i + 1..i + 7] == b"\x04\x00Name";
        let is_be = &nbt[i + 1..i + 7] == b"\x00\x04Name";
        if nbt[i] == 0x08 && (is_le || is_be) {
            let start = i + 7;
            if start + 2 <= nbt.len() {
                let slen = if is_le {
                    u16::from_le_bytes([nbt[start], nbt[start + 1]]) as usize
                } else {
                    let le = u16::from_le_bytes([nbt[start], nbt[start + 1]]) as usize;
                    let be = u16::from_be_bytes([nbt[start], nbt[start + 1]]) as usize;
                    if start + 2 + le <= nbt.len() { le } else { be }
                };
                let str_start = start + 2;
                if str_start + slen <= nbt.len() {
                    return Some(String::from_utf8_lossy(&nbt[str_start..str_start + slen]).to_string());
                }
            }
        }
        i += 1;
    }
    None
}

pub fn parse_item_repair_cost(nbt: &[u8]) -> u32 {
    let mut i = 0;
    while i + 17 <= nbt.len() {
        if nbt[i] == 0x03
            && matches!(&nbt[i + 1..i + 13], b"\x0a\x00RepairCost" | b"\x00\x0aRepairCost")
        {
            let val = i32::from_le_bytes([nbt[i + 13], nbt[i + 14], nbt[i + 15], nbt[i + 16]]);
            return val.max(0) as u32;
        }
        i += 1;
    }
    0
}

pub fn set_item_display_and_repair(item: &mut Item, custom_name: Option<&str>, repair_cost: u32) {
    let enchs = parse_enchantments_from_nbt(&item.nbt);
    item.nbt = serialize_item_nbt(&enchs, custom_name, repair_cost);
}

pub fn enchantments_conflict(a: u16, b: u16) -> bool {
    if a == b {
        return true;
    }
    if a <= 4 && b <= 4 {
        return true;
    }
    if (9..=11).contains(&a) && (9..=11).contains(&b) {
        return true;
    }
    (a == TYPE_MINING_SILK_TOUCH && b == TYPE_MINING_FORTUNE)
        || (a == TYPE_MINING_FORTUNE && b == TYPE_MINING_SILK_TOUCH)
}

pub fn is_valid_enchantment_for_item(item_id: i32, ench_id: u16) -> bool {
    match item_id {
        403 => true, // Enchanted Book
        298 | 302 | 306 | 310 | 314 => matches!(ench_id, 0..=6 | 8 | 17), // Helmets
        299 | 303 | 307 | 311 | 315 => matches!(ench_id, 0 | 1 | 3 | 4 | 5 | 17), // Chestplates
        300 | 304 | 308 | 312 | 316 => matches!(ench_id, 0 | 1 | 3 | 4 | 5 | 17), // Leggings
        301 | 305 | 309 | 313 | 317 => matches!(ench_id, 0..=5 | 7 | 17 | 25), // Boots
        267 | 268 | 272 | 276 | 283 => matches!(ench_id, 9..=14 | 17), // Swords
        261 => matches!(ench_id, 17 | 19..=22), // Bow
        256 | 257 | 258 | 269 | 270 | 271 | 273 | 274 | 275 | 277 | 278 | 279 | 284 | 285 | 286 => {
            matches!(ench_id, 15..=18) // Tools
        }
        346 => matches!(ench_id, 17 | 23 | 24), // Fishing Rod
        _ => false,
    }
}

pub fn anvil_repair_material(item_id: i32) -> i32 {
    match item_id {
        // Wood tools -> Planks (5)
        268 | 269 | 270 | 271 | 290 => 5,
        // Stone tools -> Cobblestone (4)
        272 | 273 | 274 | 275 | 291 => 4,
        // Iron tools & armor -> Iron Ingot (265)
        256 | 257 | 258 | 267 | 292 | 306 | 307 | 308 | 309 | 302 | 303 | 304 | 305 => 265,
        // Gold tools & armor -> Gold Ingot (266)
        283 | 284 | 285 | 286 | 294 | 314 | 315 | 316 | 317 => 266,
        // Diamond tools & armor -> Diamond (264)
        276 | 277 | 278 | 279 | 293 | 310 | 311 | 312 | 313 => 264,
        // Leather armor -> Leather (334)
        298 | 299 | 300 | 301 => 334,
        _ => 0,
    }
}

pub fn item_max_durability(item_id: i32) -> i32 {
    let t = crate::block::physics::tool_spec(item_id).max_durability;
    if t > 0 {
        return t as i32;
    }
    crate::entity::damage::armor_stats(item_id).1
}

pub fn anvil_combine_items(target: &Item, sacrifice: &Item, custom_name: Option<&str>) -> Option<(Item, u32)> {
    if target.is_air() {
        return None;
    }
    let mut out = target.clone();
    let base_cost = parse_item_repair_cost(&target.nbt);
    let sac_cost = parse_item_repair_cost(&sacrifice.nbt);
    let mut xp_cost = base_cost + sac_cost;
    let mut changed = false;

    if !sacrifice.is_air() {
        let repair_mat = anvil_repair_material(target.id);
        if repair_mat != 0 && sacrifice.id == repair_mat && target.meta > 0 {
            let max_dur = item_max_durability(target.id);
            if max_dur > 0 {
                let per_mat = (max_dur / 4).max(1);
                let mats_used = sacrifice.count.min(((target.meta + per_mat - 1) / per_mat) as u32);
                if mats_used > 0 {
                    out.meta = (target.meta - (mats_used as i32 * per_mat)).max(0);
                    xp_cost += mats_used;
                    changed = true;
                }
            }
        } else if sacrifice.id == 403 || sacrifice.id == target.id {
            let mut merged = parse_enchantments_from_item(&out);
            let sac_enchs = parse_enchantments_from_item(sacrifice);
            for se in sac_enchs {
                if !is_valid_enchantment_for_item(out.id, se.id) {
                    continue;
                }
                if let Some(existing) = merged.iter_mut().find(|e| e.id == se.id) {
                    let new_lvl = if existing.level == se.level {
                        (existing.level + 1).min(get_enchantment(se.id).map(|m| m.max_level).unwrap_or(5))
                    } else {
                        existing.level.max(se.level)
                    };
                    if new_lvl > existing.level {
                        existing.level = new_lvl;
                        xp_cost += new_lvl as u32;
                        changed = true;
                    }
                } else if !merged.iter().any(|e| enchantments_conflict(e.id, se.id)) {
                    merged.push(se);
                    xp_cost += se.level as u32;
                    changed = true;
                }
            }
            if target.id == sacrifice.id && target.meta > 0 {
                let max_dur = item_max_durability(target.id);
                if max_dur > 0 {
                    let rem_a = (max_dur - target.meta).max(0);
                    let rem_b = (max_dur - sacrifice.meta).max(0);
                    let bonus = max_dur * 12 / 100;
                    let total_rem = (rem_a + rem_b + bonus).min(max_dur);
                    out.meta = (max_dur - total_rem).max(0);
                    xp_cost += 2;
                    changed = true;
                }
            }
            let final_name = custom_name.map(|s| s.to_string()).or_else(|| parse_item_custom_name(&target.nbt));
            out.nbt = serialize_item_nbt(&merged, final_name.as_deref(), base_cost.max(sac_cost) * 2 + 1);
        }
    }

    if let Some(name) = custom_name {
        if !name.is_empty() && parse_item_custom_name(&target.nbt).as_deref() != Some(name) {
            let enchs = parse_enchantments_from_item(&out);
            let new_rc = (base_cost * 2 + 1).max(1);
            out.nbt = serialize_item_nbt(&enchs, Some(name), new_rc);
            xp_cost += 1;
            changed = true;
        }
    }

    if changed {
        Some((out, xp_cost.max(1)))
    } else {
        None
    }
}

pub fn count_bookshelves_around(world: &mut crate::level::World, x: i32, y: i32, z: i32) -> u32 {
    let mut count = 0u32;
    let offsets = [
        (2, 0), (-2, 0), (0, 2), (0, -2),
        (2, 1), (2, -1), (-2, 1), (-2, -1),
        (1, 2), (-1, 2), (1, -2), (-1, -2),
        (2, 2), (-2, 2), (2, -2), (-2, -2),
    ];
    for dy in 0..=1 {
        for &(dx, dz) in &offsets {
            if world.get_block_id(x + dx, y + dy, z + dz) == 47 {
                count += 1;
                if count >= 15 {
                    return 15;
                }
            }
        }
    }
    count
}

pub fn serialize_enchantments_nbt(enchs: &[EnchantmentInstance]) -> Vec<u8> {
    serialize_item_nbt(enchs, None, 0)
}

pub fn serialize_item_nbt(
    enchs: &[EnchantmentInstance],
    custom_name: Option<&str>,
    repair_cost: u32,
) -> Vec<u8> {
    let has_name = custom_name.map(|n| !n.is_empty()).unwrap_or(false);
    if enchs.is_empty() && !has_name && repair_cost == 0 {
        return Vec::new();
    }
    let mut buf = Vec::new();

    // Compound root
    buf.push(0x0a);
    buf.push(0x00);
    buf.push(0x00);

    if !enchs.is_empty() {
        buf.push(0x09); // TAG_List
        buf.extend_from_slice(&(4u16).to_le_bytes());
        buf.extend_from_slice(b"ench");
        buf.push(0x0a); // element type: TAG_Compound
        buf.extend_from_slice(&(enchs.len() as i32).to_le_bytes());

        for e in enchs {
            buf.push(0x02);
            buf.extend_from_slice(&(2u16).to_le_bytes());
            buf.extend_from_slice(b"id");
            buf.extend_from_slice(&e.id.to_le_bytes());

            buf.push(0x02);
            buf.extend_from_slice(&(3u16).to_le_bytes());
            buf.extend_from_slice(b"lvl");
            buf.extend_from_slice(&e.level.to_le_bytes());

            buf.push(0x00);
        }
    }

    if let Some(name) = custom_name {
        if !name.is_empty() {
            buf.push(0x0a); // TAG_Compound "display"
            buf.extend_from_slice(&(7u16).to_le_bytes());
            buf.extend_from_slice(b"display");

            buf.push(0x08); // TAG_String "Name"
            buf.extend_from_slice(&(4u16).to_le_bytes());
            buf.extend_from_slice(b"Name");
            let nb = name.as_bytes();
            buf.extend_from_slice(&(nb.len() as u16).to_le_bytes());
            buf.extend_from_slice(nb);

            buf.push(0x00); // End "display"
        }
    }

    if repair_cost > 0 {
        buf.push(0x03); // TAG_Int "RepairCost"
        buf.extend_from_slice(&(10u16).to_le_bytes());
        buf.extend_from_slice(b"RepairCost");
        buf.extend_from_slice(&(repair_cost as i32).to_le_bytes());
    }

    buf.push(0x00);
    buf
}
