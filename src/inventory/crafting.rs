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
}

impl CraftingManager {
    pub fn new() -> Self {
        Self { recipes: RwLock::new(HashMap::new()) }
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
}

impl Default for CraftingManager {
    fn default() -> Self {
        let m = Self::new();
        m.register_default_recipes();
        m
    }
}
