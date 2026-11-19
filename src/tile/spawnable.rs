use crate::nbt::NbtCompound;

pub trait Spawnable: Send + Sync {
    fn spawnable_name(&self) -> &'static str;
    fn update_spawnable(&mut self);
    fn save_spawnable(&self) -> NbtCompound;
}
