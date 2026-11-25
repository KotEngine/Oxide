use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use crate::level::chunk::Chunk;
use crate::level::generator::{
    Generator, FlatGenerator, NormalGenerator, VoidGenerator, HellGenerator, EnderGenerator,
};
use crate::level::storage;

pub struct World {
    pub chunks: HashMap<(i32, i32), Chunk>,
    pub spawn_x: i32,
    pub spawn_y: i32,
    pub spawn_z: i32,
    pub time: i32,
    pub generator: Box<dyn Generator>,
    pub world_dir: Option<PathBuf>,
    pub dirty_chunks: HashSet<(i32, i32)>,
    pub level_name: String,
    pub seed: u64,
    pub generator_name: String,
}

impl Default for World {
    fn default() -> Self { Self::new() }
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            spawn_x: 0,
            spawn_y: 4,
            spawn_z: 0,
            time: 0,
            generator: Box::new(FlatGenerator::new()),
            world_dir: None,
            dirty_chunks: HashSet::new(),
            level_name: "world".to_string(),
            seed: 0,
            generator_name: "flat".to_string(),
        }
    }

    pub fn with_generator(generator_type: &str, seed: u64) -> Self {
        Self::with_generator_and_dir(generator_type, seed, None, "world")
    }

    pub fn with_generator_and_dir(
        generator_type: &str,
        seed: u64,
        world_dir: Option<PathBuf>,
        level_name: &str,
    ) -> Self {
        let mut effective_gen = generator_type.to_string();
        let mut effective_seed = seed;
        let mut loaded_dat = None;
        if let Some(ref dir) = world_dir {
            if let Some(dat) = storage::load_level_dat(dir) {
                if !dat.generator_name.is_empty() {
                    effective_gen = dat.generator_name.clone();
                }
                effective_seed = dat.seed as u64;
                loaded_dat = Some(dat);
            }
        }

        let (spawn_y, generator, gen_label): (i32, Box<dyn Generator>, String) =
            if effective_gen.eq_ignore_ascii_case("normal") || effective_gen.eq_ignore_ascii_case("default") {
                (68, Box::new(NormalGenerator::new(effective_seed)), "default".to_string())
            } else if effective_gen.eq_ignore_ascii_case("hell") || effective_gen.eq_ignore_ascii_case("nether") {
                (64, Box::new(HellGenerator::new(effective_seed)), "hell".to_string())
            } else if effective_gen.eq_ignore_ascii_case("ender")
                || effective_gen.eq_ignore_ascii_case("the_end")
                || effective_gen.eq_ignore_ascii_case("end")
            {
                (65, Box::new(EnderGenerator::new(effective_seed)), "ender".to_string())
            } else if effective_gen.eq_ignore_ascii_case("void") {
                (64, Box::new(VoidGenerator::new()), "void".to_string())
            } else {
                (4, Box::new(FlatGenerator::new()), "flat".to_string())
            };

        let mut world = Self {
            chunks: HashMap::new(),
            spawn_x: 0,
            spawn_y,
            spawn_z: 0,
            time: 0,
            generator,
            world_dir,
            dirty_chunks: HashSet::new(),
            level_name: level_name.to_string(),
            seed: effective_seed,
            generator_name: gen_label,
        };

        if let Some(dat) = loaded_dat {
            world.spawn_x = dat.spawn_x;
            world.spawn_y = dat.spawn_y;
            world.spawn_z = dat.spawn_z;
            world.time = dat.time;
            if !dat.level_name.is_empty() {
                world.level_name = dat.level_name;
            }
        }

        world
    }

    pub fn get_chunk_or_generate(&mut self, x: i32, z: i32) -> &mut Chunk {
        if !self.chunks.contains_key(&(x, z)) {
            if let Some(ref dir) = self.world_dir {
                if let Some(loaded) = storage::load_chunk_from_disk(dir, x, z) {
                    self.chunks.insert((x, z), loaded);
                    return self.chunks.get_mut(&(x, z)).unwrap();
                }
            }
            let mut chunk = Chunk::new(x, z);
            self.generator.generate_chunk(&mut chunk);
            self.generator.populate(&mut chunk);
            self.chunks.insert((x, z), chunk);
        }
        self.chunks.get_mut(&(x, z)).unwrap()
    }

    pub fn get_chunk(&self, x: i32, z: i32) -> Option<&Chunk> {
        self.chunks.get(&(x, z))
    }

    pub fn mark_dirty(&mut self, cx: i32, cz: i32) {
        self.dirty_chunks.insert((cx, cz));
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, id: u8, data: u8) {
        if y < 0 || y >= 256 { return; }
        let chunk_x = x >> 4;
        let chunk_z = z >> 4;
        let cx = (x & 0x0f) as usize;
        let cz = (z & 0x0f) as usize;
        let cy = y as usize;
        let chunk = self.get_chunk_or_generate(chunk_x, chunk_z);
        chunk.set_block(cx, cy, cz, id, data);
        self.dirty_chunks.insert((chunk_x, chunk_z));
    }

    pub fn get_block_id(&mut self, x: i32, y: i32, z: i32) -> u8 {
        if y < 0 || y >= 256 { return 0; }
        let chunk_x = x >> 4;
        let chunk_z = z >> 4;
        let cx = (x & 0x0f) as usize;
        let cz = (z & 0x0f) as usize;
        let cy = y as usize;
        self.get_chunk_or_generate(chunk_x, chunk_z).get_block_id(cx, cy, cz)
    }

    pub fn get_block_id_opt(&self, x: i32, y: i32, z: i32) -> u8 {
        if y < 0 || y >= 256 { return 0; }
        let chunk_x = x >> 4;
        let chunk_z = z >> 4;
        let cx = (x & 0x0f) as usize;
        let cz = (z & 0x0f) as usize;
        let cy = y as usize;
        self.get_chunk(chunk_x, chunk_z).map(|c| c.get_block_id(cx, cy, cz)).unwrap_or(0)
    }

    pub fn get_block_data_opt(&self, x: i32, y: i32, z: i32) -> u8 {
        if y < 0 || y >= 256 { return 0; }
        self.get_chunk(x >> 4, z >> 4)
            .map(|c| c.get_block_data((x & 0x0f) as usize, y as usize, (z & 0x0f) as usize))
            .unwrap_or(0)
    }

    pub fn get_full_light_opt(&self, x: i32, y: i32, z: i32) -> u8 {
        if y >= 256 { return 15; }
        if y < 0 { return 0; }
        self.get_chunk(x >> 4, z >> 4)
            .map(|c| c.get_full_light((x & 0x0f) as usize, y as usize, (z & 0x0f) as usize))
            .unwrap_or(15)
    }

    pub fn serialize_chunk(&mut self, x: i32, z: i32) -> Vec<u8> {
        let chunk = self.get_chunk_or_generate(x, z);
        chunk.network_serialize()
    }

    pub fn serialize_chunk_with_tiles(&mut self, x: i32, z: i32, tiles: &[Vec<u8>]) -> Vec<u8> {
        let chunk = self.get_chunk_or_generate(x, z);
        chunk.network_serialize_with_tiles(tiles)
    }

    pub fn save_all(&mut self) -> usize {
        let dir = match self.world_dir.clone() {
            Some(d) => d,
            None => return 0,
        };
        let dat = storage::LevelDat {
            level_name: self.level_name.clone(),
            spawn_x: self.spawn_x,
            spawn_y: self.spawn_y,
            spawn_z: self.spawn_z,
            time: self.time,
            seed: self.seed as i64,
            generator_name: self.generator_name.clone(),
        };
        let _ = storage::save_level_dat(&dir, &dat);

        let dirty: Vec<(i32, i32)> = self.dirty_chunks.drain().collect();
        let mut saved = 0usize;
        for (cx, cz) in dirty {
            if let Some(chunk) = self.chunks.get(&(cx, cz)) {
                if storage::save_chunk_to_disk(&dir, chunk).is_ok() {
                    saved += 1;
                }
            }
        }
        saved
    }

    pub fn tick(&mut self) {
        self.time = (self.time + 1) % 24000;
    }
}
