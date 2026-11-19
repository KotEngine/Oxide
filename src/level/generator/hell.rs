use crate::level::chunk::Chunk;
use super::Generator;

pub struct HellGenerator { pub seed: u64 }

impl HellGenerator {
    pub fn new(seed: u64) -> Self { Self { seed } }
}

impl Generator for HellGenerator {
    fn name(&self) -> &'static str { "hell" }
    fn generate_chunk(&self, chunk: &mut Chunk) {
        for x in 0..16 {
            for z in 0..16 {
                chunk.set_block(x, 0, z, 7, 0);
                chunk.set_block(x, 127, z, 7, 0);
                for y in 1..60 {
                    chunk.set_block(x, y, z, 87, 0);
                }
                for y in 100..126 {
                    chunk.set_block(x, y, z, 87, 0);
                }
                chunk.set_block(x, 60, z, 11, 0);
            }
        }
    }
}
