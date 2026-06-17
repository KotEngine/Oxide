use std::collections::HashMap;
use crate::level::chunk::Chunk;

pub struct World {
    pub chunks: HashMap<(i32, i32), Chunk>,
    pub spawn_x: i32,
    pub spawn_y: i32,
    pub spawn_z: i32,
    pub time: i32,
}

impl Default for World {
    fn default() -> Self { Self::new() }
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            spawn_x: 0,
            spawn_y: 64,
            spawn_z: 0,
            time: 0,
        }
    }

    pub fn get_chunk_or_generate(&mut self, x: i32, z: i32) -> &mut Chunk {
        if !self.chunks.contains_key(&(x, z)) {
            let mut chunk = Chunk::new(x, z);
            chunk.generate_flat();
            self.chunks.insert((x, z), chunk);
        }
        self.chunks.get_mut(&(x, z)).unwrap()
    }

    pub fn get_chunk(&self, x: i32, z: i32) -> Option<&Chunk> {
        self.chunks.get(&(x, z))
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

    pub fn serialize_chunk(&mut self, x: i32, z: i32) -> Vec<u8> {
        let chunk = self.get_chunk_or_generate(x, z);
        chunk.network_serialize()
    }

    pub fn tick(&mut self) {
        self.time = (self.time + 1) % 24000;
    }
}
