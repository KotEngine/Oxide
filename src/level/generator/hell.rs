use crate::level::chunk::Chunk;
use crate::level::generator::noise::OctaveNoise;
use crate::level::generator::populator::OrePopulator;
use crate::utils::random;
use super::Generator;

pub struct HellGenerator {
    pub seed: u64,
    terrain_noise: OctaveNoise,
    glowstone_noise: OctaveNoise,
    quartz_populator: OrePopulator,
}

impl HellGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            terrain_noise: OctaveNoise::new(seed.wrapping_mul(73), 4),
            glowstone_noise: OctaveNoise::new(seed.wrapping_mul(137), 3),
            quartz_populator: OrePopulator::nether_quartz(),
        }
    }
}

impl Generator for HellGenerator {
    fn name(&self) -> &'static str {
        "hell"
    }

    fn generate_chunk(&self, chunk: &mut Chunk) {
        let base_x = (chunk.x * 16) as f64;
        let base_z = (chunk.z * 16) as f64;

        for x in 0..16 {
            for z in 0..16 {
                let wx = base_x + x as f64;
                let wz = base_z + z as f64;

                // Bedrock floor and ceiling
                chunk.set_block(x, 0, z, 7, 0);
                chunk.set_block(x, 127, z, 7, 0);

                // Irregular bedrock boundaries
                let r1 = ((wx * 13.0 + wz * 17.0).sin().abs() * 3.0) as usize;
                for y in 1..=r1 {
                    chunk.set_block(x, y, z, 7, 0);
                }
                let r2 = ((wx * 19.0 + wz * 23.0).cos().abs() * 3.0) as usize;
                for y in (127 - r2)..127 {
                    chunk.set_block(x, y, z, 7, 0);
                }

                // 3D density field for cavernous Nether terrain
                for y in 1..127usize {
                    let n = self.terrain_noise.noise3d(wx * 0.05, y as f64 * 0.05, wz * 0.05);

                    // Gradient: solid near floor and ceiling, open in center (y=32..80)
                    let dist_from_center = ((y as f64 - 64.0) / 64.0).abs();
                    let density = n + dist_from_center * 1.1;

                    if density > 0.42 {
                        // Solid Netherrack
                        chunk.set_block(x, y, z, 87, 0);
                    } else if y <= 31 {
                        // Lava sea in empty pockets below y=32
                        chunk.set_block(x, y, z, 11, 0);
                    } else {
                        // Air
                        chunk.set_block(x, y, z, 0, 0);
                    }
                }

                // Set Hell biome ID (8)
                chunk.biome_ids[(z << 4) | x] = 8;
            }
        }

        // Soul sand and gravel beaches around lava level (y = 31..33)
        for x in 0..16 {
            for z in 0..16 {
                let wx = base_x + x as f64;
                let wz = base_z + z as f64;
                for y in 31..=34usize {
                    if chunk.get_block_id(x, y, z) == 87 {
                        let sn = ((wx * 0.2).sin() + (wz * 0.2).cos()) * 0.5;
                        if sn > 0.3 {
                            chunk.set_block(x, y, z, 88, 0); // Soul Sand
                        } else if sn < -0.3 {
                            chunk.set_block(x, y, z, 13, 0); // Gravel
                        }
                    }
                }
            }
        }

        // Nether Fortress structure in chunk grid (every 14 chunks)
        if chunk.x.rem_euclid(14) == 0 && chunk.z.rem_euclid(14) == 0 {
            self.generate_fortress_section(chunk);
        }

        chunk.recalculate_height_map();
    }

    fn populate(&self, chunk: &mut Chunk) {
        let base_x = (chunk.x * 16) as f64;
        let base_z = (chunk.z * 16) as f64;

        // 1. Nether Quartz veins
        self.quartz_populator.populate(chunk);

        // 2. Glowstone clusters hanging from ceilings (y = 80..115)
        for _ in 0..4 {
            let cx = random::next_range(2, 14) as usize;
            let cz = random::next_range(2, 14) as usize;
            let n = self.glowstone_noise.noise2d(base_x + cx as f64, base_z + cz as f64);
            if n > -0.2 {
                for cy in (80..115usize).rev() {
                    if chunk.get_block_id(cx, cy, cz) == 87 && chunk.get_block_id(cx, cy - 1, cz) == 0 {
                        // Hang a glowstone stalactite/cluster
                        let cluster_len = random::next_range(1, 4) as usize;
                        for dy in 1..=cluster_len {
                            let gy = cy.saturating_sub(dy);
                            if gy > 32 {
                                chunk.set_block(cx, gy, cz, 89, 0); // Glowstone
                                for &(dx, dz) in &[(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                                    let nx = (cx as i32 + dx).max(0).min(15) as usize;
                                    let nz = (cz as i32 + dz).max(0).min(15) as usize;
                                    if random::next_range(0, 3) == 0 && chunk.get_block_id(nx, gy, nz) == 0 {
                                        chunk.set_block(nx, gy, nz, 89, 0);
                                    }
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }

        // 3. Natural Fire patches on exposed netherrack
        for _ in 0..12 {
            let fx = random::next_range(0, 16) as usize;
            let fz = random::next_range(0, 16) as usize;
            for fy in 32..110usize {
                if chunk.get_block_id(fx, fy, fz) == 87 && chunk.get_block_id(fx, fy + 1, fz) == 0 {
                    chunk.set_block(fx, fy + 1, fz, 51, 0); // Fire
                    break;
                }
            }
        }

        chunk.recalculate_height_map();
    }
}

impl HellGenerator {
    fn generate_fortress_section(&self, chunk: &mut Chunk) {
        // Massive Nether Fortress pillars and bridge walkway at y = 60..64
        let bridge_y = 60usize;

        // Bridge deck running east-west across chunk
        for x in 0..16 {
            for z in 6..=10 {
                chunk.set_block(x, bridge_y, z, 112, 0); // Nether Brick walkway
                chunk.set_block(x, bridge_y + 1, 6, 113, 0); // Nether Brick Fence railing
                chunk.set_block(x, bridge_y + 1, 10, 113, 0); // Nether Brick Fence railing

                // Clear interior
                for y in (bridge_y + 1)..=(bridge_y + 4) {
                    if z > 6 && z < 10 {
                        chunk.set_block(x, y, z, 0, 0);
                    }
                }
            }
        }

        // Support pillars down to lava or solid netherrack at x = 7..9, z = 7..9
        for x in 7..=9 {
            for z in 7..=9 {
                for y in (30..bridge_y).rev() {
                    let id = chunk.get_block_id(x, y, z);
                    chunk.set_block(x, y, z, 112, 0); // Nether Brick support
                    if id == 87 && y < 45 {
                        break;
                    }
                }
            }
        }
    }
}
