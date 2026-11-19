#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BiomeType {
    Ocean,
    Plains,
    Desert,
    Forest,
    Taiga,
    Swamp,
    River,
    Beach,
    SnowyTundra,
    SnowyMountains,
    Mountains,
    Mesa,
    Savanna,
    Jungle,
    BirchForest,
    DarkOakForest,
    MushroomIsland,
    Hell,
    End,
    Void,
}

#[derive(Debug, Clone)]
pub struct Biome {
    pub id: u8,
    pub name: &'static str,
    pub biome_type: BiomeType,
    pub temperature: f32,
    pub humidity: f32,
    pub base_height: f64,
    pub height_variance: f64,
    pub rain: bool,
    pub snow: bool,
}

impl Biome {
    pub const OCEAN: Biome = Biome { id: 0, name: "Ocean", biome_type: BiomeType::Ocean, temperature: 0.5, humidity: 0.5, base_height: -1.0, height_variance: 0.1, rain: true, snow: false };
    pub const PLAINS: Biome = Biome { id: 1, name: "Plains", biome_type: BiomeType::Plains, temperature: 0.8, humidity: 0.4, base_height: 0.125, height_variance: 0.05, rain: true, snow: false };
    pub const DESERT: Biome = Biome { id: 2, name: "Desert", biome_type: BiomeType::Desert, temperature: 2.0, humidity: 0.0, base_height: 0.125, height_variance: 0.05, rain: false, snow: false };
    pub const FOREST: Biome = Biome { id: 4, name: "Forest", biome_type: BiomeType::Forest, temperature: 0.7, humidity: 0.8, base_height: 0.1, height_variance: 0.2, rain: true, snow: false };
    pub const TAIGA: Biome = Biome { id: 5, name: "Taiga", biome_type: BiomeType::Taiga, temperature: 0.25, humidity: 0.8, base_height: 0.2, height_variance: 0.2, rain: true, snow: false };
    pub const SWAMP: Biome = Biome { id: 6, name: "Swampland", biome_type: BiomeType::Swamp, temperature: 0.8, humidity: 0.9, base_height: -0.2, height_variance: 0.1, rain: true, snow: false };
    pub const RIVER: Biome = Biome { id: 7, name: "River", biome_type: BiomeType::River, temperature: 0.5, humidity: 0.5, base_height: -0.5, height_variance: 0.0, rain: true, snow: false };
    pub const BEACH: Biome = Biome { id: 16, name: "Beach", biome_type: BiomeType::Beach, temperature: 0.8, humidity: 0.4, base_height: 0.0, height_variance: 0.025, rain: true, snow: false };
    pub const SNOWY_TUNDRA: Biome = Biome { id: 11, name: "Ice Plains", biome_type: BiomeType::SnowyTundra, temperature: 0.0, humidity: 0.5, base_height: 0.125, height_variance: 0.05, rain: true, snow: true };
    pub const MOUNTAINS: Biome = Biome { id: 3, name: "Extreme Hills", biome_type: BiomeType::Mountains, temperature: 0.2, humidity: 0.3, base_height: 1.0, height_variance: 0.5, rain: true, snow: false };
    pub const MESA: Biome = Biome { id: 37, name: "Mesa", biome_type: BiomeType::Mesa, temperature: 2.0, humidity: 0.0, base_height: 0.1, height_variance: 0.2, rain: false, snow: false };
    pub const SAVANNA: Biome = Biome { id: 35, name: "Savanna", biome_type: BiomeType::Savanna, temperature: 1.2, humidity: 0.0, base_height: 0.125, height_variance: 0.05, rain: false, snow: false };
    pub const JUNGLE: Biome = Biome { id: 21, name: "Jungle", biome_type: BiomeType::Jungle, temperature: 1.2, humidity: 0.9, base_height: 0.125, height_variance: 0.05, rain: true, snow: false };
    pub const HELL: Biome = Biome { id: 8, name: "Hell", biome_type: BiomeType::Hell, temperature: 2.0, humidity: 0.0, base_height: 0.1, height_variance: 0.2, rain: false, snow: false };
    pub const END: Biome = Biome { id: 9, name: "Sky", biome_type: BiomeType::End, temperature: 0.5, humidity: 0.5, base_height: 0.1, height_variance: 0.2, rain: false, snow: false };
    pub const VOID: Biome = Biome { id: 127, name: "Void", biome_type: BiomeType::Void, temperature: 0.5, humidity: 0.5, base_height: 0.1, height_variance: 0.2, rain: false, snow: false };

    pub fn all() -> &'static [Biome] {
        &[
            Self::OCEAN, Self::PLAINS, Self::DESERT, Self::FOREST, Self::TAIGA, Self::SWAMP,
            Self::RIVER, Self::BEACH, Self::SNOWY_TUNDRA, Self::MOUNTAINS, Self::MESA,
            Self::SAVANNA, Self::JUNGLE, Self::HELL, Self::END, Self::VOID,
        ]
    }
}
