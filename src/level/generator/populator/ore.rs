use crate::level::chunk::Chunk;
use crate::utils::random;

pub struct OrePopulator {
    pub ore_type: u8,
    pub ore_data: u8,
    pub min_y: usize,
    pub max_y: usize,
    pub size: usize,
    pub attempts: usize,
}

impl OrePopulator {
    pub fn coal() -> Self { Self { ore_type: 16, ore_data: 0, min_y: 5, max_y: 64, size: 12, attempts: 20 } }
    pub fn iron() -> Self { Self { ore_type: 15, ore_data: 0, min_y: 3, max_y: 48, size: 8, attempts: 20 } }
    pub fn gold() -> Self { Self { ore_type: 14, ore_data: 0, min_y: 1, max_y: 32, size: 6, attempts: 8 } }
    pub fn redstone() -> Self { Self { ore_type: 73, ore_data: 0, min_y: 1, max_y: 24, size: 6, attempts: 6 } }
    pub fn lapis() -> Self { Self { ore_type: 21, ore_data: 0, min_y: 1, max_y: 32, size: 5, attempts: 8 } }
    pub fn diamond() -> Self { Self { ore_type: 56, ore_data: 0, min_y: 1, max_y: 20, size: 4, attempts: 4 } }
    pub fn emerald() -> Self { Self { ore_type: 129, ore_data: 0, min_y: 1, max_y: 32, size: 1, attempts: 2 } }

    pub fn populate(&self, chunk: &mut Chunk) {
        for _ in 0..self.attempts {
            let x = random::next_range(0, 16) as usize;
            let z = random::next_range(0, 16) as usize;
            let y = random::next_range(self.min_y as i32, self.max_y as i32) as usize;
            self.generate_vein(chunk, x, y, z);
        }
    }

    fn generate_vein(&self, chunk: &mut Chunk, x: usize, y: usize, z: usize) {
        for i in 0..self.size {
            let dx = (i % 3) as i32 - 1;
            let dy = ((i / 3) % 3) as i32 - 1;
            let dz = ((i / 9) % 3) as i32 - 1;
            let bx = (x as i32 + dx).max(0).min(15) as usize;
            let by = (y as i32 + dy).max(1).min(127) as usize;
            let bz = (z as i32 + dz).max(0).min(15) as usize;
            if chunk.get_block_id(bx, by, bz) == 1 {
                chunk.set_block(bx, by, bz, self.ore_type, self.ore_data);
            }
        }
    }
}
