use std::collections::HashMap;
use parking_lot::RwLock;
use crate::protocol::types::Item;

#[derive(Debug, Clone)]
pub struct Recipe {
    pub id: String,
    pub recipe_type: RecipeType,
    pub result: Item,
    pub ingredients: Vec<RecipeIngredient>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipeType {
    Shaped,
    Shapeless,
    Furnace,
    Multi,
    BigShaped,
    BigShapeless,
}

#[derive(Debug, Clone)]
pub struct RecipeIngredient {
    pub item: Item,
}

impl RecipeIngredient {
    pub fn new(item: Item) -> Self { Self { item } }
}

pub struct CraftingManager {
    recipes: RwLock<HashMap<String, Recipe>>,
    brewing_recipes: RwLock<HashMap<(i32, i32, i32), Item>>,
}

impl CraftingManager {
    pub fn new() -> Self {
        Self {
            recipes: RwLock::new(HashMap::new()),
            brewing_recipes: RwLock::new(HashMap::new()),
        }
    }

    pub fn add_brewing_recipe(&self, ingredient_id: i32, input_potion: Item, result: Item) {
        self.brewing_recipes
            .write()
            .insert((ingredient_id, input_potion.id, input_potion.meta), result);
    }

    pub fn match_brewing(&self, ingredient: &Item, potion: &Item) -> Option<Item> {
        if ingredient.is_air() || potion.is_air() {
            return None;
        }
        self.brewing_recipes
            .read()
            .get(&(ingredient.id, potion.id, potion.meta))
            .cloned()
    }

    pub fn add_recipe(&self, recipe: Recipe) {
        self.recipes.write().insert(recipe.id.clone(), recipe);
    }

    pub fn get_recipe(&self, id: &str) -> Option<Recipe> {
        self.recipes.read().get(id).cloned()
    }

    pub fn count(&self) -> usize {
        self.recipes.read().len()
    }

    pub fn match_shaped(&self, grid: &[Item], width: u32, height: u32) -> Option<Recipe> {
        let recipes = self.recipes.read();
        for recipe in recipes.values() {
            if recipe.recipe_type != RecipeType::Shaped && recipe.recipe_type != RecipeType::BigShaped {
                continue;
            }
            if recipe.width > width || recipe.height > height {
                continue;
            }

            for dy in 0..=(height - recipe.height) {
                'offset_loop: for dx in 0..=(width - recipe.width) {
                    for gy in 0..height {
                        for gx in 0..width {
                            let grid_idx = (gy * width + gx) as usize;
                            let grid_item = grid.get(grid_idx);

                            let in_recipe_x = gx >= dx && gx < dx + recipe.width;
                            let in_recipe_y = gy >= dy && gy < dy + recipe.height;

                            if in_recipe_x && in_recipe_y {
                                let rx = gx - dx;
                                let ry = gy - dy;
                                let rec_idx = (ry * recipe.width + rx) as usize;
                                let expected = recipe.ingredients.get(rec_idx);
                                match (expected, grid_item) {
                                    (Some(ing), Some(gi)) => {
                                        if ing.item.is_air() {
                                            if !gi.is_air() && gi.count > 0 { continue 'offset_loop; }
                                        } else {
                                            if gi.is_air() || gi.count == 0 || gi.id != ing.item.id {
                                                continue 'offset_loop;
                                            }
                                        }
                                    }
                                    _ => continue 'offset_loop,
                                }
                            } else {
                                if let Some(gi) = grid_item {
                                    if !gi.is_air() && gi.count > 0 {
                                        continue 'offset_loop;
                                    }
                                }
                            }
                        }
                    }
                    return Some(recipe.clone());
                }
            }
        }
        None
    }

    pub fn match_shapeless(&self, items: &[Item]) -> Option<Recipe> {
        let mut non_air_inputs: Vec<(i32, i32)> = items
            .iter()
            .filter(|i| !i.is_air() && i.count > 0)
            .map(|i| (i.id, i.meta as i32))
            .collect();
        non_air_inputs.sort();

        let recipes = self.recipes.read();
        for recipe in recipes.values() {
            if recipe.recipe_type != RecipeType::Shapeless && recipe.recipe_type != RecipeType::BigShapeless {
                continue;
            }
            let mut rec_ings: Vec<(i32, i32)> = recipe
                .ingredients
                .iter()
                .filter(|i| !i.item.is_air())
                .map(|i| (i.item.id, i.item.meta as i32))
                .collect();
            rec_ings.sort();

            if non_air_inputs.len() != rec_ings.len() {
                continue;
            }

            let mut matched = true;
            for (inp, rec) in non_air_inputs.iter().zip(rec_ings.iter()) {
                if inp.0 != rec.0 {
                    matched = false;
                    break;
                }
            }
            if matched {
                return Some(recipe.clone());
            }
        }
        None
    }

