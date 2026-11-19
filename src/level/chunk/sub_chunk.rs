pub const BLOCK_IDS_LEN: usize = 4096;
pub const BLOCK_DATA_LEN: usize = 2048;
pub const BLOCK_LIGHT_LEN: usize = 2048;
pub const SKY_LIGHT_LEN: usize = 2048;

#[derive(Clone)]
pub struct SubChunk {
    pub block_ids: Vec<u8>,
    pub block_data: Vec<u8>,
    pub block_light: Vec<u8>,
    pub sky_light: Vec<u8>,
}

impl Default for SubChunk {
    fn default() -> Self { Self::new() }
}

impl SubChunk {
    pub fn new() -> Self {
        Self {
            block_ids: vec![0; BLOCK_IDS_LEN],
            block_data: vec![0; BLOCK_DATA_LEN],
            block_light: vec![0; BLOCK_LIGHT_LEN],
            sky_light: vec![0xff; SKY_LIGHT_LEN],
        }
    }

    pub fn empty() -> Self {
        Self {
            block_ids: vec![0; BLOCK_IDS_LEN],
            block_data: vec![0; BLOCK_DATA_LEN],
            block_light: vec![0; BLOCK_LIGHT_LEN],
            sky_light: vec![0xff; SKY_LIGHT_LEN],
        }
    }

    /// Empty section where all block IDs are air (0).
    pub fn is_empty(&self) -> bool {
        self.block_ids.iter().all(|&id| id == 0)
    }

    #[inline]
    fn index(x: usize, y: usize, z: usize) -> usize {
        (x << 8) | (z << 4) | y
    }

    pub fn get_block_id(&self, x: usize, y: usize, z: usize) -> u8 {
        self.block_ids[Self::index(x, y, z)]
    }

    pub fn set_block_id(&mut self, x: usize, y: usize, z: usize, id: u8) {
        self.block_ids[Self::index(x, y, z)] = id;
    }

    pub fn get_block_data(&self, x: usize, y: usize, z: usize) -> u8 {
        let i = Self::index(x, y, z);
        (self.block_data[i >> 1] >> ((i & 1) * 4)) & 0x0f
    }

    pub fn set_block_data(&mut self, x: usize, y: usize, z: usize, data: u8) {
        let i = Self::index(x, y, z);
        let shift = (i & 1) * 4;
        let cur = self.block_data[i >> 1];
        self.block_data[i >> 1] = (cur & !(0x0f << shift)) | ((data & 0x0f) << shift);
    }

    pub fn network_serialize(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(1 + BLOCK_IDS_LEN + BLOCK_DATA_LEN + BLOCK_LIGHT_LEN + SKY_LIGHT_LEN);
        out.push(0);
        out.extend_from_slice(&self.block_ids);
        out.extend_from_slice(&self.block_data);
        out.extend_from_slice(&self.sky_light);
        out.extend_from_slice(&self.block_light);
        out
    }
}
