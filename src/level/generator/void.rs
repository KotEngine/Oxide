use crate::level::chunk::Chunk;
use super::Generator;

pub struct VoidGenerator;

impl VoidGenerator {
    pub fn new() -> Self { Self }
}

impl Default for VoidGenerator {
    fn default() -> Self { Self }
}

impl Generator for VoidGenerator {
    fn name(&self) -> &'static str { "void" }

    fn generate_chunk(&self, chunk: &mut Chunk) {
        for x in 0..16 {
            for z in 0..16 {
                chunk.set_block(x, 64, z, 7, 0);
            }
        }
    }
}