    pub fn match_furnace(&self, input: &Item) -> Option<Recipe> {
        if input.is_air() || input.count == 0 { return None; }
        let recipes = self.recipes.read();
        for recipe in recipes.values() {
            if recipe.recipe_type != RecipeType::Furnace {
                continue;
            }
            if let Some(ing) = recipe.ingredients.first() {
                if ing.item.id == input.id {
                    return Some(recipe.clone());
                }
            }
        }
        None
    }

    pub fn register_default_recipes(&self) {
        let planks_recipe = Recipe {
            id: "wood_to_planks".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(5, 0, 4),
            ingredients: vec![RecipeIngredient::new(Item::new(17, 0, 1))],
            width: 1, height: 1,
        };
        self.add_recipe(planks_recipe);

        let sticks_recipe = Recipe {
            id: "planks_to_sticks".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(280, 0, 4),
            ingredients: vec![
                RecipeIngredient::new(Item::new(5, 0, 1)),
                RecipeIngredient::new(Item::new(5, 0, 1)),
            ],
            width: 1, height: 2,
        };
        self.add_recipe(sticks_recipe);

        let torches_recipe = Recipe {
            id: "coal_to_torch".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(50, 0, 4),
            ingredients: vec![
                RecipeIngredient::new(Item::new(263, 0, 1)),
                RecipeIngredient::new(Item::new(280, 0, 1)),
            ],
            width: 1, height: 2,
        };
        self.add_recipe(torches_recipe);

        let crafting_table_recipe = Recipe {
            id: "planks_to_table".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(58, 0, 1),
            ingredients: vec![
                RecipeIngredient::new(Item::new(5, 0, 1)),
                RecipeIngredient::new(Item::new(5, 0, 1)),
                RecipeIngredient::new(Item::new(5, 0, 1)),
                RecipeIngredient::new(Item::new(5, 0, 1)),
            ],
            width: 2, height: 2,
        };
        self.add_recipe(crafting_table_recipe);

        let furnace_recipe = Recipe {
            id: "cobblestone_to_furnace".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(61, 0, 1),
            ingredients: (0..8).map(|_| RecipeIngredient::new(Item::new(4, 0, 1))).collect(),
            width: 3, height: 3,
        };
        self.add_recipe(furnace_recipe);

        let chest_recipe = Recipe {
            id: "planks_to_chest".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(54, 0, 1),
            ingredients: (0..8).map(|_| RecipeIngredient::new(Item::new(5, 0, 1))).collect(),
            width: 3, height: 3,
        };
        self.add_recipe(chest_recipe);

        self.add_wood_tool_recipes();
        self.add_stone_tool_recipes();
        self.add_iron_tool_recipes();
        self.add_diamond_tool_recipes();
        self.add_gold_tool_recipes();
        self.add_armor_recipes();
        self.add_furnace_recipes();
        self.add_combat_recipes();
        self.add_building_recipes();
        self.add_redstone_recipes();
        self.add_food_recipes();
    }

    fn add_wood_tool_recipes(&self) {
        self.add_pickaxe_recipe("wood_pickaxe", 270, 5);
        self.add_axe_recipe("wood_axe", 271, 5);
        self.add_shovel_recipe("wood_shovel", 273, 5);
        self.add_hoe_recipe("wood_hoe", 290, 5);
        self.add_sword_recipe("wood_sword", 268, 5);
    }

    fn add_stone_tool_recipes(&self) {
        self.add_pickaxe_recipe("stone_pickaxe", 274, 4);
        self.add_axe_recipe("stone_axe", 275, 4);
        self.add_shovel_recipe("stone_shovel", 277, 4);
        self.add_hoe_recipe("stone_hoe", 291, 4);
        self.add_sword_recipe("stone_sword", 272, 4);
    }

    fn add_iron_tool_recipes(&self) {
        self.add_pickaxe_recipe("iron_pickaxe", 257, 265);
        self.add_axe_recipe("iron_axe", 258, 265);
        self.add_shovel_recipe("iron_shovel", 256, 265);
        self.add_hoe_recipe("iron_hoe", 292, 265);
        self.add_sword_recipe("iron_sword", 267, 265);
    }

    fn add_diamond_tool_recipes(&self) {
        self.add_pickaxe_recipe("diamond_pickaxe", 278, 264);
        self.add_axe_recipe("diamond_axe", 279, 264);
        self.add_shovel_recipe("diamond_shovel", 277, 264);
        self.add_hoe_recipe("diamond_hoe", 293, 264);
        self.add_sword_recipe("diamond_sword", 276, 264);
    }

