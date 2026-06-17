use super::sub_chunk::SubChunk;
use super::MAX_SUBCHUNKS;

pub const BIOMES_LEN: usize = 256;
pub const HEIGHTMAP_LEN: usize = 256;

pub struct Chunk {
    pub x: i32,
    pub z: i32,
    pub sub_chunks: Vec<SubChunk>,
    pub biome_ids: Vec<u8>,
    pub height_map: Vec<u8>,
}

impl Chunk {
    pub fn new(x: i32, z: i32) -> Self {
        let mut sub_chunks = Vec::with_capacity(MAX_SUBCHUNKS);
        for _ in 0..MAX_SUBCHUNKS {
            sub_chunks.push(SubChunk::new());
        }
        let mut height_map = vec![0u8; HEIGHTMAP_LEN];
        for v in height_map.iter_mut() {
            *v = (MAX_SUBCHUNKS * 16) as u8;
        }
        Self {
            x,
            z,
            sub_chunks,
            biome_ids: vec![0; BIOMES_LEN],
            height_map,
        }
    }

    pub fn get_block_id(&self, x: usize, y: usize, z: usize) -> u8 {
        let sy = y >> 4;
        if sy >= MAX_SUBCHUNKS { return 0; }
        self.sub_chunks[sy].get_block_id(x, y & 0x0f, z)
    }

    pub fn set_block_id(&mut self, x: usize, y: usize, z: usize, id: u8) {
        let sy = y >> 4;
        if sy >= MAX_SUBCHUNKS { return; }
        self.sub_chunks[sy].set_block_id(x, y & 0x0f, z, id);
    }

    pub fn get_block_data(&self, x: usize, y: usize, z: usize) -> u8 {
        let sy = y >> 4;
        if sy >= MAX_SUBCHUNKS { return 0; }
        self.sub_chunks[sy].get_block_data(x, y & 0x0f, z)
    }

    pub fn set_block_data(&mut self, x: usize, y: usize, z: usize, data: u8) {
        let sy = y >> 4;
        if sy >= MAX_SUBCHUNKS { return; }
        self.sub_chunks[sy].set_block_data(x, y & 0x0f, z, data);
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, id: u8, data: u8) {
        self.set_block_id(x, y, z, id);
        self.set_block_data(x, y, z, data);
    }

    pub fn network_serialize(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(MAX_SUBCHUNKS as u8);
        for sc in &self.sub_chunks {
            out.extend_from_slice(&sc.network_serialize());
        }
        out.extend_from_slice(&self.height_map);
        out.extend_from_slice(&self.biome_ids);
        out.push(0);
        out
    }

    pub fn generate_flat(&mut self) {
        for x in 0..16 {
            for z in 0..16 {
                for y in 0..3 {
                    if y == 0 {
                        self.set_block(x, y, z, 7, 0);
                    } else if y < 2 {
                        self.set_block(x, y, z, 3, 0);
                    } else {
                        self.set_block(x, y, z, 2, 0);
                    }
                }
            }
        }
    }
}
