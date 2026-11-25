use crate::protocol::types::Item;
use super::base::BaseInventory;
use super::{Inventory, InventoryType};

pub struct PlayerInventory {
    main: BaseInventory,
    armor: BaseInventory,
    offhand: BaseInventory,
    held_index: parking_lot::RwLock<usize>,
    hotbar: parking_lot::RwLock<[i32; 9]>,
}

impl PlayerInventory {
    pub fn new() -> Self {
        Self {
            main: BaseInventory::new(36).with_title("Player").with_type(InventoryType::Player),
            armor: BaseInventory::new(4).with_title("Armor"),
            offhand: BaseInventory::new(1).with_title("Offhand"),
            held_index: parking_lot::RwLock::new(0),
            hotbar: parking_lot::RwLock::new([0, 1, 2, 3, 4, 5, 6, 7, 8]),
        }
    }

    pub fn held_slot_i32(&self) -> i32 {
        let i = (*self.held_index.read()).min(8);
        self.hotbar.read()[i]
    }

    pub fn held_slot(&self) -> usize {
        self.held_slot_i32().max(0) as usize
    }

    pub fn held_item(&self) -> Item {
        let slot = self.held_slot_i32();
        if slot >= 0 && (slot as usize) < 36 {
            self.main.get_item(slot as usize)
        } else {
            Item::air()
        }
    }

    pub fn set_held_index(&self, index: usize) {
        *self.held_index.write() = index.min(8);
    }

    pub fn set_held_index_with_mapping(&self, hotbar_slot: usize, inventory_slot: i32) {
        let hs = hotbar_slot.min(8);
        *self.held_index.write() = hs;
        let mut mapped = inventory_slot - 9;
        if mapped < 0 || mapped >= 36 {
            mapped = -1;
        }
        let mut hb = self.hotbar.write();
        if mapped != -1 {
            if let Some(pos) = hb.iter().position(|&x| x == mapped) {
                hb[pos] = hb[hs];
            }
        }
        hb[hs] = mapped;
    }

    pub fn hotbar_array(&self) -> [i32; 9] {
        *self.hotbar.read()
    }

    pub fn set_hotbar_array(&self, links: [i32; 9]) {
        *self.hotbar.write() = links;
    }

    pub fn wire_hotbar_links(&self) -> Vec<i32> {
        self.hotbar
            .read()
            .iter()
            .map(|&idx| if idx < 0 { -1 } else { idx + 9 })
            .collect()
    }

    pub fn wire_contents(&self) -> Vec<Item> {
        let mut items = self.main.get_contents();
        for _ in 0..9 {
            items.push(Item::air());
        }
        items
    }

    pub fn held_index(&self) -> usize { *self.held_index.read() }

    pub fn get_armor(&self, slot: usize) -> Item { self.armor.get_item(slot) }
    pub fn set_armor(&self, slot: usize, item: Item) { self.armor.set_item(slot, item) }
    pub fn armor_contents(&self) -> Vec<Item> { self.armor.get_contents() }
    pub fn set_armor_contents(&self, items: Vec<Item>) { self.armor.set_contents(items); }

    pub fn offhand(&self) -> Item { self.offhand.get_item(0) }
    pub fn set_offhand(&self, item: Item) { self.offhand.set_item(0, item); }

    pub fn helmet(&self) -> Item { self.armor.get_item(0) }
    pub fn chestplate(&self) -> Item { self.armor.get_item(1) }
    pub fn leggings(&self) -> Item { self.armor.get_item(2) }
    pub fn boots(&self) -> Item { self.armor.get_item(3) }

    pub fn get_contents(&self) -> Vec<Item> { self.main.get_contents() }
    pub fn all_items(&self) -> Vec<Item> { self.main.get_contents() }
    pub fn set_contents(&self, items: Vec<Item>) { self.main.set_contents(items); }
    pub fn get_item(&self, slot: usize) -> Item { self.main.get_item(slot) }
    pub fn set_item(&self, slot: usize, item: Item) { self.main.set_item(slot, item); }
    pub fn add_item(&self, item: Item) -> Vec<Item> { self.main.add_item(item) }
    pub fn size(&self) -> usize { 36 }
    pub fn contains_item(&self, id: i32, count: u32) -> bool {
        let mut total = 0;
        for it in self.get_contents() {
            if it.id == id {
                total += it.count;
            }
        }
        total >= count
    }

    pub fn contains_item_meta(&self, id: i32, meta: i32, count: u32) -> bool {
        let mut total = 0;
        for it in self.get_contents() {
            if it.id == id && (meta < 0 || it.meta == meta) {
                total += it.count;
            }
        }
        total >= count
    }