    fn add_gold_tool_recipes(&self) {
        self.add_pickaxe_recipe("gold_pickaxe", 285, 266);
        self.add_axe_recipe("gold_axe", 286, 266);
        self.add_shovel_recipe("gold_shovel", 284, 266);
        self.add_hoe_recipe("gold_hoe", 294, 266);
        self.add_sword_recipe("gold_sword", 283, 266);
    }

    fn add_pickaxe_recipe(&self, id: &str, result_id: i32, material_id: i32) {
        self.add_tool_recipe(id, result_id, material_id, 3, 0,
            &[(0, 0), (1, 0), (2, 0), (0, 1), (1, 2)]);
    }

    fn add_axe_recipe(&self, id: &str, result_id: i32, material_id: i32) {
        self.add_tool_recipe(id, result_id, material_id, 3, 0,
            &[(0, 0), (1, 0), (1, 1), (0, 2)]);
    }

    fn add_shovel_recipe(&self, id: &str, result_id: i32, material_id: i32) {
        self.add_tool_recipe(id, result_id, material_id, 1, 0, &[(0, 0)]);
    }

    fn add_hoe_recipe(&self, id: &str, result_id: i32, material_id: i32) {
        self.add_tool_recipe(id, result_id, material_id, 2, 0, &[(0, 0), (1, 0)]);
    }

    fn add_sword_recipe(&self, id: &str, result_id: i32, material_id: i32) {
        self.add_tool_recipe(id, result_id, material_id, 1, 0, &[(0, 0)]);
    }

    fn add_tool_recipe(&self, id: &str, result_id: i32, material_id: i32, _w: u32, _h: u32, _positions: &[(u32, u32)]) {
        self.add_recipe(Recipe {
            id: id.to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(result_id, 0, 1),
            ingredients: (0..3).map(|_| RecipeIngredient::new(Item::new(material_id, 0, 1))).collect(),
            width: 3, height: 3,
        });
    }

    fn add_armor_recipes(&self) {
        for (mat, prefix, helmet, chest, legs, boots) in [
            (265, "iron", 306, 307, 308, 309),
            (264, "diamond", 310, 311, 312, 313),
            (266, "gold", 314, 315, 316, 317),
        ] {
            for &(result, count) in &[(helmet, 5), (chest, 8), (legs, 7), (boots, 4)] {
                self.add_recipe(Recipe {
                    id: format!("{}_armor_{}", prefix, result),
                    recipe_type: RecipeType::Shaped,
                    result: Item::new(result, 0, 1),
                    ingredients: (0..count).map(|_| RecipeIngredient::new(Item::new(mat, 0, 1))).collect(),
                    width: 3, height: 3,
                });
            }
        }
    }

    fn add_furnace_recipes(&self) {
        for &(id, input_id, output_id) in &[
            ("smelt_iron", 15, 265),
            ("smelt_gold", 14, 266),
            ("smelt_stone", 4, 1),
            ("smelt_glass", 12, 20),
            ("smelt_beef", 363, 364),
            ("smelt_pork", 319, 320),
            ("smelt_chicken", 365, 366),
            ("smelt_brick", 337, 336),
            ("smelt_charcoal", 17, 263),
        ] {
            self.add_recipe(Recipe {
                id: id.to_string(),
                recipe_type: RecipeType::Furnace,
                result: Item::new(output_id, 0, 1),
                ingredients: vec![RecipeIngredient::new(Item::new(input_id, 0, 1))],
                width: 1,
                height: 1,
            });
        }
    }

