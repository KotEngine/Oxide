use crate::level::chunk::Chunk;
use rand::Rng;

/// Populator that generates small surface and underground water and lava lakes.
pub struct LakePopulator {
    pub liquid_id: u8,
    pub min_y: i32,
    pub max_y: i32,
    pub rarity: u32, // 1 in rarity chunks
}

impl LakePopulator {
    pub fn water() -> Self {
        Self {
            liquid_id: 9, // Stationary water
            min_y: 35,
            max_y: 75,
            rarity: 4, // 1 in 4 chunks
        }
    }

    pub fn lava() -> Self {
        Self {
            liquid_id: 11, // Stationary lava
            min_y: 12,
            max_y: 45,
            rarity: 10, // 1 in 10 chunks
        }
    }

    pub fn populate(&self, chunk: &mut Chunk) {
        let mut rng = rand::thread_rng();
        if !rng.gen_bool(1.0 / self.rarity as f64) {
            return;
        }

        let center_x: i32 = rng.gen_range(4..12);
        let center_z: i32 = rng.gen_range(4..12);
        let center_y: i32 = rng.gen_range(self.min_y..=self.max_y);

        // Ensure center block is not air and not void/bedrock
        let cur_id = chunk.get_block_id(center_x as usize, center_y as usize, center_z as usize);
        if cur_id == 0 || cur_id == 7 {
            return;
        }

        let radius_x: i32 = rng.gen_range(3..=5);
        let radius_z: i32 = rng.gen_range(3..=5);
        let depth: i32 = rng.gen_range(2..=3);

        for dx in -radius_x..=radius_x {
            for dz in -radius_z..=radius_z {
                let lx = center_x + dx;
                let lz = center_z + dz;
                if lx < 0 || lx >= 16 || lz < 0 || lz >= 16 {
                    continue;
                }

                let dist_sq = (dx as f32 / radius_x as f32).powi(2) + (dz as f32 / radius_z as f32).powi(2);
                if dist_sq > 1.0 {
                    continue;
                }

                // Carve out liquid in bottom and air in top
                for dy in -depth..=2 {
                    let ly = center_y + dy;
                    if ly <= 1 || ly >= 127 {
                        continue;
                    }

                    if dy <= 0 {
                        chunk.set_block(lx as usize, ly as usize, lz as usize, self.liquid_id, 0);
                    } else {
                        chunk.set_block(lx as usize, ly as usize, lz as usize, 0, 0); // Air
                    }
                }

                // Place stone or sand/gravel bottom
                let bottom_y = center_y - depth - 1;
                if bottom_y > 0 && chunk.get_block_id(lx as usize, bottom_y as usize, lz as usize) != 7 {
                    let border_block = if self.liquid_id == 9 && rng.gen_bool(0.4) {
                        13 // Gravel
                    } else if self.liquid_id == 9 && rng.gen_bool(0.3) {
                        12 // Sand
                    } else {
                        1 // Stone
                    };
                    chunk.set_block(lx as usize, bottom_y as usize, lz as usize, border_block, 0);
                }
            }
        }
    }
}
