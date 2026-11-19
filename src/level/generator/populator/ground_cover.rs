use crate::level::chunk::Chunk;
use crate::utils::random;

pub struct GroundCoverPopulator;

impl Default for GroundCoverPopulator {
    fn default() -> Self {
        Self::new()
    }
}

impl GroundCoverPopulator {
    pub fn new() -> Self {
        Self
    }

    pub fn populate(&self, chunk: &mut Chunk) {
        for x in 0..16 {
            for z in 0..16 {
                let mut top_y = 0;
                for y in (1..255).rev() {
                    let id = chunk.get_block_id(x, y, z);
                    if id != 0 && id != 78 { // Not air and not snow layer
                        top_y = y;
                        break;
                    }
                }

                if top_y < 1 || top_y >= 254 {
                    continue;
                }

                let top_id = chunk.get_block_id(x, top_y, z);
                let above_y = top_y + 1;

                if chunk.get_block_id(x, above_y, z) != 0 {
                    continue;
                }

                // 1. Grass Block (2) -> Tall grass, flowers, pumpkins
                if top_id == 2 {
                    let roll = random::next_range(0, 100);
                    if roll < 45 {
                        // Tall grass (31, data 1)
                        chunk.set_block(x, above_y, z, 31, 1);
                    } else if roll < 49 {
                        // Yellow flower / Dandelion (37)
                        chunk.set_block(x, above_y, z, 37, 0);
                    } else if roll < 53 {
                        // Red flower / Poppy (38, data 0..8)
                        let flower_data = random::next_range(0, 9) as u8;
                        chunk.set_block(x, above_y, z, 38, flower_data);
                    } else if roll == 54 {
                        // Rare pumpkin patch
                        chunk.set_block(x, above_y, z, 86, 0);
                    }
                }
                // 2. Sand (12) -> Cactus, dead bush
                else if top_id == 12 {
                    let roll = random::next_range(0, 100);
                    if roll < 3 {
                        // Dead bush (32)
                        chunk.set_block(x, above_y, z, 32, 0);
                    } else if roll < 6 {
                        // Cactus (81) height 1 to 3
                        let height = random::next_range(1, 4) as usize;
                        for dy in 0..height {
                            if above_y + dy < 256 {
                                chunk.set_block(x, above_y + dy, z, 81, 0);
                            }
                        }
                    }
                }

                // 3. Sugar cane (83) on sand/dirt/grass adjacent to water
                if (top_id == 2 || top_id == 3 || top_id == 12) && chunk.get_block_id(x, above_y, z) == 0 {
                    let mut adjacent_water = false;
                    for &(dx, dz) in &[(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                        let nx = x as i32 + dx;
                        let nz = z as i32 + dz;
                        if nx >= 0 && nx < 16 && nz >= 0 && nz < 16 {
                            let adj_id = chunk.get_block_id(nx as usize, top_y, nz as usize);
                            if adj_id == 8 || adj_id == 9 {
                                adjacent_water = true;
                                break;
                            }
                        }
                    }

                    if adjacent_water && random::next_range(0, 10) < 4 {
                        let cane_height = random::next_range(1, 4) as usize;
                        for dy in 0..cane_height {
                            if above_y + dy < 256 {
                                chunk.set_block(x, above_y + dy, z, 83, 0);
                            }
                        }
                    }
                }
            }
        }
    }
}
