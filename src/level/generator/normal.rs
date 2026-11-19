use crate::level::chunk::Chunk;
use crate::level::generator::noise::OctaveNoise;
use crate::level::generator::biome::Biome;
use super::Generator;

pub struct NormalGenerator {
    height_noise: OctaveNoise,
    cave_noise: OctaveNoise,
    ore_noise: OctaveNoise,
    seed: u64,
}

impl NormalGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            height_noise: OctaveNoise::new(seed, 8),
            cave_noise: OctaveNoise::new(seed.wrapping_mul(31), 4),
            ore_noise: OctaveNoise::new(seed.wrapping_mul(97), 4),
            seed,
        }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    fn height_at(&self, x: f64, z: f64) -> i32 {
        let n = self.height_noise.noise2d(x / 200.0, z / 200.0);
        let n2 = self.height_noise.noise2d(x / 50.0, z / 50.0) * 0.3;
        let base = 64.0;
        let h = base + (n + n2) * 32.0;
        h.max(1.0).min(200.0) as i32
    }

    fn biome_at(&self, x: f64, z: f64) -> &'static Biome {
        let temp = self.height_noise.noise2d(x / 400.0, z / 400.0);
        let humid = self.height_noise.noise2d((x + 1000.0) / 400.0, (z + 1000.0) / 400.0);
        if temp < -0.4 { &Biome::SNOWY_TUNDRA }
        else if temp > 0.5 && humid < -0.2 { &Biome::DESERT }
        else if temp > 0.4 && humid < 0.0 { &Biome::SAVANNA }
        else if humid > 0.4 && temp > 0.5 { &Biome::JUNGLE }
        else if humid > 0.3 && temp < 0.3 { &Biome::TAIGA }
        else if humid > 0.5 { &Biome::FOREST }
        else { &Biome::PLAINS }
    }
}

impl Generator for NormalGenerator {
    fn name(&self) -> &'static str { "normal" }

    fn generate_chunk(&self, chunk: &mut Chunk) {
        let base_x = (chunk.x * 16) as f64;
        let base_z = (chunk.z * 16) as f64;

        for x in 0..16 {
            for z in 0..16 {
                let world_x = base_x + x as f64;
                let world_z = base_z + z as f64;
                let height = self.height_at(world_x, world_z);
                let height_u = (height.max(1).min(254)) as usize;
                let biome = self.biome_at(world_x, world_z);

                for y in 0..=height_u {
                    if y == 0 {
                        chunk.set_block(x, y, z, 7, 0);
                    } else if y + 4 < height_u {
                        chunk.set_block(x, y, z, 1, 0);
                    } else if y < height_u {
                        chunk.set_block(x, y, z, 3, 0);
                    } else {
                        if biome.biome_type == crate::level::generator::biome::BiomeType::Desert
                            || biome.biome_type == crate::level::generator::biome::BiomeType::Beach {
                            chunk.set_block(x, y, z, 12, 0);
                        } else if height_u < 62 {
                            chunk.set_block(x, y, z, 13, 0);
                        } else {
                            chunk.set_block(x, y, z, 2, 0);
                        }
                    }
                }

                if height_u < 62 {
                    for y in (height_u + 1)..=62 {
                        chunk.set_block(x, y, z, 9, 0);
                    }
                }

                if biome.snow && height_u >= 62 && height_u + 1 < 256 {
                    chunk.set_block(x, height_u + 1, z, 78, 0);
                }

                chunk.biome_ids[(z << 4) | x] = biome.id;
            }
        }
        chunk.recalculate_height_map();
        chunk.populate_sky_light();
    }

    fn populate(&self, chunk: &mut Chunk) {
        let cave_n = self.cave_noise.noise3d(chunk.x as f64 * 0.5, 0.0, chunk.z as f64 * 0.5);
        if cave_n > 0.4 {
            let cx = 8;
            let cz = 8;
            for y in 10..50usize {
                let n = self.cave_noise.noise3d(cx as f64, y as f64 * 0.1, cz as f64);
                if n > 0.6 {
                    chunk.set_block(cx, y, cz, 0, 0);
                }
            }
        }

        for x in 0..16 {
            for z in 0..16 {
                let wx = (chunk.x * 16 + x as i32) as f64;
                let wz = (chunk.z * 16 + z as i32) as f64;
                for y in 1..64usize {
                    if chunk.get_block_id(x, y, z) == 1 {
                        let n = self.ore_noise.noise3d(wx * 0.1, y as f64 * 0.1, wz * 0.1);
                        if n > 0.75 {
                            if y < 16 && n > 0.85 {
                                chunk.set_block(x, y, z, 56, 0);
                            } else if y < 16 && n > 0.80 {
                                chunk.set_block(x, y, z, 73, 0);
                            } else if y < 32 && n > 0.82 {
                                chunk.set_block(x, y, z, 14, 0);
                            } else if y < 55 && n > 0.77 {
                                chunk.set_block(x, y, z, 15, 0);
                            } else {
                                chunk.set_block(x, y, z, 16, 0);
                            }
                        }
                    }
                }
            }
        }
    }
}
