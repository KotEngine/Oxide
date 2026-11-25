use crate::protocol::types::Item;
use rand::Rng;

/// Generates loot drops for entities upon death based on vanilla Bedrock 1.1.5 loot tables.
pub struct LootTable;

impl LootTable {
    /// Generate dropped items for a given entity type ID.
    ///
    /// # Arguments
    /// * `entity_type` - Bedrock entity type ID (e.g. 32 for Zombie, 33 for Creeper)
    /// * `killed_by_player` - Whether the killer was a player (unlocks rare drops)
    /// * `on_fire` - Whether the mob was burning when it died (cooks raw food)
    /// * `looting_level` - Looting enchantment level (0 to 3)
    pub fn get_drops(
        entity_type: i32,
        killed_by_player: bool,
        on_fire: bool,
        looting_level: u32,
    ) -> Vec<Item> {
        let mut drops = Vec::new();
        let mut rng = rand::thread_rng();

        let looting_bonus = |_max: u32, rng: &mut rand::rngs::ThreadRng| -> u32 {
            if looting_level > 0 {
                rng.gen_range(0..=looting_level)
            } else {
                0
            }
        };

        match entity_type {
            // Chicken (10)
            10 => {
                // Feathers: 0-2 + looting
                let feathers = rng.gen_range(0..=2) + looting_bonus(looting_level, &mut rng);
                if feathers > 0 {
                    drops.push(Item::new(288, 0, feathers));
                }
                // Meat: 1 (cooked if on fire)
                let meat_id = if on_fire { 366 } else { 365 };
                drops.push(Item::new(meat_id, 0, 1));
            }

            // Cow (11)
            11 => {
                // Leather: 0-2 + looting
                let leather = rng.gen_range(0..=2) + looting_bonus(looting_level, &mut rng);
                if leather > 0 {
                    drops.push(Item::new(334, 0, leather));
                }
                // Beef: 1-3 + looting (cooked if on fire)
                let beef_count = rng.gen_range(1..=3) + looting_bonus(looting_level, &mut rng);
                let beef_id = if on_fire { 364 } else { 363 };
                drops.push(Item::new(beef_id, 0, beef_count));
            }

            // Pig (12)
            12 => {
                // Porkchop: 1-3 + looting (cooked if on fire)
                let pork_count = rng.gen_range(1..=3) + looting_bonus(looting_level, &mut rng);
                let pork_id = if on_fire { 320 } else { 319 };
                drops.push(Item::new(pork_id, 0, pork_count));
            }

            // Sheep (13)
            13 => {
                // Wool: 1
                drops.push(Item::new(35, 0, 1));
                // Mutton: 1-2 + looting (cooked if on fire)
                let mutton_count = rng.gen_range(1..=2) + looting_bonus(looting_level, &mut rng);
                let mutton_id = if on_fire { 424 } else { 423 };
                drops.push(Item::new(mutton_id, 0, mutton_count));
            }

            // Zombie (32) & Husk (47)
            32 | 47 => {
                // Rotten Flesh: 0-2 + looting
                let count = rng.gen_range(0..=2) + looting_bonus(looting_level, &mut rng);
                if count > 0 {
                    drops.push(Item::new(367, 0, count));
                }
                // Rare drops if killed by player (2.5% base + 1% per looting)
                if killed_by_player {
                    let rare_chance = 0.025 + (looting_level as f64 * 0.01);
                    if rng.gen_bool(rare_chance.min(1.0)) {
                        match rng.gen_range(0..3) {
                            0 => drops.push(Item::new(265, 0, 1)), // Iron Ingot
                            1 => drops.push(Item::new(391, 0, 1)), // Carrot
                            _ => drops.push(Item::new(392, 0, 1)), // Potato
                        }
                    }
                }
            }

            // Creeper (33)
            33 => {
                // Gunpowder: 0-2 + looting
                let gunpowder = rng.gen_range(0..=2) + looting_bonus(looting_level, &mut rng);
                if gunpowder > 0 {
                    drops.push(Item::new(289, 0, gunpowder));
                }
            }

            // Skeleton (34) & Stray (46)
            34 | 46 => {
                // Arrows: 0-2 + looting
                let arrows = rng.gen_range(0..=2) + looting_bonus(looting_level, &mut rng);
                if arrows > 0 {
                    drops.push(Item::new(262, 0, arrows));
                }
                // Bones: 0-2 + looting
                let bones = rng.gen_range(0..=2) + looting_bonus(looting_level, &mut rng);
                if bones > 0 {
                    drops.push(Item::new(352, 0, bones));
                }
            }

            // Mooshroom (16)
            16 => {
                let leather = rng.gen_range(0..=2) + looting_bonus(looting_level, &mut rng);
                if leather > 0 {
                    drops.push(Item::new(334, 0, leather));
                }
                let beef_count = rng.gen_range(1..=3) + looting_bonus(looting_level, &mut rng);
                let beef_id = if on_fire { 364 } else { 363 };
                drops.push(Item::new(beef_id, 0, beef_count));
            }

            // Squid (17)
            17 => {
                let ink = rng.gen_range(1..=3) + looting_bonus(looting_level, &mut rng);
                drops.push(Item::new(351, 0, ink));
            }

            // Rabbit (18)
            18 => {
                let hide = rng.gen_range(0..=1) + looting_bonus(looting_level, &mut rng);
                if hide > 0 {
                    drops.push(Item::new(415, 0, hide));
                }
                let meat_id = if on_fire { 412 } else { 411 };
                drops.push(Item::new(meat_id, 0, 1));
            }

            // Iron Golem (20)
            20 => {
                let iron = rng.gen_range(3..=5);
                drops.push(Item::new(265, 0, iron));
                let poppy = rng.gen_range(0..=2);
                if poppy > 0 {
                    drops.push(Item::new(38, 0, poppy));
                }
            }

            // Snow Golem (21)
            21 => {
                let balls = rng.gen_range(0..=15);
                if balls > 0 {
                    drops.push(Item::new(332, 0, balls));
                }
            }

            // Spider (35) & Cave Spider (40)
            35 | 40 => {
                // String: 0-2 + looting
                let string = rng.gen_range(0..=2) + looting_bonus(looting_level, &mut rng);
                if string > 0 {
                    drops.push(Item::new(287, 0, string));
                }
                // Spider Eye: 0-1 if killed by player
                if killed_by_player {
                    let eye = rng.gen_range(0..=1) + looting_bonus(looting_level, &mut rng);
                    if eye > 0 {
                        drops.push(Item::new(375, 0, eye));
                    }
                }
            }

            // Zombie Pigman (36)
            36 => {
                // Rotten Flesh: 0-1 + looting
                let flesh = rng.gen_range(0..=1) + looting_bonus(looting_level, &mut rng);
                if flesh > 0 {
                    drops.push(Item::new(367, 0, flesh));
                }
                // Gold Nugget: 0-1 + looting
                let nugget = rng.gen_range(0..=1) + looting_bonus(looting_level, &mut rng);
                if nugget > 0 {
                    drops.push(Item::new(371, 0, nugget));
                }
                // Rare Gold Ingot (2.5%)
                if killed_by_player && rng.gen_bool(0.025 + (looting_level as f64 * 0.01)) {
                    drops.push(Item::new(266, 0, 1));
                }
            }

            // Slime (37)
            37 => {
                let slimeballs = rng.gen_range(0..=2) + looting_bonus(looting_level, &mut rng);
                if slimeballs > 0 {
                    drops.push(Item::new(341, 0, slimeballs));
                }
            }

            // Enderman (38)
            38 => {
                // Ender Pearl: 0-1 + looting
                let pearls = rng.gen_range(0..=1) + looting_bonus(looting_level, &mut rng);
                if pearls > 0 {
                    drops.push(Item::new(368, 0, pearls));
                }
            }

            // Ghast (41)
            41 => {
                let gunpowder = rng.gen_range(0..=2) + looting_bonus(looting_level, &mut rng);
                if gunpowder > 0 {
                    drops.push(Item::new(289, 0, gunpowder));
                }
                let tears = rng.gen_range(0..=1) + looting_bonus(looting_level, &mut rng);
                if tears > 0 {
                    drops.push(Item::new(370, 0, tears));
                }
            }

            // Magma Cube (42)
            42 => {
                let cream = rng.gen_range(0..=1) + looting_bonus(looting_level, &mut rng);
                if cream > 0 {
                    drops.push(Item::new(378, 0, cream));
                }
            }

            // Blaze (43)
            43 => {
                if killed_by_player {
                    let rods = rng.gen_range(0..=1) + looting_bonus(looting_level, &mut rng);
                    if rods > 0 {
                        drops.push(Item::new(369, 0, rods));
                    }
                }
            }

            // Witch (45)
            45 => {
                // Common witch drops: glass bottles, glowstone dust, redstone, sugar, sticks
                let pool = [374, 348, 331, 353, 280]; // bottle, glowstone, redstone, sugar, stick
                let count = rng.gen_range(1..=3);
                for _ in 0..count {
                    let item_id = pool[rng.gen_range(0..pool.len())];
                    let qty = rng.gen_range(1..=2) + looting_bonus(looting_level, &mut rng);
                    drops.push(Item::new(item_id, 0, qty));
                }
            }

            // Wither Skeleton (48)
            48 => {
                // Bones: 0-2
                let bones = rng.gen_range(0..=2) + looting_bonus(looting_level, &mut rng);
                if bones > 0 {
                    drops.push(Item::new(352, 0, bones));
                }
                // Coal: 0-1
                let coal = rng.gen_range(0..=1) + looting_bonus(looting_level, &mut rng);
                if coal > 0 {
                    drops.push(Item::new(263, 0, coal));
                }
                // Skull: 2.5% base + 1% per looting level
                if killed_by_player {
                    let chance = 0.025 + (looting_level as f64 * 0.01);
                    if rng.gen_bool(chance.min(1.0)) {
                        drops.push(Item::new(397, 1, 1)); // Wither Skeleton Skull
                    }
                }
            }

            // Wither Boss (52 in Bedrock)
            52 => {
                // Guaranteed Nether Star
                drops.push(Item::new(399, 0, 1));
            }

            // Shulker (54)
            54 => {
                let shells = rng.gen_range(0..=1) + looting_bonus(looting_level, &mut rng);
                if shells > 0 {
                    drops.push(Item::new(445, 0, shells));
                }
            }

            // Evoker (104 / 58)
            104 | 58 => {
                // Guaranteed Totem of Undying
                drops.push(Item::new(449, 0, 1));
                // Emeralds: 0-1
                let emeralds = rng.gen_range(0..=1) + looting_bonus(looting_level, &mut rng);
                if emeralds > 0 {
                    drops.push(Item::new(388, 0, emeralds));
                }
            }

            // Vindicator (105 / 57)
            105 | 57 => {
                let emeralds = rng.gen_range(0..=1) + looting_bonus(looting_level, &mut rng);
                if emeralds > 0 {
                    drops.push(Item::new(388, 0, emeralds));
                }
            }

            _ => {}
        }

        drops
    }

