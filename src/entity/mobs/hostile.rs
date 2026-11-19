macro_rules! hostile_mob {
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

hostile_mob!(Zombie, 32, "Zombie", 20.0, 3.0, 1.95);
hostile_mob!(Creeper, 33, "Creeper", 20.0, 0.0, 1.7);
hostile_mob!(Skeleton, 34, "Skeleton", 20.0, 2.0, 1.95);
hostile_mob!(Spider, 35, "Spider", 16.0, 2.0, 0.9);
hostile_mob!(PigZombie, 36, "PigZombie", 20.0, 5.0, 1.95);
hostile_mob!(Slime, 37, "Slime", 16.0, 2.0, 0.51);
hostile_mob!(Enderman, 38, "Enderman", 40.0, 7.0, 2.9);
hostile_mob!(CaveSpider, 39, "CaveSpider", 12.0, 2.0, 0.5);
hostile_mob!(Silverfish, 40, "Silverfish", 8.0, 1.0, 0.3);
hostile_mob!(Blaze, 41, "Blaze", 20.0, 6.0, 1.8);
hostile_mob!(LavaSlime, 42, "MagmaCube", 16.0, 4.0, 0.51);
hostile_mob!(EnderDragon, 45, "EnderDragon", 200.0, 10.0, 4.0);
hostile_mob!(Wither, 46, "Wither", 300.0, 12.0, 3.5);
hostile_mob!(Witch, 45, "Witch", 26.0, 0.0, 1.95);
hostile_mob!(Stray, 46, "Stray", 20.0, 2.0, 1.95);
hostile_mob!(Husk, 47, "Husk", 20.0, 3.0, 1.95);
hostile_mob!(WitherSkeleton, 48, "WitherSkeleton", 20.0, 4.0, 2.4);
hostile_mob!(Guardian, 49, "Guardian", 30.0, 9.0, 0.85);
hostile_mob!(ElderGuardian, 50, "ElderGuardian", 80.0, 12.0, 1.9975);
hostile_mob!(Shulker, 54, "Shulker", 30.0, 4.0, 1.0);
hostile_mob!(Endermite, 55, "Endermite", 8.0, 2.0, 0.3);
hostile_mob!(Vindicator, 57, "Vindicator", 24.0, 13.0, 1.95);
hostile_mob!(Evoker, 58, "Evoker", 24.0, 0.0, 1.95);
hostile_mob!(Vex, 59, "Vex", 14.0, 2.0, 0.8);
