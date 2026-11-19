use crate::level::chunk::Chunk;
use crate::utils::random;

pub struct TreePopulator {
    pub attempts: usize,
    pub tree_type: u8,
}

impl TreePopulator {
    pub fn oak() -> Self { Self { attempts: 5, tree_type: 0 } }
    pub fn birch() -> Self { Self { attempts: 3, tree_type: 2 } }
    pub fn spruce() -> Self { Self { attempts: 3, tree_type: 1 } }

    pub fn populate(&self, chunk: &mut Chunk) {
        for _ in 0..self.attempts {
            let x = random::next_range(2, 14) as usize;
            let z = random::next_range(2, 14) as usize;
            let mut y = 127;
            while y > 0 && chunk.get_block_id(x, y, z) == 0 { y -= 1; }
            if chunk.get_block_id(x, y, z) == 2 || chunk.get_block_id(x, y, z) == 12 {
                self.generate_tree(chunk, x, y + 1, z);
            }
        }
    }

    fn generate_tree(&self, chunk: &mut Chunk, x: usize, y: usize, z: usize) {
        let height = random::next_range(4, 7) as usize;
        let log_id: u8 = if self.tree_type == 0 { 17 } else { 17 };
        let log_data: u8 = self.tree_type;
        let leaves_id: u8 = 18;
        let leaves_data: u8 = self.tree_type;

        for i in 0..height {
            if y + i < 128 {
                chunk.set_block(x, y + i, z, log_id, log_data);
            }
        }
        let top = y + height;
        for dx in -2i32..=2 {
            for dz in -2i32..=2 {
                for dy in -1i32..=2 {
                    let lx = (x as i32 + dx).max(0).min(15) as usize;
                    let ly = (top as i32 + dy).max(0).min(127) as usize;
                    let lz = (z as i32 + dz).max(0).min(15) as usize;
                    if chunk.get_block_id(lx, ly, lz) == 0 {
                        let dist = dx.abs() + dz.abs() + dy.abs();
                        if dist <= 3 || (dist == 4 && dy <= 0) {
                            chunk.set_block(lx, ly, lz, leaves_id, leaves_data);
                        }
                    }
                }
            }
        }
    }
}
