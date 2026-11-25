use std::collections::{HashSet, VecDeque};
use crate::level::World;
use crate::protocol::types::Item;
use crate::item::enchantment::{
    parse_enchantments_from_nbt, TYPE_MINING_DURABILITY, TYPE_MINING_EFFICIENCY,
    TYPE_MINING_FORTUNE, TYPE_MINING_SILK_TOUCH,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolKind {
    None,
    Pickaxe,
    Axe,
    Shovel,
    Hoe,
    Sword,
    Shears,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ToolTier {
    None = 0,
    Wood = 1,
    Gold = 2,
    Stone = 3,
    Iron = 4,
    Diamond = 5,
}

#[derive(Debug, Clone, Copy)]
pub struct ToolSpec {
    pub kind: ToolKind,
    pub tier: ToolTier,
    pub max_durability: i32,
}

pub fn tool_spec(item_id: i32) -> ToolSpec {
    match item_id {
        270 => ToolSpec { kind: ToolKind::Pickaxe, tier: ToolTier::Wood, max_durability: 60 },
        285 => ToolSpec { kind: ToolKind::Pickaxe, tier: ToolTier::Gold, max_durability: 33 },
        274 => ToolSpec { kind: ToolKind::Pickaxe, tier: ToolTier::Stone, max_durability: 132 },
        257 => ToolSpec { kind: ToolKind::Pickaxe, tier: ToolTier::Iron, max_durability: 251 },
        278 => ToolSpec { kind: ToolKind::Pickaxe, tier: ToolTier::Diamond, max_durability: 1562 },

        271 => ToolSpec { kind: ToolKind::Axe, tier: ToolTier::Wood, max_durability: 60 },
        286 => ToolSpec { kind: ToolKind::Axe, tier: ToolTier::Gold, max_durability: 33 },
        275 => ToolSpec { kind: ToolKind::Axe, tier: ToolTier::Stone, max_durability: 132 },
        258 => ToolSpec { kind: ToolKind::Axe, tier: ToolTier::Iron, max_durability: 251 },
        279 => ToolSpec { kind: ToolKind::Axe, tier: ToolTier::Diamond, max_durability: 1562 },

        269 => ToolSpec { kind: ToolKind::Shovel, tier: ToolTier::Wood, max_durability: 60 },
        284 => ToolSpec { kind: ToolKind::Shovel, tier: ToolTier::Gold, max_durability: 33 },
        273 => ToolSpec { kind: ToolKind::Shovel, tier: ToolTier::Stone, max_durability: 132 },
        256 => ToolSpec { kind: ToolKind::Shovel, tier: ToolTier::Iron, max_durability: 251 },
        277 => ToolSpec { kind: ToolKind::Shovel, tier: ToolTier::Diamond, max_durability: 1562 },

        290 => ToolSpec { kind: ToolKind::Hoe, tier: ToolTier::Wood, max_durability: 60 },
        294 => ToolSpec { kind: ToolKind::Hoe, tier: ToolTier::Gold, max_durability: 33 },
        291 => ToolSpec { kind: ToolKind::Hoe, tier: ToolTier::Stone, max_durability: 132 },
        292 => ToolSpec { kind: ToolKind::Hoe, tier: ToolTier::Iron, max_durability: 251 },
        293 => ToolSpec { kind: ToolKind::Hoe, tier: ToolTier::Diamond, max_durability: 1562 },

        268 => ToolSpec { kind: ToolKind::Sword, tier: ToolTier::Wood, max_durability: 60 },
        283 => ToolSpec { kind: ToolKind::Sword, tier: ToolTier::Gold, max_durability: 33 },
        272 => ToolSpec { kind: ToolKind::Sword, tier: ToolTier::Stone, max_durability: 132 },
        267 => ToolSpec { kind: ToolKind::Sword, tier: ToolTier::Iron, max_durability: 251 },
        276 => ToolSpec { kind: ToolKind::Sword, tier: ToolTier::Diamond, max_durability: 1562 },

        359 => ToolSpec { kind: ToolKind::Shears, tier: ToolTier::None, max_durability: 239 },
        259 => ToolSpec { kind: ToolKind::None, tier: ToolTier::None, max_durability: 65 },
        _ => ToolSpec { kind: ToolKind::None, tier: ToolTier::None, max_durability: 0 },
    }
}

/// Horizontal facing index matching standard entity yaw convention:
/// 0 = South (+Z), 1 = West (-X), 2 = North (-Z), 3 = East (+X).
pub fn player_facing(yaw: f32) -> u8 {
    let mut rot = (yaw - 90.0) % 360.0;
    if rot < 0.0 {
        rot += 360.0;
    }
    if rot < 45.0 || rot >= 315.0 {
        2
    } else if rot < 135.0 {
        3
    } else if rot < 225.0 {
        0
    } else {
        1
    }
}

pub fn is_replaceable(id: u8) -> bool {
    matches!(id, 0 | 8 | 9 | 10 | 11 | 31 | 32 | 51 | 78 | 106 | 175)
}

pub fn is_transparent(id: u8) -> bool {
    matches!(
        id,
        0 | 6 | 8 | 9 | 10 | 11 | 18 | 20 | 26 | 27 | 28 | 30 | 31 | 32 | 37 | 38 | 39 | 40
            | 44 | 50 | 51 | 52 | 53 | 54 | 55 | 59 | 60 | 63 | 64 | 65 | 66 | 67 | 68 | 69
            | 70 | 71 | 72 | 75 | 76 | 77 | 78 | 79 | 81 | 83 | 85 | 90 | 92 | 93 | 94 | 95
            | 96 | 101 | 102 | 104 | 105 | 106 | 107 | 108 | 109 | 111 | 113 | 114 | 115 | 116
            | 117 | 118 | 119 | 120 | 126 | 127 | 128 | 130 | 131 | 132 | 134 | 135 | 136 | 138
            | 139 | 140 | 141 | 142 | 143 | 144 | 145 | 146 | 147 | 148 | 149 | 150 | 154 | 156
            | 158 | 160 | 161 | 163 | 164 | 167 | 171 | 175 | 180 | 182 | 183 | 184 | 185 | 186
            | 187 | 188 | 189 | 190 | 191 | 192 | 193 | 194 | 195 | 196 | 197 | 198 | 199 | 203
            | 205 | 208 | 244
    )
}

pub fn is_solid(id: u8) -> bool {
    !matches!(
        id,
        0 | 6 | 8 | 9 | 10 | 11 | 27 | 28 | 31 | 32 | 37 | 38 | 39 | 40 | 50 | 51 | 55 | 59
            | 63 | 65 | 66 | 68 | 69 | 70 | 72 | 75 | 76 | 77 | 83 | 90 | 93 | 94 | 104 | 105
            | 106 | 111 | 115 | 119 | 126 | 131 | 132 | 141 | 142 | 143 | 147 | 148 | 149 | 150
            | 175 | 244
    )
}

pub fn light_level(id: u8) -> u8 {
    match id {
        10 | 11 | 51 | 89 | 91 | 124 | 138 | 169 => 15,
        50 | 198 | 208 => 14,
        62 => 13,
        90 => 11,
        74 => 9,
        76 | 130 => 7,
        39 | 145 => 1,
        120 => 1,
        _ => 0,
    }
}

pub fn light_filter(id: u8) -> u8 {
    match id {
        8 | 9 | 18 | 30 | 79 | 161 | 174 => 3,
        id if is_transparent(id) => 1,
        _ => 15,
    }
}

pub fn block_hardness(id: u8) -> f32 {
    match id {
        0 => 0.0,
        7 | 119 | 120 | 137 => -1.0,
        49 => 50.0,
        42 | 57 | 133 | 145 | 152 | 173 => 5.0,
        22 | 41 | 116 | 130 | 138 | 140 => 3.0,
        45 | 48 | 61 | 62 | 98 | 101 | 108 | 109 | 112 | 114 => 2.0,
        1 | 4 | 14 | 15 | 16 | 21 | 43 | 44 | 53 | 56 | 67 | 71 | 73 | 74 | 121 | 129 | 153 | 156 | 167 | 181 | 182 => 2.5,
        5 | 17 | 47 | 54 | 58 | 64 | 85 | 96 | 107 | 146 | 157 | 158 | 162 | 183..=188 | 193..=197 => 2.0,
        25 | 84 | 168 | 169 => 2.0,
        24 | 155 | 179 | 201 | 202 | 203 | 206 => 0.8,
        12 | 13 | 88 => 0.5,
        2 | 3 | 60 | 110 | 198 | 243 => 0.6,
        35 | 171 => 0.8,
        20 | 89 | 95 | 102 | 123 | 124 | 160 => 0.3,
        18 | 26 | 79 | 161 | 174 => 0.2,
        78 | 80 => 0.2,
        6 | 31 | 32 | 37 | 38 | 39 | 40 | 46 | 50 | 51 | 55 | 59 | 75 | 76 | 83 | 104 | 105 | 106 | 111 | 115 | 141 | 142 | 175 | 244 => 0.0,
        _ => 1.5,
    }
}

pub fn blast_resistance(id: u8) -> f32 {
    match id {
        7 | 119 | 120 | 137 => 18_000_000.0,
        49 | 130 | 145 => 6_000.0,
        8 | 9 | 10 | 11 => 500.0,
        121 | 206 => 45.0,
        1 | 4 | 42 | 43 | 44 | 45 | 48 | 57 | 67 | 98 | 108 | 109 | 112 | 114 | 133 | 152 | 173 | 181 | 182 => 30.0,
        61 | 62 => 17.5,
        14 | 15 | 16 | 21 | 56 | 73 | 74 | 129 | 153 | 5 | 53 | 58 | 64 | 71 | 85 | 101 | 134..=136 | 163 | 164 | 188 => 15.0,
        17 | 162 | 54 | 146 | 154 => 10.0,
        24 | 155 | 179 | 35 => 4.0,
        2 | 3 | 12 | 13 | 60 | 88 | 110 | 243 => 2.5,
        20 | 89 | 95 | 102 | 123 | 124 | 160 | 18 | 161 => 1.5,
        _ => 1.0,
    }
}

pub fn required_tool(id: u8) -> (ToolKind, ToolTier) {
    match id {
        49 => (ToolKind::Pickaxe, ToolTier::Diamond),
        14 | 56 | 129 | 73 | 74 | 41 | 57 | 133 | 152 => (ToolKind::Pickaxe, ToolTier::Iron),
        15 | 21 | 22 | 42 => (ToolKind::Pickaxe, ToolTier::Stone),
        1 | 4 | 16 | 24 | 43 | 44 | 45 | 48 | 61 | 62 | 67 | 89 | 98 | 101 | 108 | 109 | 112
        | 114 | 116 | 117 | 118 | 121 | 128 | 130 | 139 | 145 | 153 | 154 | 155 | 156 | 168
        | 173 | 179 | 180 | 181 | 182 | 201 | 202 | 203 | 204 | 205 | 206 | 71 | 167 => {
            (ToolKind::Pickaxe, ToolTier::Wood)
        }
        5 | 17 | 25 | 47 | 53 | 54 | 58 | 63 | 64 | 65 | 68 | 84 | 85 | 96 | 107 | 134 | 135
        | 136 | 146 | 157 | 158 | 162 | 163 | 164 | 183..=197 => (ToolKind::Axe, ToolTier::None),
        2 | 3 | 12 | 13 | 60 | 78 | 80 | 82 | 88 | 110 | 198 | 243 | 252 => (ToolKind::Shovel, ToolTier::None),
        18 | 30 | 35 | 106 | 161 | 171 => (ToolKind::Shears, ToolTier::None),
        _ => (ToolKind::None, ToolTier::None),
    }
}

pub fn can_harvest_with(block_id: u8, held_item_id: i32) -> bool {
    let (req_kind, req_tier) = required_tool(block_id);
    if req_tier == ToolTier::None {
        return true;
    }
    let spec = tool_spec(held_item_id);
    spec.kind == req_kind && spec.tier >= req_tier
}

/// Break duration in seconds matching standard bedrock / reference core timings.
pub fn get_break_time(
    block_id: u8,
    held_item: &Item,
    haste_lvl: u8,
    mining_fatigue_lvl: u8,
    in_water: bool,
    has_aqua_affinity: bool,
) -> f32 {
    let hardness = block_hardness(block_id);
    if hardness < 0.0 {
        return f32::INFINITY;
    }
    if hardness == 0.0 {
        return 0.0;
    }

    let spec = tool_spec(held_item.id);
    let can_harvest = can_harvest_with(block_id, held_item.id);
    let mut base = hardness * if can_harvest { 1.5 } else { 5.0 };

    let (req_kind, _) = required_tool(block_id);
    if req_kind == ToolKind::Shears && spec.kind == ToolKind::Shears {
        base /= 15.0;
    } else if req_kind != ToolKind::None && spec.kind == req_kind {
        let speed = match spec.tier {
            ToolTier::Wood => 2.0,
            ToolTier::Stone => 4.0,
            ToolTier::Iron => 6.0,
            ToolTier::Diamond => 8.0,
            ToolTier::Gold => 12.0,
            ToolTier::None => 1.0,
        };
        let eff_lvl = parse_enchantments_from_nbt(&held_item.nbt)
            .iter()
            .find(|e| e.id == TYPE_MINING_EFFICIENCY)
            .map(|e| e.level)
            .unwrap_or(0);
        let eff_bonus = if eff_lvl > 0 {
            (eff_lvl as f32).powi(2) + 1.0
        } else {
            0.0
        };
        // Speed multiplier adds efficiency quadratic bonus over tool tier
        let _ = base;
        base = hardness * if can_harvest { 1.5 } else { 5.0 } / (speed + eff_bonus);
    } else if spec.kind == ToolKind::Sword {
        base /= 1.5;
    }

    if haste_lvl > 0 {
        base /= 1.0 + 0.2 * haste_lvl as f32;
    }
    if mining_fatigue_lvl > 0 {
        let penalty = 0.3f32.powi(mining_fatigue_lvl.min(4) as i32);
        base /= penalty;
    }
    if in_water && !has_aqua_affinity {
        base *= 5.0;
    }

    base.max(0.05)
}

pub fn get_block_drops_ext(block_id: u8, meta: u8, held_item: &Item) -> Vec<Item> {
    if !can_harvest_with(block_id, held_item.id) {
        return Vec::new();
    }

    let enchs = parse_enchantments_from_nbt(&held_item.nbt);
    let silk = enchs.iter().any(|e| e.id == TYPE_MINING_SILK_TOUCH);
    let fortune = enchs
        .iter()
        .find(|e| e.id == TYPE_MINING_FORTUNE)
        .map(|e| e.level)
        .unwrap_or(0);

    if silk {
        match block_id {
            1 | 2 | 16 | 18 | 20 | 21 | 47 | 56 | 73 | 74 | 79 | 89 | 95 | 102 | 110 | 129
            | 153 | 160 | 161 | 169 | 174 | 243 => {
                let drop_id = if block_id == 74 { 73 } else { block_id as i32 };
                let drop_meta = if matches!(block_id, 18 | 161) { (meta & 0x03) as i32 } else { 0 };
                return vec![Item::new(drop_id, drop_meta, 1)];
            }
            _ => {}
        }
    }

    let fortune_extra = |base: u32| -> u32 {
        if fortune == 0 {
            base
        } else {
            let bonus = rand::random::<u32>() % (fortune as u32 + 1);
            base + bonus
        }
    };

    match block_id {
        0 | 7 | 8 | 9 | 10 | 11 | 51 | 52 | 90 | 119 => vec![],
        20 | 79 | 95 | 102 | 160 | 174 => vec![],
        1 => vec![Item::new(4, 0, 1)],
        2 | 60 | 110 | 198 | 243 => vec![Item::new(3, 0, 1)],
        13 => {
            let chance = match fortune {
                0 => 10,
                1 => 7,
                2 => 4,
                _ => 1,
            };
            if (rand::random::<u32>() % chance) == 0 {
                vec![Item::new(318, 0, 1)]
            } else {
                vec![Item::new(13, 0, 1)]
            }
        }
        16 => vec![Item::new(263, 0, fortune_extra(1))],
        21 => vec![Item::new(351, 4, fortune_extra(4 + (rand::random::<u32>() % 5)))],
        56 => vec![Item::new(264, 0, fortune_extra(1))],
        73 | 74 => vec![Item::new(331, 0, fortune_extra(4 + (rand::random::<u32>() % 2)))],
        89 => vec![Item::new(348, 0, (2 + (rand::random::<u32>() % 3) + fortune as u32).min(4))],
        129 => vec![Item::new(388, 0, fortune_extra(1))],
        153 => vec![Item::new(406, 0, fortune_extra(1))],
        169 => vec![Item::new(409, 0, (2 + (rand::random::<u32>() % 2) + fortune as u32).min(5))],
        47 => vec![Item::new(340, 0, 3)],
        43 => vec![Item::new(44, (meta & 0x07) as i32, 2)],
        157 => vec![Item::new(158, (meta & 0x07) as i32, 2)],
        181 => vec![Item::new(182, (meta & 0x07) as i32, 2)],
        204 => vec![Item::new(205, (meta & 0x07) as i32, 2)],
        44 | 158 | 182 | 205 => vec![Item::new(block_id as i32, (meta & 0x07) as i32, 1)],
        17 | 162 => vec![Item::new(block_id as i32, (meta & 0x03) as i32, 1)],
        18 | 161 => {
            if tool_spec(held_item.id).kind == ToolKind::Shears {
                vec![Item::new(block_id as i32, (meta & 0x03) as i32, 1)]
            } else {
                let mut out = Vec::new();
                if (rand::random::<u32>() % 20) == 0 {
                    let sap_meta = if block_id == 161 { (meta & 0x01) + 4 } else { meta & 0x03 };
                    out.push(Item::new(6, sap_meta as i32, 1));
                }
                if block_id == 18 && (meta & 0x03) == 0 && (rand::random::<u32>() % 200) == 0 {
                    out.push(Item::new(260, 0, 1));
                }
                out
            }
        }
        31 => {
            if tool_spec(held_item.id).kind == ToolKind::Shears {
                vec![Item::new(31, meta as i32, 1)]
            } else if (rand::random::<u32>() % 8) == 0 {
                vec![Item::new(295, 0, 1)]
            } else {
                vec![]
            }
        }
        32 => vec![Item::new(280, 0, rand::random::<u32>() % 3)],
        26 => vec![Item::new(355, 0, 1)],
        63 | 68 => vec![Item::new(323, 0, 1)],
        64 => vec![Item::new(324, 0, 1)],
        71 => vec![Item::new(330, 0, 1)],
        193 => vec![Item::new(427, 0, 1)],
        194 => vec![Item::new(428, 0, 1)],
        195 => vec![Item::new(429, 0, 1)],
        196 => vec![Item::new(430, 0, 1)],
        197 => vec![Item::new(431, 0, 1)],
        55 => vec![Item::new(331, 0, 1)],
        75 | 76 => vec![Item::new(76, 0, 1)],
        83 => vec![Item::new(338, 0, 1)],
        93 | 94 => vec![Item::new(356, 0, 1)],
        149 | 150 => vec![Item::new(404, 0, 1)],
        117 => vec![Item::new(379, 0, 1)],
        118 => vec![Item::new(380, 0, 1)],
        140 => vec![Item::new(390, 0, 1)],
        115 => {
            let count = if meta >= 3 { fortune_extra(2 + (rand::random::<u32>() % 3)) } else { 1 };
            vec![Item::new(372, 0, count)]
        }
        59 => {
            if meta >= 7 {
                vec![Item::new(296, 0, 1), Item::new(295, 0, 1 + (rand::random::<u32>() % 3) + fortune as u32)]
            } else {
                vec![Item::new(295, 0, 1)]
            }
        }
        141 => vec![Item::new(391, 0, if meta >= 7 { 1 + (rand::random::<u32>() % 3) + fortune as u32 } else { 1 })],
        142 => vec![Item::new(392, 0, if meta >= 7 { 1 + (rand::random::<u32>() % 3) + fortune as u32 } else { 1 })],
        244 => {
            if meta >= 4 {
                vec![Item::new(457, 0, 1), Item::new(458, 0, 1 + (rand::random::<u32>() % 3))]
            } else {
                vec![Item::new(458, 0, 1)]
            }
        }
        78 => vec![Item::new(332, 0, (meta as u32 + 1).min(8))],
        80 => vec![Item::new(332, 0, 4)],
        82 => vec![Item::new(337, 0, 4)],
        103 => vec![Item::new(360, 0, (3 + (rand::random::<u32>() % 5) + fortune as u32).min(9))],
        53 | 67 | 108 | 109 | 114 | 128 | 134 | 135 | 136 | 156 | 163 | 164 | 180 | 203 => {
            vec![Item::new(block_id as i32, 0, 1)]
        }
        61 | 62 => vec![Item::new(61, 0, 1)],
        54 | 65 | 96 | 107 | 130 | 146 | 167 | 183..=187 => vec![Item::new(block_id as i32, 0, 1)],
        _ => vec![Item::new(block_id as i32, meta as i32, 1)],
    }
}

pub fn ore_xp_reward(block_id: u8, held_item: &Item) -> u32 {
    if !can_harvest_with(block_id, held_item.id) {
        return 0;
    }
    let silk = parse_enchantments_from_nbt(&held_item.nbt)
        .iter()
        .any(|e| e.id == TYPE_MINING_SILK_TOUCH);
    if silk {
        return 0;
    }
    match block_id {
        16 => rand::random::<u32>() % 3,
        56 | 129 => 3 + (rand::random::<u32>() % 5),
        21 | 153 => 2 + (rand::random::<u32>() % 4),
        73 | 74 => 1 + (rand::random::<u32>() % 5),
        _ => 0,
    }
}

/// Decrements tool durability on block break; returns true if the tool broke.
pub fn apply_tool_damage(item: &mut Item) -> bool {
    let spec = tool_spec(item.id);
    if spec.max_durability <= 0 {
        return false;
    }
    let unbreaking = parse_enchantments_from_nbt(&item.nbt)
        .iter()
        .find(|e| e.id == TYPE_MINING_DURABILITY)
        .map(|e| e.level)
        .unwrap_or(0);
    if unbreaking > 0 && rand::random::<f32>() >= (1.0 / (unbreaking as f32 + 1.0)) {
        return false;
    }
    item.meta += 1;
    item.meta >= spec.max_durability
}

pub fn item_to_block_id(item_id: i32, target_id: u8, face: i32) -> Option<u8> {
    if item_id > 0 && item_id < 256 {
        return Some(item_id as u8);
    }
    match item_id {
        295 => Some(59),
        323 => Some(if face == 1 { 63 } else { 68 }),
        324 => Some(64),
        330 => Some(71),
        331 => Some(55),
        338 => Some(83),
        354 => Some(92),
        355 => Some(26),
        356 => Some(93),
        361 => Some(104),
        362 => Some(105),
        372 => Some(115),
        379 => Some(117),
        380 => Some(118),
        390 => Some(140),
        391 if target_id == 60 && face == 1 => Some(141),
        392 if target_id == 60 && face == 1 => Some(142),
        404 => Some(149),
        427 => Some(193),
        428 => Some(194),
        429 => Some(195),
        430 => Some(196),
        431 => Some(197),
        458 => Some(244),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub enum PlacementResult {
    Denied,
    Single { x: i32, y: i32, z: i32, id: u8, meta: u8 },
    Double {
        x1: i32, y1: i32, z1: i32, id1: u8, meta1: u8,
        x2: i32, y2: i32, z2: i32, id2: u8, meta2: u8,
    },
}

fn double_slab_for(id: u8) -> u8 {
    match id {
        44 => 43,
        158 => 157,
        182 => 181,
        205 => 204,
        _ => 43,
    }
}

fn is_slab(id: u8) -> bool {
    matches!(id, 44 | 158 | 182 | 205)
}

fn is_stair(id: u8) -> bool {
    matches!(id, 53 | 67 | 108 | 109 | 114 | 128 | 134 | 135 | 136 | 156 | 163 | 164 | 180 | 203)
}

fn is_door(id: u8) -> bool {
    matches!(id, 64 | 71 | 193 | 194 | 195 | 196 | 197)
}

fn is_trapdoor(id: u8) -> bool {
    matches!(id, 96 | 167)
}

fn is_fence_gate(id: u8) -> bool {
    matches!(id, 107 | 183 | 184 | 185 | 186 | 187)
}

pub fn resolve_placement(
    world: &mut World,
    tx: i32,
    ty: i32,
    tz: i32,
    face: i32,
    fy: f32,
    player_yaw: f32,
    block_id: u8,
    item_meta: i32,
) -> PlacementResult {
    let target_id = world.get_block_id(tx, ty, tz);
    let target_meta = {
        let ch = world.get_chunk_or_generate(tx >> 4, tz >> 4);
        ch.get_block_data((tx & 0x0f) as usize, ty.clamp(0, 255) as usize, (tz & 0x0f) as usize)
    };

    let (nx, ny, nz) = if is_replaceable(target_id) {
        (tx, ty, tz)
    } else {
        match face {
            0 => (tx, ty - 1, tz),
            1 => (tx, ty + 1, tz),
            2 => (tx, ty, tz - 1),
            3 => (tx, ty, tz + 1),
            4 => (tx - 1, ty, tz),
            5 => (tx + 1, ty, tz),
            _ => (tx, ty + 1, tz),
        }
    };

    if ny < 0 || ny >= 256 {
        return PlacementResult::Denied;
    }

    let replace_id = world.get_block_id(nx, ny, nz);
    let replace_meta = {
        let ch = world.get_chunk_or_generate(nx >> 4, nz >> 4);
        ch.get_block_data((nx & 0x0f) as usize, ny as usize, (nz & 0x0f) as usize)
    };

    let variant = (item_meta & 0x0f) as u8;
    let dir = player_facing(player_yaw);

    // Slab placement & double slab merge
    if is_slab(block_id) {
        let var7 = variant & 0x07;
        let dbl = double_slab_for(block_id);
        if face == 0 {
            if target_id == block_id && (target_meta & 0x08) == 0x08 && (target_meta & 0x07) == var7 {
                return PlacementResult::Single { x: tx, y: ty, z: tz, id: dbl, meta: var7 };
            } else if replace_id == block_id && (replace_meta & 0x07) == var7 {
                return PlacementResult::Single { x: nx, y: ny, z: nz, id: dbl, meta: var7 };
            }
            if !is_replaceable(replace_id) {
                return PlacementResult::Denied;
            }
            return PlacementResult::Single { x: nx, y: ny, z: nz, id: block_id, meta: var7 | 0x08 };
        } else if face == 1 {
            if target_id == block_id && (target_meta & 0x08) == 0 && (target_meta & 0x07) == var7 {
                return PlacementResult::Single { x: tx, y: ty, z: tz, id: dbl, meta: var7 };
            } else if replace_id == block_id && (replace_meta & 0x07) == var7 {
                return PlacementResult::Single { x: nx, y: ny, z: nz, id: dbl, meta: var7 };
            }
            if !is_replaceable(replace_id) {
                return PlacementResult::Denied;
            }
            return PlacementResult::Single { x: nx, y: ny, z: nz, id: block_id, meta: var7 };
        } else {
            if replace_id == block_id {
                if (replace_meta & 0x07) == var7 {
                    return PlacementResult::Single { x: nx, y: ny, z: nz, id: dbl, meta: var7 };
                }
                return PlacementResult::Denied;
            }
            if !is_replaceable(replace_id) {
                return PlacementResult::Denied;
            }
            let final_meta = if fy > 0.5 { var7 | 0x08 } else { var7 };
            return PlacementResult::Single { x: nx, y: ny, z: nz, id: block_id, meta: final_meta };
        }
    }

    if !is_replaceable(replace_id) {
        return PlacementResult::Denied;
    }

    // Doors (2 blocks tall + hinge bit)
    if is_door(block_id) {
        if face != 1 || ny + 1 >= 256 {
            return PlacementResult::Denied;
        }
        let down_id = if ny > 0 { world.get_block_id(nx, ny - 1, nz) } else { 0 };
        let up_id = world.get_block_id(nx, ny + 1, nz);
        if is_transparent(down_id) || !is_replaceable(up_id) {
            return PlacementResult::Denied;
        }
        let side_offsets: [(i32, i32); 4] = [(0, 1), (-1, 0), (0, -1), (1, 0)];
        let left_dir = ((dir + 2) % 4) as usize;
        let right_dir = (dir % 4) as usize;
        let (lx, lz) = side_offsets[left_dir];
        let (rx, rz) = side_offsets[right_dir];
        let left_id = world.get_block_id(nx + lx, ny, nz + lz);
        let right_id = world.get_block_id(nx + rx, ny, nz + rz);
        let mut meta_up = 0x08u8;
        if left_id == block_id || (!is_transparent(right_id) && is_transparent(left_id)) {
            meta_up |= 0x01;
        }
        return PlacementResult::Double {
            x1: nx, y1: ny, z1: nz, id1: block_id, meta1: dir & 0x03,
            x2: nx, y2: ny + 1, z2: nz, id2: block_id, meta2: meta_up,
        };
    }

    // Beds (2 blocks horizontal)
    if block_id == 26 {
        let down_id = if ny > 0 { world.get_block_id(nx, ny - 1, nz) } else { 0 };
        if is_transparent(down_id) {
            return PlacementResult::Denied;
        }
        let rot = ((dir as i8 - 1).rem_euclid(4)) as u8;
        let (hx, hz) = match rot {
            0 => (nx, nz + 1),
            1 => (nx - 1, nz),
            2 => (nx, nz - 1),
            _ => (nx + 1, nz),
        };
        let head_replace = world.get_block_id(hx, ny, hz);
        let head_down = if ny > 0 { world.get_block_id(hx, ny - 1, hz) } else { 0 };
        if !is_replaceable(head_replace) || is_transparent(head_down) {
            return PlacementResult::Denied;
        }
        return PlacementResult::Double {
            x1: nx, y1: ny, z1: nz, id1: 26, meta1: rot,
            x2: hx, y2: ny, z2: hz, id2: 26, meta2: rot | 0x08,
        };
    }

    // Stairs
    if is_stair(block_id) {
        let faces = [0u8, 2, 1, 3];
        let mut meta = faces[(dir & 0x03) as usize];
        if (fy > 0.5 && face != 1) || face == 0 {
            meta |= 0x04;
        }
        return PlacementResult::Single { x: nx, y: ny, z: nz, id: block_id, meta };
    }

    // Wood logs axis
    if block_id == 17 || block_id == 162 {
        let axis = match face {
            2 | 3 => 0b1000,
            4 | 5 => 0b0100,
            _ => 0,
        };
        return PlacementResult::Single { x: nx, y: ny, z: nz, id: block_id, meta: (variant & 0x03) | axis };
    }

    // Chests, Furnaces, EnderChest
    if matches!(block_id, 54 | 61 | 62 | 130 | 146) {
        let faces = [4u8, 2, 5, 3];
        let meta = faces[(dir & 0x03) as usize];
        return PlacementResult::Single { x: nx, y: ny, z: nz, id: block_id, meta };
    }

    // Pumpkins
    if block_id == 86 || block_id == 91 {
        let faces = [1u8, 2, 3, 0];
        let meta = faces[(dir & 0x03) as usize];
        return PlacementResult::Single { x: nx, y: ny, z: nz, id: block_id, meta };
    }

    // Fence gates
    if is_fence_gate(block_id) {
        let meta = ((dir as i8 - 1).rem_euclid(4)) as u8;
        return PlacementResult::Single { x: nx, y: ny, z: nz, id: block_id, meta };
    }

    // Trapdoors
    if is_trapdoor(block_id) {
        let dirs = [1u8, 3, 0, 2];
        let mut meta = dirs[(dir & 0x03) as usize];
        if (fy > 0.5 && face != 1) || face == 0 {
            meta |= 0x04;
        }
        return PlacementResult::Single { x: nx, y: ny, z: nz, id: block_id, meta };
    }

    // Torches & Redstone Torches
    if matches!(block_id, 50 | 75 | 76) {
        if !is_transparent(target_id) && face != 0 {
            let m = match face {
                1 => 5,
                2 => 4,
                3 => 3,
                4 => 2,
                5 => 1,
                _ => 5,
            };
            return PlacementResult::Single { x: nx, y: ny, z: nz, id: block_id, meta: m };
        }
        let below = if ny > 0 { world.get_block_id(nx, ny - 1, nz) } else { 0 };
        if !is_transparent(below) || matches!(below, 85 | 113 | 139 | 123 | 124) {
            return PlacementResult::Single { x: nx, y: ny, z: nz, id: block_id, meta: 0 };
        }
        return PlacementResult::Denied;
    }

    // Ladders
    if block_id == 65 {
        if (2..=5).contains(&face) && !is_transparent(target_id) {
            return PlacementResult::Single { x: nx, y: ny, z: nz, id: 65, meta: face as u8 };
        }
        return PlacementResult::Denied;
    }

    // Signs
    if block_id == 63 || block_id == 68 {
        if face == 0 {
            return PlacementResult::Denied;
        }
        if face == 1 {
            let m = (((player_yaw + 180.0) * 16.0 / 360.0 + 0.5).floor() as i32 & 0x0f) as u8;
            return PlacementResult::Single { x: nx, y: ny, z: nz, id: 63, meta: m };
        }
        return PlacementResult::Single { x: nx, y: ny, z: nz, id: 68, meta: face as u8 };
    }

    // Crops / Saplings / Flowers / Rails / Pressure plates / Redstone wire
    if matches!(block_id, 59 | 141 | 142 | 244) {
        let below = if ny > 0 { world.get_block_id(nx, ny - 1, nz) } else { 0 };
        if below != 60 {
            return PlacementResult::Denied;
        }
        return PlacementResult::Single { x: nx, y: ny, z: nz, id: block_id, meta: 0 };
    }

    if block_id == 115 {
        let below = if ny > 0 { world.get_block_id(nx, ny - 1, nz) } else { 0 };
        if below != 88 {
            return PlacementResult::Denied;
        }
        return PlacementResult::Single { x: nx, y: ny, z: nz, id: 115, meta: 0 };
    }

    if block_id == 83 {
        let below = if ny > 0 { world.get_block_id(nx, ny - 1, nz) } else { 0 };
        let has_water = matches!(
            world.get_block_id(nx + 1, ny - 1, nz), 8 | 9
        ) || matches!(world.get_block_id(nx - 1, ny - 1, nz), 8 | 9)
          || matches!(world.get_block_id(nx, ny - 1, nz + 1), 8 | 9)
          || matches!(world.get_block_id(nx, ny - 1, nz - 1), 8 | 9);
        if below == 83 || (matches!(below, 2 | 3 | 12 | 243) && has_water) {
            return PlacementResult::Single { x: nx, y: ny, z: nz, id: 83, meta: 0 };
        }
        return PlacementResult::Denied;
    }

    // Player-placed leaves should not decay (bit 0x04)
    if block_id == 18 || block_id == 161 {
        return PlacementResult::Single { x: nx, y: ny, z: nz, id: block_id, meta: (variant & 0x03) | 0x04 };
    }

    // Lever & Button
    if block_id == 69 || block_id == 77 || block_id == 143 {
        let m = match face {
            0 => 0,
            1 => if matches!(dir, 1 | 3) { 6 } else { 5 },
            2 => 4,
            3 => 3,
            4 => 2,
            5 => 1,
            _ => 5,
        };
        let final_m = if block_id == 69 { m } else { face as u8 };
        return PlacementResult::Single { x: nx, y: ny, z: nz, id: block_id, meta: final_m };
    }

    // Repeaters & Comparators
    if block_id == 93 || block_id == 149 {
        let m = ((dir + 2) & 0x03) as u8;
        return PlacementResult::Single { x: nx, y: ny, z: nz, id: block_id, meta: m };
    }

    PlacementResult::Single { x: nx, y: ny, z: nz, id: block_id, meta: variant }
}

/// Checks cascading support losses and double-block removals around modified coordinates.
/// Returns `(changed_blocks, dropped_items)`.
pub fn cascade_neighbor_breaks(
    world: &mut World,
    origins: &[(i32, i32, i32)],
) -> (Vec<(i32, i32, i32, u8, u8)>, Vec<(i32, i32, i32, Item)>) {
    let mut changed = Vec::new();
    let mut drops = Vec::new();
    let mut queue = VecDeque::new();
    let mut queued = HashSet::new();

    for &(ox, oy, oz) in origins {
        let candidates = [
            (ox, oy, oz),
            (ox + 1, oy, oz),
            (ox - 1, oy, oz),
            (ox, oy + 1, oz),
            (ox, oy - 1, oz),
            (ox, oy, oz + 1),
            (ox, oy, oz - 1),
        ];
        for pos in candidates {
            if pos.1 >= 0 && pos.1 < 256 && queued.insert(pos) {
                queue.push_back(pos);
            }
        }
    }

    while let Some((x, y, z)) = queue.pop_front() {
        let id = world.get_block_id(x, y, z);
        if id == 0 {
            continue;
        }
        let meta = {
            let ch = world.get_chunk_or_generate(x >> 4, z >> 4);
            ch.get_block_data((x & 0x0f) as usize, y as usize, (z & 0x0f) as usize)
        };

        let mut should_break = false;
        let below_id = if y > 0 { world.get_block_id(x, y - 1, z) } else { 0 };

        if is_door(id) {
            if (meta & 0x08) != 0 {
                if y == 0 || world.get_block_id(x, y - 1, z) != id {
                    should_break = true;
                }
            } else {
                let down_ok = y > 0 && !is_transparent(below_id);
                let up_ok = y + 1 < 256 && world.get_block_id(x, y + 1, z) == id;
                if !down_ok || !up_ok {
                    should_break = true;
                }
            }
        } else if id == 26 {
            let is_head = (meta & 0x08) != 0;
            let rot = meta & 0x03;
            let (ox, oz) = match (rot, is_head) {
                (0, false) | (2, true) => (x, z + 1),
                (1, false) | (3, true) => (x - 1, z),
                (2, false) | (0, true) => (x, z - 1),
                _ => (x + 1, z),
            };
            if world.get_block_id(ox, y, oz) != 26 {
                should_break = true;
            }
        } else if matches!(id, 50 | 75 | 76) {
            let support = match meta & 0x07 {
                1 => (x - 1, y, z),
                2 => (x + 1, y, z),
                3 => (x, y, z - 1),
                4 => (x, y, z + 1),
                _ => (x, y - 1, z),
            };
            let sup_id = if support.1 >= 0 { world.get_block_id(support.0, support.1, support.2) } else { 0 };
            if is_transparent(sup_id) && !((meta & 0x07 == 0 || meta & 0x07 == 5) && matches!(sup_id, 85 | 113 | 139 | 123 | 124)) {
                should_break = true;
            }
        } else if id == 65 || id == 68 {
            let support = match meta {
                2 => (x, y, z + 1),
                3 => (x, y, z - 1),
                4 => (x + 1, y, z),
                5 => (x - 1, y, z),
                _ => (x, y - 1, z),
            };
            if is_transparent(world.get_block_id(support.0, support.1, support.2)) {
                should_break = true;
            }
        } else if matches!(id, 59 | 141 | 142 | 244) {
            if below_id != 60 {
                should_break = true;
            }
        } else if id == 115 {
            if below_id != 88 {
                should_break = true;
            }
        } else if matches!(id, 6 | 31 | 32 | 37 | 38 | 39 | 40 | 55 | 63 | 66 | 27 | 28 | 70 | 72 | 78 | 93 | 94 | 126 | 147 | 148 | 149 | 150 | 171 | 175) {
            if y == 0 || is_transparent(below_id) {
                if !matches!(id, 6 | 31 | 37 | 38) || !matches!(below_id, 2 | 3 | 60 | 110 | 243) {
                    if !matches!(id, 175) || y == 0 || below_id != 175 {
                        should_break = true;
                    }
                }
            }
        } else if id == 83 {
            if y == 0 || (below_id != 83 && !matches!(below_id, 2 | 3 | 12 | 243)) {
                should_break = true;
            }
        } else if id == 81 {
            let adj_solid = !is_transparent(world.get_block_id(x + 1, y, z))
                || !is_transparent(world.get_block_id(x - 1, y, z))
                || !is_transparent(world.get_block_id(x, y, z + 1))
                || !is_transparent(world.get_block_id(x, y, z - 1));
            if adj_solid || (below_id != 81 && below_id != 12) {
                should_break = true;
            }
        } else if (id == 12 || id == 13) && y > 0 && is_replaceable(below_id) {
            let mut fall_y = y;
            while fall_y > 0 && is_replaceable(world.get_block_id(x, fall_y - 1, z)) {
                fall_y -= 1;
            }
            if fall_y != y {
                world.set_block(x, y, z, 0, 0);
                world.set_block(x, fall_y, z, id, meta);
                changed.push((x, y, z, 0, 0));
                changed.push((x, fall_y, z, id, meta));
                for n in [(x, y + 1, z), (x + 1, y, z), (x - 1, y, z), (x, y, z + 1), (x, y, z - 1)] {
                    if n.1 >= 0 && n.1 < 256 && queued.insert(n) {
                        queue.push_back(n);
                    }
                }
            }
            continue;
        }

        if should_break {
            world.set_block(x, y, z, 0, 0);
            changed.push((x, y, z, 0, 0));
            // Suppress duplicate bed/door drops when upper half collapses
            let suppress_drop = (is_door(id) && (meta & 0x08) != 0) || (id == 26 && (meta & 0x08) != 0);
            if !suppress_drop {
                for item in get_block_drops_ext(id, meta, &Item::air()) {
                    drops.push((x, y, z, item));
                }
            }
            for n in [
                (x + 1, y, z), (x - 1, y, z),
                (x, y + 1, z), (x, y - 1, z),
                (x, y, z + 1), (x, y, z - 1),
            ] {
                if n.1 >= 0 && n.1 < 256 && queued.insert(n) {
                    queue.push_back(n);
                }
            }
        }
    }

    (changed, drops)
}