    fn add_combat_recipes(&self) {
        // Bow: 3 sticks (280) + 3 strings (287)
        self.add_recipe(Recipe {
            id: "craft_bow".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(261, 0, 1),
            ingredients: vec![
                RecipeIngredient::new(Item::new(280, 0, 1)),
                RecipeIngredient::new(Item::new(280, 0, 1)),
                RecipeIngredient::new(Item::new(280, 0, 1)),
                RecipeIngredient::new(Item::new(287, 0, 1)),
                RecipeIngredient::new(Item::new(287, 0, 1)),
                RecipeIngredient::new(Item::new(287, 0, 1)),
            ],
            width: 3,
            height: 3,
        });

        // Arrow (4x): 1 flint (318) + 1 stick (280) + 1 feather (288)
        self.add_recipe(Recipe {
            id: "craft_arrow".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(262, 0, 4),
            ingredients: vec![
                RecipeIngredient::new(Item::new(318, 0, 1)),
                RecipeIngredient::new(Item::new(280, 0, 1)),
                RecipeIngredient::new(Item::new(288, 0, 1)),
            ],
            width: 1,
            height: 3,
        });

        // TNT: 5 gunpowder (289) + 4 sand (12)
        self.add_recipe(Recipe {
            id: "craft_tnt".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(46, 0, 1),
            ingredients: vec![
                RecipeIngredient::new(Item::new(289, 0, 1)),
                RecipeIngredient::new(Item::new(12, 0, 1)),
                RecipeIngredient::new(Item::new(289, 0, 1)),
                RecipeIngredient::new(Item::new(12, 0, 1)),
                RecipeIngredient::new(Item::new(289, 0, 1)),
                RecipeIngredient::new(Item::new(12, 0, 1)),
                RecipeIngredient::new(Item::new(289, 0, 1)),
                RecipeIngredient::new(Item::new(12, 0, 1)),
                RecipeIngredient::new(Item::new(289, 0, 1)),
            ],
            width: 3,
            height: 3,
        });

        // Golden apple: 8 gold ingots (266) + 1 apple (260)
        self.add_recipe(Recipe {
            id: "craft_golden_apple".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(322, 0, 1),
            ingredients: vec![
                RecipeIngredient::new(Item::new(266, 0, 1)),
                RecipeIngredient::new(Item::new(266, 0, 1)),
                RecipeIngredient::new(Item::new(266, 0, 1)),
                RecipeIngredient::new(Item::new(266, 0, 1)),
                RecipeIngredient::new(Item::new(260, 0, 1)),
                RecipeIngredient::new(Item::new(266, 0, 1)),
                RecipeIngredient::new(Item::new(266, 0, 1)),
                RecipeIngredient::new(Item::new(266, 0, 1)),
                RecipeIngredient::new(Item::new(266, 0, 1)),
            ],
            width: 3,
            height: 3,
        });
    }

