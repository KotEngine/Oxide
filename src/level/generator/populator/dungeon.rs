use crate::level::chunk::Chunk;
use rand::Rng;

/// Populator that generates underground monster room dungeons with cobblestone, mossy cobblestone, and spawners.
pub struct DungeonPopulator {
    pub rarity: u32,
}

impl Default for DungeonPopulator {
    fn default() -> Self {
        Self::new()
    }
}

impl DungeonPopulator {
    pub fn new() -> Self {
        Self { rarity: 16 } // 1 in 16 chunks
    }

    pub fn populate(&self, chunk: &mut Chunk) {
        let mut rng = rand::thread_rng();
        if !rng.gen_bool(1.0 / self.rarity as f64) {
            return;
        }

        let cx: i32 = rng.gen_range(5..11);
        let cz: i32 = rng.gen_range(5..11);
        let cy: i32 = rng.gen_range(16..48);

        // Ensure surrounding is solid stone
        if chunk.get_block_id(cx as usize, cy as usize, cz as usize) != 1 {
            return;
        }

        let half_w: i32 = rng.gen_range(2..=3); // 5x5 or 7x7
        let half_l: i32 = rng.gen_range(2..=3);
        let height: i32 = 3;

        // Build room walls, floor, and ceiling
        for dx in -half_w..=half_w {
            for dz in -half_l..=half_l {
                let bx = cx + dx;
                let bz = cz + dz;
                if bx < 0 || bx >= 16 || bz < 0 || bz >= 16 {
                    continue;
                }

                for dy in 0..=height {
                    let by = cy + dy;
                    if by <= 1 || by >= 127 {
                        continue;
                    }

                    let is_wall = dx == -half_w || dx == half_w || dz == -half_l || dz == half_l;
                    let is_floor_or_ceil = dy == 0 || dy == height;

                    if is_floor_or_ceil || is_wall {
                        // Floor, ceiling, or wall: Cobblestone (4) or Mossy Cobblestone (48)
                        let mat = if rng.gen_bool(0.4) { 48 } else { 4 };
                        chunk.set_block(bx as usize, by as usize, bz as usize, mat, 0);
                    } else {
                        // Hollow inside
                        chunk.set_block(bx as usize, by as usize, bz as usize, 0, 0);
                    }
                }
            }
        }

        // Place Monster Spawner (52) in center floor
        let spawner_y = cy + 1;
        if spawner_y < 127 {
            chunk.set_block(cx as usize, spawner_y as usize, cz as usize, 52, 0);
        }

        // Place 1 or 2 chests (54) against a wall
        let chest_x = cx + half_w - 1;
        let chest_z = cz;
        if chest_x < 16 && spawner_y < 127 {
            chunk.set_block(chest_x as usize, spawner_y as usize, chest_z as usize, 54, 0);
        }
    }
}
