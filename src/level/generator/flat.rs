use crate::level::chunk::Chunk;
use super::Generator;

pub struct FlatGenerator {
    pub layers: Vec<(u8, u8)>,
}

impl Default for FlatGenerator {
    fn default() -> Self {
        Self {
            layers: vec![
                (7, 0),
                (3, 0),
                (2, 0),
            ],
        }
    }
}

impl FlatGenerator {
    pub fn new() -> Self { Self::default() }

    pub fn classic() -> Self {
        Self {
            layers: vec![
                (7, 0),
                (3, 0),
                (2, 0),
                (0, 0),
                (0, 0),
                (8, 0),
            ],
        }
    }
}

impl Generator for FlatGenerator {
    fn name(&self) -> &'static str { "flat" }

    fn generate_chunk(&self, chunk: &mut Chunk) {
        let mut y = 0;
        for (block_id, block_data) in &self.layers {
            for _ in 0..1 {
                if y >= 255 { break; }
                for x in 0..16 {
                    for z in 0..16 {
                        chunk.set_block(x, y, z, *block_id, *block_data);
                    }
                }
                y += 1;
            }
        }
    }
}
