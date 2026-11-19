use crate::level::chunk::Chunk;
use super::Generator;

pub struct EnderGenerator { pub seed: u64 }

impl EnderGenerator {
    pub fn new(seed: u64) -> Self { Self { seed } }
}

impl Generator for EnderGenerator {
    fn name(&self) -> &'static str { "ender" }
    fn generate_chunk(&self, chunk: &mut Chunk) {
        if chunk.x == 0 && chunk.z == 0 {
            for x in 0..16 {
                for z in 0..16 {
                    chunk.set_block(x, 0, z, 121, 0);
                }
            }
        }
    }
}