    fn add_building_recipes(&self) {
        // Wooden door (3x): 6 planks (5)
        self.add_recipe(Recipe {
            id: "craft_wooden_door".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(324, 0, 3),
            ingredients: (0..6).map(|_| RecipeIngredient::new(Item::new(5, 0, 1))).collect(),
            width: 2,
            height: 3,
        });

        // Iron door: 6 iron ingots (265)
        self.add_recipe(Recipe {
            id: "craft_iron_door".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(330, 0, 1),
            ingredients: (0..6).map(|_| RecipeIngredient::new(Item::new(265, 0, 1))).collect(),
            width: 2,
            height: 3,
        });

        // Wooden trapdoor (2x): 6 planks (5)
        self.add_recipe(Recipe {
            id: "craft_wooden_trapdoor".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(96, 0, 2),
            ingredients: (0..6).map(|_| RecipeIngredient::new(Item::new(5, 0, 1))).collect(),
            width: 3,
            height: 2,
        });

        // Ladder (3x): 7 sticks (280)
        self.add_recipe(Recipe {
            id: "craft_ladder".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(65, 0, 3),
            ingredients: (0..7).map(|_| RecipeIngredient::new(Item::new(280, 0, 1))).collect(),
            width: 3,
            height: 3,
        });

        // Fence (3x): 4 planks (5) + 2 sticks (280)
        self.add_recipe(Recipe {
            id: "craft_fence".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(85, 0, 3),
            ingredients: vec![
                RecipeIngredient::new(Item::new(5, 0, 1)),
                RecipeIngredient::new(Item::new(280, 0, 1)),
                RecipeIngredient::new(Item::new(5, 0, 1)),
                RecipeIngredient::new(Item::new(5, 0, 1)),
                RecipeIngredient::new(Item::new(280, 0, 1)),
                RecipeIngredient::new(Item::new(5, 0, 1)),
            ],
            width: 3,
            height: 2,
        });

        // Fence gate: 2 planks (5) + 4 sticks (280)
        self.add_recipe(Recipe {
            id: "craft_fence_gate".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(107, 0, 1),
            ingredients: vec![
                RecipeIngredient::new(Item::new(280, 0, 1)),
                RecipeIngredient::new(Item::new(5, 0, 1)),
                RecipeIngredient::new(Item::new(280, 0, 1)),
                RecipeIngredient::new(Item::new(280, 0, 1)),
                RecipeIngredient::new(Item::new(5, 0, 1)),
                RecipeIngredient::new(Item::new(280, 0, 1)),
            ],
            width: 3,
            height: 2,
        });

        // Bucket: 3 iron ingots (265)
        self.add_recipe(Recipe {
            id: "craft_bucket".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(325, 0, 1),
            ingredients: (0..3).map(|_| RecipeIngredient::new(Item::new(265, 0, 1))).collect(),
            width: 3,
            height: 2,
        });

        // Shears: 2 iron ingots (265)
        self.add_recipe(Recipe {
            id: "craft_shears".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(359, 0, 1),
            ingredients: vec![
                RecipeIngredient::new(Item::new(265, 0, 1)),
                RecipeIngredient::new(Item::new(265, 0, 1)),
            ],
            width: 2,
            height: 2,
        });

        // Flint and steel: 1 iron ingot (265) + 1 flint (318)
        self.add_recipe(Recipe {
            id: "craft_flint_and_steel".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(259, 0, 1),
            ingredients: vec![
                RecipeIngredient::new(Item::new(265, 0, 1)),
                RecipeIngredient::new(Item::new(318, 0, 1)),
            ],
            width: 2,
            height: 1,
        });

        // Bed: 3 wool (35) + 3 planks (5)
        self.add_recipe(Recipe {
            id: "craft_bed".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(355, 0, 1),
            ingredients: vec![
                RecipeIngredient::new(Item::new(35, 0, 1)),
                RecipeIngredient::new(Item::new(35, 0, 1)),
                RecipeIngredient::new(Item::new(35, 0, 1)),
                RecipeIngredient::new(Item::new(5, 0, 1)),
                RecipeIngredient::new(Item::new(5, 0, 1)),
                RecipeIngredient::new(Item::new(5, 0, 1)),
            ],
            width: 3,
            height: 2,
        });

        // Glass bottle (3x): 3 glass (20)
        self.add_recipe(Recipe {
            id: "craft_glass_bottle".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(374, 0, 3),
            ingredients: (0..3).map(|_| RecipeIngredient::new(Item::new(20, 0, 1))).collect(),
            width: 3,
            height: 2,
        });

        // Anvil: 3 iron blocks (42) + 4 iron ingots (265)
        self.add_recipe(Recipe {
            id: "craft_anvil".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(145, 0, 1),
            ingredients: vec![
                RecipeIngredient::new(Item::new(42, 0, 1)),
                RecipeIngredient::new(Item::new(42, 0, 1)),
                RecipeIngredient::new(Item::new(42, 0, 1)),
                RecipeIngredient::new(Item::new(265, 0, 1)),
                RecipeIngredient::new(Item::new(265, 0, 1)),
                RecipeIngredient::new(Item::new(265, 0, 1)),
                RecipeIngredient::new(Item::new(265, 0, 1)),
            ],
            width: 3,
            height: 3,
        });

        // Enchanting table: 1 book (340) + 2 diamonds (264) + 4 obsidian (49)
        self.add_recipe(Recipe {
            id: "craft_enchanting_table".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(116, 0, 1),
            ingredients: vec![
                RecipeIngredient::new(Item::new(340, 0, 1)),
                RecipeIngredient::new(Item::new(264, 0, 1)),
                RecipeIngredient::new(Item::new(264, 0, 1)),
                RecipeIngredient::new(Item::new(49, 0, 1)),
                RecipeIngredient::new(Item::new(49, 0, 1)),
                RecipeIngredient::new(Item::new(49, 0, 1)),
                RecipeIngredient::new(Item::new(49, 0, 1)),
            ],
            width: 3,
            height: 3,
        });
    }

