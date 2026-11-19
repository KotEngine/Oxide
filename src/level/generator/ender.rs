use crate::level::chunk::Chunk;
use crate::level::generator::noise::OctaveNoise;
use super::Generator;

pub struct EnderGenerator {
    pub seed: u64,
    noise: OctaveNoise,
}

impl EnderGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            noise: OctaveNoise::new(seed.wrapping_mul(199), 3),
        }
    }
}

impl Generator for EnderGenerator {
    fn name(&self) -> &'static str {
        "ender"
    }

    fn generate_chunk(&self, chunk: &mut Chunk) {
        let base_x = (chunk.x * 16) as f64;
        let base_z = (chunk.z * 16) as f64;

        // Radius squared from center of the End island
        let dist_sq = (chunk.x * 16 + 8).pow(2) + (chunk.z * 16 + 8).pow(2);

        // Island reaches out to ~80 blocks radius (dist_sq <= 6400)
        if dist_sq < 8100 {
            for x in 0..16 {
                for z in 0..16 {
                    let wx = base_x + x as f64;
                    let wz = base_z + z as f64;
                    let block_dist = (wx * wx + wz * wz).sqrt();

                    if block_dist < 80.0 {
                        let n = self.noise.noise2d(wx * 0.05, wz * 0.05) * 8.0;
                        // Island dome top at ~65, bottom at ~50
                        let max_y = (65.0 + n - (block_dist / 80.0).powi(2) * 20.0).max(45.0) as usize;
                        let min_y = (50.0 - n * 0.5 + (block_dist / 80.0).powi(2) * 15.0).max(40.0) as usize;

                        if max_y > min_y {
                            for y in min_y..=max_y {
                                chunk.set_block(x, y, z, 121, 0); // End Stone
                            }
                        }
                    }

                    // End Biome ID is 9
                    chunk.biome_ids[(z << 4) | x] = 9;
                }
            }
        } else {
            for x in 0..16 {
                for z in 0..16 {
                    chunk.biome_ids[(z << 4) | x] = 9;
                }
            }
        }

        // Obsidian spawn platform at (-42, 49, 0)
        if chunk.x == -3 && chunk.z == 0 {
            // wx in [-48..-32], wz in [0..15]
            for x in 3..=8 {
                for z in 0..=4 {
                    chunk.set_block(x, 49, z, 49, 0); // Obsidian floor
                    for y in 50..=53 {
                        chunk.set_block(x, y, z, 0, 0); // Air clearance
                    }
                }
            }
        }

        chunk.recalculate_height_map();
    }

    fn populate(&self, chunk: &mut Chunk) {
        let base_x = chunk.x * 16;
        let base_z = chunk.z * 16;

        // 8 Obsidian pillars at radius 42 around center
        let pillar_angles = [
            0.0f64,
            std::f64::consts::FRAC_PI_4,
            std::f64::consts::FRAC_PI_2,
            3.0 * std::f64::consts::FRAC_PI_4,
            std::f64::consts::PI,
            5.0 * std::f64::consts::FRAC_PI_4,
            3.0 * std::f64::consts::FRAC_PI_2,
            7.0 * std::f64::consts::FRAC_PI_4,
        ];

        let pillar_heights = [76usize, 82, 88, 94, 78, 85, 91, 98];

        for (i, &angle) in pillar_angles.iter().enumerate() {
            let px = (42.0 * angle.cos()).round() as i32;
            let pz = (42.0 * angle.sin()).round() as i32;
            let height = pillar_heights[i];

            // Check if this pillar intersects the current chunk
            if px >= base_x - 2 && px <= base_x + 18 && pz >= base_z - 2 && pz <= base_z + 18 {
                let radius = 2i32; // 5x5 circular pillar
                for dx in -radius..=radius {
                    for dz in -radius..=radius {
                        if dx * dx + dz * dz <= 5 {
                            let world_bx = px + dx;
                            let world_bz = pz + dz;
                            if world_bx >= base_x && world_bx < base_x + 16 && world_bz >= base_z && world_bz < base_z + 16 {
                                let cx = (world_bx - base_x) as usize;
                                let cz = (world_bz - base_z) as usize;

                                for y in 50..=height {
                                    chunk.set_block(cx, y, cz, 49, 0); // Obsidian
                                }

                                // Top center gets bedrock + fire
                                if dx == 0 && dz == 0 {
                                    chunk.set_block(cx, height, cz, 7, 0); // Bedrock
                                    if height + 1 < 256 {
                                        chunk.set_block(cx, height + 1, cz, 51, 0); // Fire
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        chunk.recalculate_height_map();
    }
}
