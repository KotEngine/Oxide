use std::sync::Arc;
use parking_lot::RwLock;
use crate::protocol::types::Item;
use super::base::BaseInventory;

pub trait Inventory: Send + Sync {
    fn size(&self) -> usize;
    fn get_item(&self, index: usize) -> Item;
    fn set_item(&self, index: usize, item: Item);
    fn title(&self) -> &str { "Inventory" }
    fn inventory_type(&self) -> InventoryType { InventoryType::Chest }
    fn add_item(&self, item: Item) -> Vec<Item> {
        let mut remaining = item.clone();
        let mut overflow = Vec::new();
        let size = self.size();
        for i in 0..size {
            if remaining.is_air() || remaining.count == 0 { break; }
            let existing = self.get_item(i);
            if !existing.is_air() && existing.id == remaining.id && existing.meta == remaining.meta {
                let max = 64u32;
                let space = max.saturating_sub(existing.count);
                if space > 0 {
                    let take = remaining.count.min(space);
                    let mut new_item = existing.clone();
                    new_item.count += take;
                    self.set_item(i, new_item);
                    remaining.count -= take;
                }
            }
        }
        for i in 0..size {
            if remaining.is_air() || remaining.count == 0 { break; }
            let existing = self.get_item(i);
            if existing.is_air() {
                let max = 64u32;
                let take = remaining.count.min(max);
                let mut new_item = remaining.clone();
                new_item.count = take;
                self.set_item(i, new_item);
                remaining.count -= take;
            }
        }
        if remaining.count > 0 {
            overflow.push(remaining);
        }
        overflow
    }
    fn remove_item(&self, index: usize) -> Item {
        let item = self.get_item(index);
        self.set_item(index, Item::air());
        item
    }
    fn clear(&self) {
        for i in 0..self.size() {
            self.set_item(i, Item::air());
        }
    }
    fn contains(&self, item: &Item) -> bool {
        for i in 0..self.size() {
            let existing = self.get_item(i);
            if !existing.is_air() && existing.id == item.id && existing.count >= item.count {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InventoryType {
    Player,
    Chest,
    DoubleChest,
    Furnace,
    Crafting,
    Anvil,
    Enchant,
    Hopper,
    Dropper,
    Dispenser,
    Brewing,
    Beacon,
    ShulkerBox,
    EnderChest,
}

pub struct InventoryHolder {
    pub inv: Arc<dyn Inventory>,
    pub items: RwLock<Vec<Item>>,
}

impl InventoryHolder {
    pub fn new(size: usize) -> Self {
        let mut items = Vec::with_capacity(size);
        for _ in 0..size { items.push(Item::air()); }
        Self {
            inv: Arc::new(BaseInventory::new(size)),
            items: RwLock::new(items),
        }
    }
}
