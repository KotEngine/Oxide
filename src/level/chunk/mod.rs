pub mod sub_chunk;
pub mod chunk;

pub use chunk::Chunk;
pub use sub_chunk::SubChunk;

pub const MAX_SUBCHUNKS: usize = 16;
pub const CHUNK_SIZE: usize = 16;
pub const SUBCHUNK_BLOCKS: usize = 4096;