    pub fn remove_item(&self, id: i32, mut count: u32) -> bool {
        if !self.contains_item(id, count) {
            return false;
        }
        for i in 0..36 {
            let mut it = self.get_item(i);
            if it.id == id && it.count > 0 {
                if it.count <= count {
                    count -= it.count;
                    self.set_item(i, Item::air());
                } else {
                    it.count -= count;
                    count = 0;
                    self.set_item(i, it);
                }
                if count == 0 {
                    break;
                }
            }
        }
        true
    }

    pub fn remove_item_meta(&self, id: i32, meta: i32, mut count: u32) -> bool {
        if !self.contains_item_meta(id, meta, count) {
            return false;
        }
        for i in 0..36 {
            let mut it = self.get_item(i);
            if it.id == id && (meta < 0 || it.meta == meta) && it.count > 0 {
                if it.count <= count {
                    count -= it.count;
                    self.set_item(i, Item::air());
                } else {
                    it.count -= count;
                    count = 0;
                    self.set_item(i, it);
                }
                if count == 0 {
                    break;
                }
            }
        }
        true
    }

    pub fn clear(&self) {
        for i in 0..36 {
            self.main.set_item(i, Item::air());
        }
        for i in 0..4 {
            self.armor.set_item(i, Item::air());
        }
        self.offhand.set_item(0, Item::air());
        *self.hotbar.write() = [0, 1, 2, 3, 4, 5, 6, 7, 8];
    }
}

impl Default for PlayerInventory {
    fn default() -> Self { Self::new() }
}

impl Inventory for PlayerInventory {
    fn size(&self) -> usize { 36 }
    fn get_item(&self, index: usize) -> Item { self.main.get_item(index) }
    fn set_item(&self, index: usize, item: Item) { self.main.set_item(index, item) }
    fn title(&self) -> &str { "Player" }
    fn inventory_type(&self) -> InventoryType { InventoryType::Player }
}

pub struct ChestInventory {
    inv: BaseInventory,
}

impl ChestInventory {
    pub fn new() -> Self { Self { inv: BaseInventory::new(27).with_title("Chest") } }
    pub fn large() -> Self { Self { inv: BaseInventory::new(54).with_title("Large Chest") } }
}

impl Default for ChestInventory {
    fn default() -> Self { Self::new() }
}

impl Inventory for ChestInventory {
    fn size(&self) -> usize { self.inv.size() }
    fn get_item(&self, index: usize) -> Item { self.inv.get_item(index) }
    fn set_item(&self, index: usize, item: Item) { self.inv.set_item(index, item) }
    fn title(&self) -> &str { self.inv.title() }
    fn inventory_type(&self) -> InventoryType { InventoryType::Chest }
}

pub struct FurnaceInventory {
    #[allow(dead_code)]
    inv: BaseInventory,
    pub smelting: parking_lot::RwLock<Item>,
    pub fuel: parking_lot::RwLock<Item>,
    pub result: parking_lot::RwLock<Item>,
    pub cook_time: parking_lot::RwLock<u32>,
    pub burn_time: parking_lot::RwLock<u32>,
}

impl FurnaceInventory {
    pub fn new() -> Self {
        Self {
            inv: BaseInventory::new(3).with_title("Furnace").with_type(InventoryType::Furnace),
            smelting: parking_lot::RwLock::new(Item::air()),
            fuel: parking_lot::RwLock::new(Item::air()),
            result: parking_lot::RwLock::new(Item::air()),
            cook_time: parking_lot::RwLock::new(0),
            burn_time: parking_lot::RwLock::new(0),
        }
    }
}

impl Default for FurnaceInventory {
    fn default() -> Self { Self::new() }
}

impl Inventory for FurnaceInventory {
    fn size(&self) -> usize { 3 }
    fn get_item(&self, index: usize) -> Item {
        match index {
            0 => self.smelting.read().clone(),
            1 => self.fuel.read().clone(),
            2 => self.result.read().clone(),
            _ => Item::air(),
        }
    }
    fn set_item(&self, index: usize, item: Item) {
        match index {
            0 => *self.smelting.write() = item,
            1 => *self.fuel.write() = item,
            2 => *self.result.write() = item,
            _ => {}
        }
    }
    fn title(&self) -> &str { "Furnace" }
    fn inventory_type(&self) -> InventoryType { InventoryType::Furnace }
}

pub struct CraftingInventory {
    inv: BaseInventory,
}

impl CraftingInventory {
    pub fn new() -> Self {
        Self { inv: BaseInventory::new(10).with_title("Crafting").with_type(InventoryType::Crafting) }
    }
}

impl Default for CraftingInventory {
    fn default() -> Self { Self::new() }
}

impl Inventory for CraftingInventory {
    fn size(&self) -> usize { self.inv.size() }
    fn get_item(&self, index: usize) -> Item { self.inv.get_item(index) }
    fn set_item(&self, index: usize, item: Item) { self.inv.set_item(index, item) }
}

