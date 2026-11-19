use std::collections::HashMap;
use parking_lot::RwLock;

pub struct Fuel {
    durations: RwLock<HashMap<i32, u32>>,
}

impl Fuel {
    pub fn new() -> Self {
        let mut d = HashMap::new();
        d.insert(263, 80);   // coal
        d.insert(173, 8000); // block of coal
        d.insert(5, 100);    // planks
        d.insert(17, 100);   // wood
        d.insert(280, 5);    // stick
        d.insert(58, 150);   // crafting table
        d.insert(61, 0);     // furnace (self)
        d.insert(162, 300);  // wood2 (acacia/dark oak)
        d.insert(18, 100);   // fence
        d.insert(151, 100);  // fence_gate
        d.insert(54, 150);   // chest
        d.insert(146, 150);  // trapped chest
        d.insert(85, 100);   // oak fence
        d.insert(107, 100);  // fence gate
        d.insert(323, 0);    // sign item
        d.insert(336, 100);  // ladder
        d.insert(321, 100);  // painting item
        d.insert(263, 80);
        d.insert(50, 100);   // torch
        d.insert(259, 10);   // flint and steel
        d.insert(341, 0);
        d.insert(355, 100);  // bed item
        d.insert(35, 100);   // wool
        d.insert(173, 8000);
        d.insert(311, 0);
        Self { durations: RwLock::new(d) }
    }

    pub fn duration(&self, item_id: i32) -> u32 {
        self.durations.read().get(&item_id).copied().unwrap_or(0)
    }

    pub fn is_fuel(&self, item_id: i32) -> bool {
        self.duration(item_id) > 0
    }

    pub fn register(&self, item_id: i32, duration: u32) {
        self.durations.write().insert(item_id, duration);
    }
}

impl Default for Fuel {
    fn default() -> Self { Self::new() }
}