    /// Experience points awarded when killing an entity.
    pub fn xp_reward(entity_type: i32, size: u8) -> u32 {
        match entity_type {
            10 | 11 | 12 | 13 | 14 | 16 | 17 | 18 | 22 => 2,
            32 | 33 | 34 | 35 | 36 | 38 | 39 | 40 | 41 | 44 | 45 | 46 | 47 | 48 | 49 | 54 => 5,
            43 => 10,          // Blaze
            37 | 42 => (size as u32).max(1), // Slime / Magma Cube
            50 | 57 | 58 => 10, // Elder Guardian / Vindicator / Evoker
            52 => 50,          // Wither
            53 => 500,         // Ender Dragon
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chicken_drops_meat_and_feathers() {
        let drops_raw = LootTable::get_drops(10, true, false, 0);
        let has_raw = drops_raw.iter().any(|i| i.id == 365);
        assert!(has_raw, "Chicken must drop raw chicken");

        let drops_cooked = LootTable::get_drops(10, true, true, 0);
        let has_cooked = drops_cooked.iter().any(|i| i.id == 366);
        assert!(has_cooked, "Burning chicken must drop cooked chicken");
    }

    #[test]
    fn test_cow_drops_beef() {
        let drops = LootTable::get_drops(11, true, false, 0);
        let has_beef = drops.iter().any(|i| i.id == 363);
        assert!(has_beef, "Cow must drop beef");
    }

    #[test]
    fn test_wither_drops_nether_star() {
        let drops = LootTable::get_drops(52, true, false, 0);
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].id, 399); // Nether Star
    }

    #[test]
    fn test_evoker_drops_totem() {
        let drops = LootTable::get_drops(58, true, false, 0);
        let has_totem = drops.iter().any(|i| i.id == 449);
        assert!(has_totem, "Evoker must drop Totem of Undying");
    }
}
