use super::sub_chunk::SubChunk;
use super::MAX_SUBCHUNKS;

pub const BIOMES_LEN: usize = 256;
pub const HEIGHTMAP_LEN: usize = 256;

pub struct Chunk {
    pub x: i32,
    pub z: i32,
    pub sub_chunks: Vec<SubChunk>,
    pub biome_ids: Vec<u8>,
    /// Column height map. Serialized to network as 256 little-endian u16 values.
    pub height_map: Vec<u16>,
}

impl Chunk {
    pub fn new(x: i32, z: i32) -> Self {
        let mut sub_chunks = Vec::with_capacity(MAX_SUBCHUNKS);
        for _ in 0..MAX_SUBCHUNKS {
            sub_chunks.push(SubChunk::new());
        }
        let height_map = vec![256u16; HEIGHTMAP_LEN];
        Self {
            x,
            z,
            sub_chunks,
            biome_ids: vec![1; BIOMES_LEN],
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

    pub fn get_sky_light(&self, x: usize, y: usize, z: usize) -> u8 {
        let sy = y >> 4;
        if sy >= MAX_SUBCHUNKS { return 15; }
        self.sub_chunks[sy].get_sky_light(x, y & 0x0f, z)
    }

    pub fn set_sky_light(&mut self, x: usize, y: usize, z: usize, level: u8) {
        let sy = y >> 4;
        if sy >= MAX_SUBCHUNKS { return; }
        self.sub_chunks[sy].set_sky_light(x, y & 0x0f, z, level);
    }

    pub fn recalculate_height_map(&mut self) {
        for z in 0..16 {
            for x in 0..16 {
                let mut top = 0u16;
                for y in (0..256).rev() {
                    if self.get_block_id(x, y, z) != 0 {
                        top = (y + 1) as u16;
                        break;
                    }
                }
                self.height_map[(z << 4) | x] = top;
            }
        }
    }

    pub fn populate_sky_light(&mut self) {
        for z in 0..16 {
            for x in 0..16 {
                let top = self.height_map[(z << 4) | x] as usize;
                for y in 0..top {
                    self.set_sky_light(x, y, z, 0);
                }
                for y in top..256 {
                    self.set_sky_light(x, y, z, 15);
                }
            }
        }
    }

    /// Number of subchunks to serialize:
    /// index of the highest non-empty subchunk + 1.
    pub fn sub_chunk_send_count(&self) -> usize {
        for y in (0..MAX_SUBCHUNKS).rev() {
            if !self.sub_chunks[y].is_empty() {
                return y + 1;
            }
        }
        0
    }

    pub fn network_serialize(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let count = self.sub_chunk_send_count();
        out.push(count as u8);
        for sc in self.sub_chunks.iter().take(count) {
            out.extend_from_slice(&sc.network_serialize());
        }
        // Heightmap: 256 little-endian shorts (512 bytes)
        for h in &self.height_map {
            out.extend_from_slice(&h.to_le_bytes());
        }
        out.extend_from_slice(&self.biome_ids);
        out.push(0); // Border blocks count
        out.push(0); // Extra data count (varint)
        out
    }

    pub fn generate_flat(&mut self) {
        for x in 0..16 {
            for z in 0..16 {
                self.set_block(x, 0, z, 7, 0); // Bedrock
                self.set_block(x, 1, z, 3, 0); // Dirt
                self.set_block(x, 2, z, 3, 0); // Dirt
                self.set_block(x, 3, z, 2, 0); // Grass
                self.height_map[(z << 4) | x] = 4;
                self.biome_ids[(z << 4) | x] = 1; // Plains
            }
        }
        self.populate_sky_light();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Network layout test: verifies byte alignment and serialization output.
    /// count(1) + count*(1+4096+2048+2048+2048) + heightMap(512) + biomes(256)
    /// + border(1) + extraData varint(1)
    #[test]
    fn network_layout_serialization() {
        let mut c = Chunk::new(0, 0);
        c.generate_flat();

        let sub = c.sub_chunk_send_count();
        assert_eq!(sub, 1, "Flat chunk must send 1 subchunk section");

        let data = c.network_serialize();
        let expected = 1 + sub * (1 + 4096 + 2048 + 2048 + 2048) + 512 + 256 + 1 + 1;
        assert_eq!(data.len(), expected, "Chunk payload length must match specification");

        assert_eq!(data[0], 1, "Subchunk section count");
        assert_eq!(data[1], 0, "Subchunk version");
        // Block (0,0,0) = bedrock(7), (0,1,0) = dirt(3), (0,2,0) = dirt(3), (0,3,0) = grass(2)
        assert_eq!(data[2], 7);
        assert_eq!(data[3], 3);
        assert_eq!(data[4], 3);
        assert_eq!(data[5], 2);

        // Heightmap: column 0 has height 4 (little-endian)
        let hm = 1 + sub * 10241;
        assert_eq!(&data[hm..hm + 2], &[4, 0], "Column height");
        // Trailing bytes: border count and extra data count
        assert_eq!(&data[data.len() - 2..], &[0, 0]);
    }
}
