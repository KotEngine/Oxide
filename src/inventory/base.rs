use crate::protocol::types::Item;
use super::{Inventory, InventoryType};

pub struct BaseInventory {
    size: usize,
    title: String,
    inv_type: InventoryType,
    items: parking_lot::RwLock<Vec<Item>>,
}

impl BaseInventory {
    pub fn new(size: usize) -> Self {
        let mut items = Vec::with_capacity(size);
        for _ in 0..size { items.push(Item::air()); }
        Self {
            size,
            title: "Chest".to_string(),
            inv_type: InventoryType::Chest,
            items: parking_lot::RwLock::new(items),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_type(mut self, t: InventoryType) -> Self {
        self.inv_type = t;
        self
    }

    pub fn set_contents(&self, items: Vec<Item>) {
        let mut guard = self.items.write();
        guard.clear();
        for i in 0..self.size.min(items.len()) {
            guard.push(items[i].clone());
        }
        while guard.len() < self.size {
            guard.push(Item::air());
        }
    }

    pub fn get_contents(&self) -> Vec<Item> {
        self.items.read().clone()
    }
}

impl Inventory for BaseInventory {
    fn size(&self) -> usize { self.size }
    fn get_item(&self, index: usize) -> Item {
        self.items.read().get(index).cloned().unwrap_or_default()
    }
    fn set_item(&self, index: usize, item: Item) {
        if let Some(slot) = self.items.write().get_mut(index) {
            *slot = item;
        }
    }
    fn title(&self) -> &str { &self.title }
    fn inventory_type(&self) -> InventoryType { self.inv_type }
}