    fn add_redstone_recipes(&self) {
        // Redstone torch: 1 redstone (331) + 1 stick (280)
        self.add_recipe(Recipe {
            id: "craft_redstone_torch".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(76, 0, 1),
            ingredients: vec![
                RecipeIngredient::new(Item::new(331, 0, 1)),
                RecipeIngredient::new(Item::new(280, 0, 1)),
            ],
            width: 1,
            height: 2,
        });

        // Lever: 1 stick (280) + 1 cobblestone (4)
        self.add_recipe(Recipe {
            id: "craft_lever".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(69, 0, 1),
            ingredients: vec![
                RecipeIngredient::new(Item::new(280, 0, 1)),
                RecipeIngredient::new(Item::new(4, 0, 1)),
            ],
            width: 1,
            height: 2,
        });

        // Stone button: 1 stone (1)
        self.add_recipe(Recipe {
            id: "craft_stone_button".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(77, 0, 1),
            ingredients: vec![RecipeIngredient::new(Item::new(1, 0, 1))],
            width: 1,
            height: 1,
        });

        // Wooden button: 1 plank (5)
        self.add_recipe(Recipe {
            id: "craft_wooden_button".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(143, 0, 1),
            ingredients: vec![RecipeIngredient::new(Item::new(5, 0, 1))],
            width: 1,
            height: 1,
        });

        // Wooden pressure plate: 2 planks (5)
        self.add_recipe(Recipe {
            id: "craft_wooden_pressure_plate".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(72, 0, 1),
            ingredients: vec![
                RecipeIngredient::new(Item::new(5, 0, 1)),
                RecipeIngredient::new(Item::new(5, 0, 1)),
            ],
            width: 2,
            height: 1,
        });

        // Stone pressure plate: 2 stone (1)
        self.add_recipe(Recipe {
            id: "craft_stone_pressure_plate".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(70, 0, 1),
            ingredients: vec![
                RecipeIngredient::new(Item::new(1, 0, 1)),
                RecipeIngredient::new(Item::new(1, 0, 1)),
            ],
            width: 2,
            height: 1,
        });

        // Hopper: 5 iron ingots (265) + 1 chest (54)
        self.add_recipe(Recipe {
            id: "craft_hopper".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(154, 0, 1),
            ingredients: vec![
                RecipeIngredient::new(Item::new(265, 0, 1)),
                RecipeIngredient::new(Item::new(265, 0, 1)),
                RecipeIngredient::new(Item::new(265, 0, 1)),
                RecipeIngredient::new(Item::new(54, 0, 1)),
                RecipeIngredient::new(Item::new(265, 0, 1)),
                RecipeIngredient::new(Item::new(265, 0, 1)),
            ],
            width: 3,
            height: 3,
        });

        // Piston: 3 planks (5) + 4 cobblestone (4) + 1 iron ingot (265) + 1 redstone (331)
        self.add_recipe(Recipe {
            id: "craft_piston".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(33, 0, 1),
            ingredients: vec![
                RecipeIngredient::new(Item::new(5, 0, 1)),
                RecipeIngredient::new(Item::new(5, 0, 1)),
                RecipeIngredient::new(Item::new(5, 0, 1)),
                RecipeIngredient::new(Item::new(4, 0, 1)),
                RecipeIngredient::new(Item::new(265, 0, 1)),
                RecipeIngredient::new(Item::new(4, 0, 1)),
                RecipeIngredient::new(Item::new(4, 0, 1)),
                RecipeIngredient::new(Item::new(331, 0, 1)),
                RecipeIngredient::new(Item::new(4, 0, 1)),
            ],
            width: 3,
            height: 3,
        });
    }

    fn add_food_recipes(&self) {
        // Bread: 3 wheat (296)
        self.add_recipe(Recipe {
            id: "craft_bread".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(297, 0, 1),
            ingredients: vec![
                RecipeIngredient::new(Item::new(296, 0, 1)),
                RecipeIngredient::new(Item::new(296, 0, 1)),
                RecipeIngredient::new(Item::new(296, 0, 1)),
            ],
            width: 3,
            height: 1,
        });

        // Sugar: 1 sugar cane (338)
        self.add_recipe(Recipe {
            id: "craft_sugar".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(353, 0, 1),
            ingredients: vec![RecipeIngredient::new(Item::new(338, 0, 1))],
            width: 1,
            height: 1,
        });

        // Paper (3x): 3 sugar canes (338)
        self.add_recipe(Recipe {
            id: "craft_paper".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(339, 0, 3),
            ingredients: vec![
                RecipeIngredient::new(Item::new(338, 0, 1)),
                RecipeIngredient::new(Item::new(338, 0, 1)),
                RecipeIngredient::new(Item::new(338, 0, 1)),
            ],
            width: 3,
            height: 1,
        });

        // Book: 3 paper (339) + 1 leather (334)
        self.add_recipe(Recipe {
            id: "craft_book".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(340, 0, 1),
            ingredients: vec![
                RecipeIngredient::new(Item::new(339, 0, 1)),
                RecipeIngredient::new(Item::new(339, 0, 1)),
                RecipeIngredient::new(Item::new(339, 0, 1)),
                RecipeIngredient::new(Item::new(334, 0, 1)),
            ],
            width: 2,
            height: 2,
        });

        // Bowl (4x): 3 planks (5)
        self.add_recipe(Recipe {
            id: "craft_bowl".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(281, 0, 4),
            ingredients: vec![
                RecipeIngredient::new(Item::new(5, 0, 1)),
                RecipeIngredient::new(Item::new(5, 0, 1)),
                RecipeIngredient::new(Item::new(5, 0, 1)),
            ],
            width: 3,
            height: 2,
        });

        // Mushroom stew: 1 bowl (281) + 1 red mushroom (40) + 1 brown mushroom (39)
        self.add_recipe(Recipe {
            id: "craft_mushroom_stew".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(282, 0, 1),
            ingredients: vec![
                RecipeIngredient::new(Item::new(40, 0, 1)),
                RecipeIngredient::new(Item::new(39, 0, 1)),
                RecipeIngredient::new(Item::new(281, 0, 1)),
            ],
            width: 2,
            height: 2,
        });

        // Pumpkin pie: 1 pumpkin (86) + 1 sugar (353) + 1 egg (344)
        self.add_recipe(Recipe {
            id: "craft_pumpkin_pie".to_string(),
            recipe_type: RecipeType::Shaped,
            result: Item::new(400, 0, 1),
            ingredients: vec![
                RecipeIngredient::new(Item::new(86, 0, 1)),
                RecipeIngredient::new(Item::new(353, 0, 1)),
                RecipeIngredient::new(Item::new(344, 0, 1)),
            ],
            width: 2,
            height: 2,
        });

        self.register_default_brewing_recipes();
    }

