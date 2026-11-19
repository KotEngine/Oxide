use crate::math::BlockPos;
use crate::nbt::{NbtCompound, NbtValue};
use crate::tile::Tile;
use crate::tile::nameable::Nameable;

pub struct ChestTile {
    pos: BlockPos,
    pub name: String,
    pub pairs_with: Option<BlockPos>,
    pub viewer_count: u32,
}

impl ChestTile {
    pub fn new(pos: BlockPos) -> Self {
        Self { pos, name: String::new(), pairs_with: None, viewer_count: 0 }
    }
}

impl Tile for ChestTile {
    fn tile_id(&self) -> &'static str { "Chest" }
    fn position(&self) -> BlockPos { self.pos }
    fn save_nbt(&self) -> NbtCompound {
        let mut c = NbtCompound::new();
        c.insert("id".to_string(), NbtValue::String(self.tile_id().to_string()));
        c.insert("x".to_string(), NbtValue::Int(self.pos.x));
        c.insert("y".to_string(), NbtValue::Int(self.pos.y));
        c.insert("z".to_string(), NbtValue::Int(self.pos.z));
        if !self.name.is_empty() {
            c.insert("CustomName".to_string(), NbtValue::String(self.name.clone()));
        }
        if let Some(p) = self.pairs_with {
            c.insert("pairx".to_string(), NbtValue::Int(p.x));
            c.insert("pairz".to_string(), NbtValue::Int(p.z));
        }
        c
    }
    fn load_nbt(&mut self, compound: NbtCompound) {
        if let Some(NbtValue::String(n)) = compound.get("CustomName") {
            self.name = n.clone();
        }
    }
}

impl Nameable for ChestTile {
    fn name(&self) -> &str { &self.name }
    fn set_name(&mut self, name: String) { self.name = name; }
}

macro_rules! simple_tile {
    ($name:ident, $id:expr) => {
        pub struct $name {
            pos: BlockPos,
        }
        impl $name {
            pub fn new(pos: BlockPos) -> Self { Self { pos } }
        }
        impl Tile for $name {
            fn tile_id(&self) -> &'static str { $id }
            fn position(&self) -> BlockPos { self.pos }
            fn save_nbt(&self) -> NbtCompound {
                let mut c = NbtCompound::new();
                c.insert("id".to_string(), NbtValue::String($id.to_string()));
                c.insert("x".to_string(), NbtValue::Int(self.pos.x));
                c.insert("y".to_string(), NbtValue::Int(self.pos.y));
                c.insert("z".to_string(), NbtValue::Int(self.pos.z));
                c
            }
            fn load_nbt(&mut self, _compound: NbtCompound) {}
        }
    };
}

simple_tile!(FurnaceTile, "Furnace");
simple_tile!(BrewingStandTile, "BrewingStand");
simple_tile!(HopperTile, "Hopper");
simple_tile!(DispenserTile, "Dispenser");
simple_tile!(BeaconTile, "Beacon");
simple_tile!(BedTile, "Bed");
simple_tile!(CauldronTile, "Cauldron");
simple_tile!(EnchantTableTile, "EnchantTable");
simple_tile!(EnderChestTile, "EnderChest");
simple_tile!(FlowerPotTile, "FlowerPot");
simple_tile!(ItemFrameTile, "ItemFrame");
simple_tile!(MobSpawnerTile, "MobSpawner");
simple_tile!(SkullTile, "Skull");
simple_tile!(ShulkerBoxTile, "ShulkerBox");
simple_tile!(DaylightDetectorTile, "DaylightDetector");
simple_tile!(PistonArmTile, "PistonArm");

pub struct SignTile {
    pos: BlockPos,
    pub text: [String; 4],
    pub author: String,
}

impl SignTile {
    pub fn new(pos: BlockPos) -> Self {
        Self {
            pos,
            text: [String::new(), String::new(), String::new(), String::new()],
            author: String::new(),
        }
    }

    pub fn set_line(&mut self, line: usize, text: String) {
        if line < 4 { self.text[line] = text; }
    }
}

impl Tile for SignTile {
    fn tile_id(&self) -> &'static str { "Sign" }
    fn position(&self) -> BlockPos { self.pos }
    fn save_nbt(&self) -> NbtCompound {
        let mut c = NbtCompound::new();
        c.insert("id".to_string(), NbtValue::String("Sign".to_string()));
        c.insert("x".to_string(), NbtValue::Int(self.pos.x));
        c.insert("y".to_string(), NbtValue::Int(self.pos.y));
        c.insert("z".to_string(), NbtValue::Int(self.pos.z));
        for (i, line) in self.text.iter().enumerate() {
            c.insert(format!("Text{}", i + 1), NbtValue::String(line.clone()));
        }
        if !self.author.is_empty() {
            c.insert("Creator".to_string(), NbtValue::String(self.author.clone()));
        }
        c
    }
    fn load_nbt(&mut self, compound: NbtCompound) {
        for (i, line) in self.text.iter_mut().enumerate() {
            if let Some(NbtValue::String(t)) = compound.get(&format!("Text{}", i + 1)) {
                *line = t.clone();
            }
        }
    }
}
