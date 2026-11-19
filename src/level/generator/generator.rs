use crate::level::chunk::Chunk;

pub trait Generator: Send + Sync {
    fn name(&self) -> &'static str;
    fn generate_chunk(&self, chunk: &mut Chunk);
    fn populate(&self, _chunk: &mut Chunk) {}
    fn settings(&self) -> Vec<(String, String)> { Vec::new() }
}
