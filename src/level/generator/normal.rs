use crate::level::chunk::Chunk;
use crate::level::generator::noise::OctaveNoise;
use crate::level::generator::biome::{Biome, BiomeType};
use crate::level::generator::populator::{
    CavePopulator, GroundCoverPopulator, OrePopulator, TreePopulator,
    LakePopulator, DungeonPopulator,
};
use super::Generator;

pub struct NormalGenerator {
    height_noise: OctaveNoise,
    cave_populator: CavePopulator,
    water_lake: LakePopulator,
    lava_lake: LakePopulator,
    dungeon_populator: DungeonPopulator,
    ground_cover: GroundCoverPopulator,
    ores: Vec<OrePopulator>,
    seed: u64,
}

impl NormalGenerator {
    pub fn new(seed: u64) -> Self {
        let ores = vec![
            OrePopulator::dirt(),
            OrePopulator::gravel(),
            OrePopulator::granite(),
            OrePopulator::diorite(),
            OrePopulator::andesite(),
            OrePopulator::coal(),
            OrePopulator::iron(),
            OrePopulator::gold(),
            OrePopulator::redstone(),
            OrePopulator::lapis(),
            OrePopulator::diamond(),
            OrePopulator::emerald(),
        ];

        Self {
            height_noise: OctaveNoise::new(seed, 8),
            cave_populator: CavePopulator::new(seed),
            water_lake: LakePopulator::water(),
            lava_lake: LakePopulator::lava(),
            dungeon_populator: DungeonPopulator::new(),
            ground_cover: GroundCoverPopulator::new(),
            ores,
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
        if temp < -0.4 {
            &Biome::SNOWY_TUNDRA
        } else if temp > 0.5 && humid < -0.2 {
            &Biome::DESERT
        } else if temp > 0.4 && humid < 0.0 {
            &Biome::SAVANNA
        } else if humid > 0.4 && temp > 0.5 {
            &Biome::JUNGLE
        } else if humid > 0.3 && temp < 0.3 {
            &Biome::TAIGA
        } else if humid > 0.5 {
            &Biome::FOREST
        } else {
            &Biome::PLAINS
        }
    }
}

impl Generator for NormalGenerator {
    fn name(&self) -> &'static str {
        "normal"
    }

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
                        chunk.set_block(x, y, z, 7, 0); // Bedrock
                    } else if y + 4 < height_u {
                        chunk.set_block(x, y, z, 1, 0); // Stone
                    } else if y < height_u {
                        chunk.set_block(x, y, z, 3, 0); // Dirt
                    } else {
                        if biome.biome_type == BiomeType::Desert || biome.biome_type == BiomeType::Beach {
                            chunk.set_block(x, y, z, 12, 0); // Sand
                        } else if height_u < 62 {
                            chunk.set_block(x, y, z, 13, 0); // Gravel under sea level
                        } else {
                            chunk.set_block(x, y, z, 2, 0); // Grass
                        }
                    }
                }

                // Ocean / sea level water fill up to y=62
                if height_u < 62 {
                    for y in (height_u + 1)..=62 {
                        chunk.set_block(x, y, z, 9, 0); // Stationary water
                    }
                }

                // Snow layer for cold biomes
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
        // 1. Underground Cave carving
        self.cave_populator.populate(chunk);

        // 2. Underground Dungeons
        self.dungeon_populator.populate(chunk);

        // 3. Lakes (Water & Lava)
        self.water_lake.populate(chunk);
        self.lava_lake.populate(chunk);

        // 4. Ore veins
        for ore in &self.ores {
            ore.populate(chunk);
        }

        // 3. Tree population by biome
        let center_biome_id = chunk.biome_ids[136]; // Center (8, 8)
        if center_biome_id == Biome::FOREST.id {
            TreePopulator::oak().populate(chunk);
            TreePopulator::birch().populate(chunk);
        } else if center_biome_id == Biome::TAIGA.id {
            TreePopulator::spruce().populate(chunk);
        } else if center_biome_id == Biome::JUNGLE.id {
            TreePopulator::jungle().populate(chunk);
            TreePopulator::oak().populate(chunk);
        } else if center_biome_id == Biome::SAVANNA.id {
            TreePopulator::acacia().populate(chunk);
        } else if center_biome_id == Biome::PLAINS.id {
            let mut sparse_oak = TreePopulator::oak();
            sparse_oak.attempts = 1;
            sparse_oak.populate(chunk);
        }

        // 4. Ground foliage and cover (grass, flowers, cacti, sugar canes, pumpkins)
        self.ground_cover.populate(chunk);

        // 5. Final recalculation of lighting and height map
        chunk.recalculate_height_map();
        chunk.populate_sky_light();
    }
}
