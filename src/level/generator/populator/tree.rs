use crate::level::chunk::Chunk;
use crate::utils::random;

pub struct TreePopulator {
    pub attempts: usize,
    pub tree_type: u8,
}

impl TreePopulator {
    pub fn oak() -> Self { Self { attempts: 6, tree_type: 0 } }
    pub fn birch() -> Self { Self { attempts: 4, tree_type: 2 } }
    pub fn spruce() -> Self { Self { attempts: 5, tree_type: 1 } }
    pub fn jungle() -> Self { Self { attempts: 8, tree_type: 3 } }
    pub fn acacia() -> Self { Self { attempts: 3, tree_type: 4 } }
    pub fn dark_oak() -> Self { Self { attempts: 4, tree_type: 5 } }

    pub fn populate(&self, chunk: &mut Chunk) {
        for _ in 0..self.attempts {
            let x = random::next_range(3, 13) as usize;
            let z = random::next_range(3, 13) as usize;
            let mut y = 200;
            while y > 0 && chunk.get_block_id(x, y, z) == 0 {
                y -= 1;
            }
            let ground_id = chunk.get_block_id(x, y, z);
            if ground_id == 2 || ground_id == 3 {
                // Replace grass underneath tree with dirt
                chunk.set_block(x, y, z, 3, 0);
                self.generate_tree(chunk, x, y + 1, z);
            }
        }
    }

    fn generate_tree(&self, chunk: &mut Chunk, x: usize, y: usize, z: usize) {
        let (log_id, log_meta, leaves_id, leaves_meta) = match self.tree_type {
            1 => (17, 1, 18, 1), // Spruce
            2 => (17, 2, 18, 2), // Birch
            3 => (17, 3, 18, 3), // Jungle
            4 => (162, 0, 161, 0), // Acacia
            5 => (162, 1, 161, 1), // Dark Oak
            _ => (17, 0, 18, 0), // Oak
        };

        if self.tree_type == 1 {
            // Conical Spruce tree
            let height = random::next_range(7, 11) as usize;
            for i in 0..height {
                if y + i < 256 {
                    chunk.set_block(x, y + i, z, log_id, log_meta);
                }
            }
            let top = y + height;
            if top < 256 {
                chunk.set_block(x, top, z, leaves_id, leaves_meta);
            }
            let mut radius = 1i32;
            for ly in (y + 3..top).rev() {
                for dx in -radius..=radius {
                    for dz in -radius..=radius {
                        if dx.abs() == radius && dz.abs() == radius {
                            continue;
                        }
                        let lx = (x as i32 + dx).max(0).min(15) as usize;
                        let lz = (z as i32 + dz).max(0).min(15) as usize;
                        if chunk.get_block_id(lx, ly, lz) == 0 {
                            chunk.set_block(lx, ly, lz, leaves_id, leaves_meta);
                        }
                    }
                }
                radius = if radius == 1 { 2 } else { 1 };
            }
        } else if self.tree_type == 3 {
            // Tall Jungle tree
            let height = random::next_range(9, 14) as usize;
            for i in 0..height {
                if y + i < 256 {
                    chunk.set_block(x, y + i, z, log_id, log_meta);
                }
            }
            let top = y + height;
            for dx in -2i32..=2 {
                for dz in -2i32..=2 {
                    for dy in -1i32..=1 {
                        let lx = (x as i32 + dx).max(0).min(15) as usize;
                        let ly = (top as i32 + dy).max(0).min(255) as usize;
                        let lz = (z as i32 + dz).max(0).min(15) as usize;
                        if chunk.get_block_id(lx, ly, lz) == 0 && (dx.abs() + dz.abs() <= 3) {
                            chunk.set_block(lx, ly, lz, leaves_id, leaves_meta);
                        }
                    }
                }
            }
        } else {
            // Standard spherical/oval canopy for Oak, Birch, Acacia, Dark Oak
            let height = random::next_range(5, 8) as usize;
            for i in 0..height {
                if y + i < 256 {
                    chunk.set_block(x, y + i, z, log_id, log_meta);
                }
            }
            let top = y + height;
            for dx in -2i32..=2 {
                for dz in -2i32..=2 {
                    for dy in -2i32..=1 {
                        let lx = (x as i32 + dx).max(0).min(15) as usize;
                        let ly = (top as i32 + dy).max(0).min(255) as usize;
                        let lz = (z as i32 + dz).max(0).min(15) as usize;
                        if chunk.get_block_id(lx, ly, lz) == 0 {
                            let dist = dx.abs() + dz.abs() + dy.abs().max(1);
                            if dist <= 3 || (dy <= 0 && dx.abs() <= 1 && dz.abs() <= 1) {
                                chunk.set_block(lx, ly, lz, leaves_id, leaves_meta);
                            }
                        }
                    }
                }
            }
        }
    }
}
