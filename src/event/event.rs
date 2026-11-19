use std::any::Any;

pub trait Event: Any + Send + Sync {
    fn name(&self) -> &'static str { "Event" }
    fn is_cancellable(&self) -> bool { false }
    fn is_cancelled(&self) -> bool { false }
    fn set_cancelled(&mut self, _cancelled: bool) {}
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Debug)]
pub struct PlayerJoinEvent {
    pub player_name: String,
}

impl Event for PlayerJoinEvent {
    fn name(&self) -> &'static str { "PlayerJoinEvent" }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[derive(Debug)]
pub struct PlayerQuitEvent {
    pub player_name: String,
    pub reason: String,
}

impl Event for PlayerQuitEvent {
    fn name(&self) -> &'static str { "PlayerQuitEvent" }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[derive(Debug)]
pub struct PlayerChatEvent {
    pub player_name: String,
    pub message: String,
    pub cancelled: bool,
}

impl Event for PlayerChatEvent {
    fn name(&self) -> &'static str { "PlayerChatEvent" }
    fn is_cancellable(&self) -> bool { true }
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[derive(Debug)]
pub struct PlayerMoveEvent {
    pub player_name: String,
    pub from: crate::math::Vec3,
    pub to: crate::math::Vec3,
    pub cancelled: bool,
}

impl Event for PlayerMoveEvent {
    fn name(&self) -> &'static str { "PlayerMoveEvent" }
    fn is_cancellable(&self) -> bool { true }
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[derive(Debug)]
pub struct BlockBreakEvent {
    pub player_name: String,
    pub x: i32,
    pub y: u32,
    pub z: i32,
    pub block_id: u16,
    pub cancelled: bool,
}

impl Event for BlockBreakEvent {
    fn name(&self) -> &'static str { "BlockBreakEvent" }
    fn is_cancellable(&self) -> bool { true }
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[derive(Debug)]
pub struct BlockPlaceEvent {
    pub player_name: String,
    pub x: i32,
    pub y: u32,
    pub z: i32,
    pub block_id: u16,
    pub cancelled: bool,
}

impl Event for BlockPlaceEvent {
    fn name(&self) -> &'static str { "BlockPlaceEvent" }
    fn is_cancellable(&self) -> bool { true }
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
