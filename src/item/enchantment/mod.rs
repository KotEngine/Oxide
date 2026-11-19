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
        // Tag Short (0x02), name "id" (len 2: 0x00 0x02 'i' 'd'), 2 bytes value
        if nbt[i] == 0x02 && i + 7 <= nbt.len() && &nbt[i+1..i+5] == b"\x00\x02id" {
            let id = u16::from_le_bytes([nbt[i+5], nbt[i+6]]);
            // Next is likely TAG_Short "lvl" (len 3: 0x00 0x03 'l' 'v' 'l'), 2 bytes value
            let next = i + 7;
            if next + 8 <= nbt.len() && nbt[next] == 0x02 && &nbt[next+1..next+6] == b"\x00\x03lvl" {
                let lvl = u16::from_le_bytes([nbt[next+6], nbt[next+7]]);
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
    item.nbt = serialize_enchantments_nbt(&enchs);
}

pub fn serialize_enchantments_nbt(enchs: &[EnchantmentInstance]) -> Vec<u8> {
    if enchs.is_empty() { return Vec::new(); }
    let mut buf = Vec::new();

    // Compound root
    buf.push(0x0a);
    buf.push(0x00);
    buf.push(0x00);

    // TAG_List "ench" (type 0x09)
    buf.push(0x09);
    buf.extend_from_slice(&(4u16).to_le_bytes()); // name len 4
    buf.extend_from_slice(b"ench");
    buf.push(0x0a); // list element type: TAG_Compound
    buf.extend_from_slice(&(enchs.len() as i32).to_le_bytes()); // count

    for e in enchs {
        // TAG_Short "id"
        buf.push(0x02);
        buf.extend_from_slice(&(2u16).to_le_bytes());
        buf.extend_from_slice(b"id");
        buf.extend_from_slice(&e.id.to_le_bytes());

        // TAG_Short "lvl"
        buf.push(0x02);
        buf.extend_from_slice(&(3u16).to_le_bytes());
        buf.extend_from_slice(b"lvl");
        buf.extend_from_slice(&e.level.to_le_bytes());

        // TAG_End of element compound
        buf.push(0x00);
    }

    // TAG_End of root compound
    buf.push(0x00);
    buf
}