pub struct HopperInventory { inv: BaseInventory }
impl HopperInventory {
    pub fn new() -> Self { Self { inv: BaseInventory::new(5).with_title("Hopper").with_type(InventoryType::Hopper) } }
}
impl Default for HopperInventory { fn default() -> Self { Self::new() } }
impl Inventory for HopperInventory {
    fn size(&self) -> usize { self.inv.size() }
    fn get_item(&self, index: usize) -> Item { self.inv.get_item(index) }
    fn set_item(&self, index: usize, item: Item) { self.inv.set_item(index, item) }
}

pub struct DropperInventory { inv: BaseInventory }
impl DropperInventory {
    pub fn new() -> Self { Self { inv: BaseInventory::new(9).with_title("Dropper").with_type(InventoryType::Dropper) } }
}
impl Default for DropperInventory { fn default() -> Self { Self::new() } }
impl Inventory for DropperInventory {
    fn size(&self) -> usize { self.inv.size() }
    fn get_item(&self, index: usize) -> Item { self.inv.get_item(index) }
    fn set_item(&self, index: usize, item: Item) { self.inv.set_item(index, item) }
}

pub struct BrewingInventory { inv: BaseInventory }
impl BrewingInventory {
    pub fn new() -> Self { Self { inv: BaseInventory::new(4).with_title("Brewing Stand").with_type(InventoryType::Brewing) } }
}
impl Default for BrewingInventory { fn default() -> Self { Self::new() } }
impl Inventory for BrewingInventory {
    fn size(&self) -> usize { self.inv.size() }
    fn get_item(&self, index: usize) -> Item { self.inv.get_item(index) }
    fn set_item(&self, index: usize, item: Item) { self.inv.set_item(index, item) }
}

pub struct EnchantInventory { inv: BaseInventory }
impl EnchantInventory {
    pub fn new() -> Self { Self { inv: BaseInventory::new(2).with_title("Enchant").with_type(InventoryType::Enchant) } }
}
impl Default for EnchantInventory { fn default() -> Self { Self::new() } }
impl Inventory for EnchantInventory {
    fn size(&self) -> usize { self.inv.size() }
    fn get_item(&self, index: usize) -> Item { self.inv.get_item(index) }
    fn set_item(&self, index: usize, item: Item) { self.inv.set_item(index, item) }
}

pub struct AnvilInventory { inv: BaseInventory }
impl AnvilInventory {
    pub fn new() -> Self { Self { inv: BaseInventory::new(3).with_title("Anvil").with_type(InventoryType::Anvil) } }
}
impl Default for AnvilInventory { fn default() -> Self { Self::new() } }
impl Inventory for AnvilInventory {
    fn size(&self) -> usize { self.inv.size() }
    fn get_item(&self, index: usize) -> Item { self.inv.get_item(index) }
    fn set_item(&self, index: usize, item: Item) { self.inv.set_item(index, item) }
}

pub struct BeaconInventory { inv: BaseInventory }
impl BeaconInventory {
    pub fn new() -> Self { Self { inv: BaseInventory::new(1).with_title("Beacon").with_type(InventoryType::Beacon) } }
}
impl Default for BeaconInventory { fn default() -> Self { Self::new() } }
impl Inventory for BeaconInventory {
    fn size(&self) -> usize { self.inv.size() }
    fn get_item(&self, index: usize) -> Item { self.inv.get_item(index) }
    fn set_item(&self, index: usize, item: Item) { self.inv.set_item(index, item) }
}

pub struct ShulkerBoxInventory { inv: BaseInventory }
impl ShulkerBoxInventory {
    pub fn new() -> Self { Self { inv: BaseInventory::new(27).with_title("Shulker Box").with_type(InventoryType::ShulkerBox) } }
}
impl Default for ShulkerBoxInventory { fn default() -> Self { Self::new() } }
impl Inventory for ShulkerBoxInventory {
    fn size(&self) -> usize { self.inv.size() }
    fn get_item(&self, index: usize) -> Item { self.inv.get_item(index) }
    fn set_item(&self, index: usize, item: Item) { self.inv.set_item(index, item) }
}

pub struct EnderChestInventory { inv: BaseInventory }
impl EnderChestInventory {
    pub fn new() -> Self { Self { inv: BaseInventory::new(27).with_title("Ender Chest").with_type(InventoryType::EnderChest) } }
}
impl Default for EnderChestInventory { fn default() -> Self { Self::new() } }
impl Inventory for EnderChestInventory {
    fn size(&self) -> usize { self.inv.size() }
    fn get_item(&self, index: usize) -> Item { self.inv.get_item(index) }
    fn set_item(&self, index: usize, item: Item) { self.inv.set_item(index, item) }
}
