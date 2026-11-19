use crate::math::BlockPos;
use crate::nbt::NbtCompound;

pub trait Tile: Send + Sync {
    fn tile_id(&self) -> &'static str;
    fn position(&self) -> BlockPos;
    fn save_nbt(&self) -> NbtCompound;
    fn load_nbt(&mut self, compound: NbtCompound);
    fn tick(&mut self) {}
    fn closed(&self) -> bool { false }
    fn close(&mut self) {}
}