    fn register_default_brewing_recipes(&self) {
        // Potion meta constants matching Bedrock 1.1.5
        const WATER: i32 = 0;
        const MUNDANE: i32 = 1;
        const MUNDANE_EXT: i32 = 2;
        const THICK: i32 = 3;
        const AWKWARD: i32 = 4;
        const NIGHT_VISION: i32 = 5;
        const NIGHT_VISION_T: i32 = 6;
        const INVISIBILITY: i32 = 7;
        const INVISIBILITY_T: i32 = 8;
        const LEAPING: i32 = 9;
        const LEAPING_T: i32 = 10;
        const LEAPING_TWO: i32 = 11;
        const FIRE_RESISTANCE: i32 = 12;
        const FIRE_RESISTANCE_T: i32 = 13;
        const SWIFTNESS: i32 = 14;
        const SWIFTNESS_T: i32 = 15;
        const SWIFTNESS_TWO: i32 = 16;
        const SLOWNESS: i32 = 17;
        const SLOWNESS_T: i32 = 18;
        const WATER_BREATHING: i32 = 19;
        const WATER_BREATHING_T: i32 = 20;
        const HEALING: i32 = 21;
        const HEALING_TWO: i32 = 22;
        const HARMING: i32 = 23;
        const HARMING_TWO: i32 = 24;
        const POISON: i32 = 25;
        const POISON_T: i32 = 26;
        const POISON_TWO: i32 = 27;
        const REGENERATION: i32 = 28;
        const REGENERATION_T: i32 = 29;
        const REGENERATION_TWO: i32 = 30;
        const STRENGTH: i32 = 31;
        const STRENGTH_T: i32 = 32;
        const STRENGTH_TWO: i32 = 33;
        const WEAKNESS: i32 = 34;
        const WEAKNESS_T: i32 = 35;

        // Ingredients
        const GUNPOWDER: i32 = 289;
        const REDSTONE: i32 = 331;
        const GLOWSTONE_DUST: i32 = 348;
        const SUGAR: i32 = 353;
        const GHAST_TEAR: i32 = 370;
        const NETHER_WART: i32 = 372;
        const SPIDER_EYE: i32 = 375;
        const FERMENTED_EYE: i32 = 376;
        const BLAZE_POWDER: i32 = 377;
        const MAGMA_CREAM: i32 = 378;
        const GLISTERING_MELON: i32 = 382;
        const GOLDEN_CARROT: i32 = 396;
        const RABBIT_FOOT: i32 = 414;
        const PUFFERFISH: i32 = 462;

        // Register for both normal Potion (373) and Splash Potion (438)
        for pid in [373, 438] {
            let p = |m: i32| Item::new(pid, m, 1);

            self.add_brewing_recipe(NETHER_WART, p(WATER), p(AWKWARD));
            self.add_brewing_recipe(GLOWSTONE_DUST, p(WATER), p(THICK));
            self.add_brewing_recipe(REDSTONE, p(WATER), p(MUNDANE_EXT));
            self.add_brewing_recipe(FERMENTED_EYE, p(WATER), p(WEAKNESS));
            self.add_brewing_recipe(GHAST_TEAR, p(WATER), p(MUNDANE));
            self.add_brewing_recipe(GLISTERING_MELON, p(WATER), p(MUNDANE));
            self.add_brewing_recipe(BLAZE_POWDER, p(WATER), p(MUNDANE));
            self.add_brewing_recipe(MAGMA_CREAM, p(WATER), p(MUNDANE));
            self.add_brewing_recipe(SUGAR, p(WATER), p(MUNDANE));
            self.add_brewing_recipe(SPIDER_EYE, p(WATER), p(MUNDANE));
            self.add_brewing_recipe(RABBIT_FOOT, p(WATER), p(MUNDANE));

            self.add_brewing_recipe(FERMENTED_EYE, p(MUNDANE), p(WEAKNESS));
            self.add_brewing_recipe(FERMENTED_EYE, p(THICK), p(WEAKNESS));
            self.add_brewing_recipe(FERMENTED_EYE, p(MUNDANE_EXT), p(WEAKNESS_T));
            self.add_brewing_recipe(REDSTONE, p(WEAKNESS), p(WEAKNESS_T));

            self.add_brewing_recipe(GHAST_TEAR, p(AWKWARD), p(REGENERATION));
            self.add_brewing_recipe(REDSTONE, p(REGENERATION), p(REGENERATION_T));
            self.add_brewing_recipe(GLOWSTONE_DUST, p(REGENERATION), p(REGENERATION_TWO));

            self.add_brewing_recipe(BLAZE_POWDER, p(AWKWARD), p(STRENGTH));
            self.add_brewing_recipe(REDSTONE, p(STRENGTH), p(STRENGTH_T));
            self.add_brewing_recipe(GLOWSTONE_DUST, p(STRENGTH), p(STRENGTH_TWO));

            self.add_brewing_recipe(SUGAR, p(AWKWARD), p(SWIFTNESS));
            self.add_brewing_recipe(REDSTONE, p(SWIFTNESS), p(SWIFTNESS_T));
            self.add_brewing_recipe(GLOWSTONE_DUST, p(SWIFTNESS), p(SWIFTNESS_TWO));

            self.add_brewing_recipe(MAGMA_CREAM, p(AWKWARD), p(FIRE_RESISTANCE));
            self.add_brewing_recipe(REDSTONE, p(FIRE_RESISTANCE), p(FIRE_RESISTANCE_T));

            self.add_brewing_recipe(GLISTERING_MELON, p(AWKWARD), p(HEALING));
            self.add_brewing_recipe(GLOWSTONE_DUST, p(HEALING), p(HEALING_TWO));

            self.add_brewing_recipe(SPIDER_EYE, p(AWKWARD), p(POISON));
            self.add_brewing_recipe(REDSTONE, p(POISON), p(POISON_T));
            self.add_brewing_recipe(GLOWSTONE_DUST, p(POISON), p(POISON_TWO));

            self.add_brewing_recipe(RABBIT_FOOT, p(AWKWARD), p(LEAPING));
            self.add_brewing_recipe(REDSTONE, p(LEAPING), p(LEAPING_T));
            self.add_brewing_recipe(GLOWSTONE_DUST, p(LEAPING), p(LEAPING_TWO));

            self.add_brewing_recipe(PUFFERFISH, p(AWKWARD), p(WATER_BREATHING));
            self.add_brewing_recipe(REDSTONE, p(WATER_BREATHING), p(WATER_BREATHING_T));

            self.add_brewing_recipe(GOLDEN_CARROT, p(AWKWARD), p(NIGHT_VISION));
            self.add_brewing_recipe(REDSTONE, p(NIGHT_VISION), p(NIGHT_VISION_T));
            self.add_brewing_recipe(FERMENTED_EYE, p(NIGHT_VISION), p(INVISIBILITY));
            self.add_brewing_recipe(FERMENTED_EYE, p(NIGHT_VISION_T), p(INVISIBILITY_T));
            self.add_brewing_recipe(REDSTONE, p(INVISIBILITY), p(INVISIBILITY_T));

            self.add_brewing_recipe(FERMENTED_EYE, p(SWIFTNESS), p(SLOWNESS));
            self.add_brewing_recipe(FERMENTED_EYE, p(LEAPING), p(SLOWNESS));
            self.add_brewing_recipe(FERMENTED_EYE, p(FIRE_RESISTANCE), p(SLOWNESS));
            self.add_brewing_recipe(FERMENTED_EYE, p(SWIFTNESS_T), p(SLOWNESS_T));
            self.add_brewing_recipe(FERMENTED_EYE, p(LEAPING_T), p(SLOWNESS_T));
            self.add_brewing_recipe(FERMENTED_EYE, p(FIRE_RESISTANCE_T), p(SLOWNESS_T));
            self.add_brewing_recipe(REDSTONE, p(SLOWNESS), p(SLOWNESS_T));

            self.add_brewing_recipe(FERMENTED_EYE, p(HEALING), p(HARMING));
            self.add_brewing_recipe(FERMENTED_EYE, p(POISON), p(HARMING));
            self.add_brewing_recipe(FERMENTED_EYE, p(HEALING_TWO), p(HARMING_TWO));
            self.add_brewing_recipe(FERMENTED_EYE, p(POISON_TWO), p(HARMING_TWO));
            self.add_brewing_recipe(GLOWSTONE_DUST, p(HARMING), p(HARMING_TWO));
        }

        for meta in 0..=36 {
            self.add_brewing_recipe(GUNPOWDER, Item::new(373, meta, 1), Item::new(438, meta, 1));
        }
    }
}

impl Default for CraftingManager {
    fn default() -> Self {
        let m = Self::new();
        m.register_default_recipes();
        m
    }
}
