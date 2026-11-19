macro_rules! mob_def {
    ($name:ident, $type_id:expr, $str_name:expr, $health:expr, $attack:expr, $height:expr) => {
        pub struct $name {
            pub id: i64,
            pub health: f32,
        }
        impl $name {
            pub fn new(id: i64) -> Self { Self { id, health: $health } }
            pub fn max_health() -> f32 { $health }
            pub fn entity_type() -> i32 { $type_id }
            pub fn mob_name() -> &'static str { $str_name }
            pub fn attack_damage() -> f32 { $attack }
            pub fn height() -> f32 { $height }
        }
    };
}

mob_def!(Chicken, 10, "Chicken", 4.0, 0.0, 0.7);
mob_def!(Cow, 11, "Cow", 10.0, 0.0, 1.3);
mob_def!(Pig, 12, "Pig", 10.0, 0.0, 0.9);
mob_def!(Sheep, 13, "Sheep", 8.0, 0.0, 1.3);
mob_def!(Wolf, 14, "Wolf", 8.0, 4.0, 0.85);
mob_def!(Villager, 15, "Villager", 20.0, 0.0, 1.95);
mob_def!(Mooshroom, 16, "Mooshroom", 10.0, 0.0, 1.3);
mob_def!(Squid, 17, "Squid", 10.0, 0.0, 0.8);
mob_def!(Rabbit, 18, "Rabbit", 3.0, 0.0, 0.5);
mob_def!(Bat, 19, "Bat", 6.0, 0.0, 0.9);
mob_def!(IronGolem, 20, "IronGolem", 100.0, 21.0, 2.7);
mob_def!(SnowGolem, 21, "SnowGolem", 4.0, 0.0, 1.9);
mob_def!(Ocelot, 22, "Ocelot", 10.0, 0.0, 0.7);
mob_def!(Horse, 23, "Horse", 30.0, 6.0, 1.6);
mob_def!(Donkey, 24, "Donkey", 30.0, 6.0, 1.6);
mob_def!(Mule, 25, "Mule", 30.0, 6.0, 1.6);
mob_def!(SkeletonHorse, 26, "SkeletonHorse", 30.0, 6.0, 1.6);
mob_def!(ZombieHorse, 27, "ZombieHorse", 30.0, 6.0, 1.6);
mob_def!(PolarBear, 28, "PolarBear", 30.0, 6.0, 1.4);
mob_def!(Llama, 29, "Llama", 30.0, 1.0, 1.87);
