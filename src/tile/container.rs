use crate::inventory::Inventory;
use std::sync::Arc;

pub trait ContainerTile: Send + Sync {
    fn inventory(&self) -> &Arc<dyn Inventory>;
    fn viewer_count(&self) -> u32 { 0 }
}
