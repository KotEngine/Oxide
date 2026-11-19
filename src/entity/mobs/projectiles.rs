macro_rules! projectile {
    ($name:ident, $type_id:expr, $str_name:expr) => {
        pub struct $name { pub id: i64, pub damage: f32 }
        impl $name {
            pub fn new(id: i64) -> Self { Self { id, damage: 2.0 } }
            pub fn entity_type() -> i32 { $type_id }
            pub fn projectile_name() -> &'static str { $str_name }
        }
    };
}

projectile!(Arrow, 80, "Arrow");
projectile!(Snowball, 81, "Snowball");
projectile!(Egg, 82, "Egg");
projectile!(ThrownEnderPearl, 87, "EnderPearl");
projectile!(ThrownExpBottle, 88, "ExpBottle");
projectile!(ThrownPotion, 86, "Potion");
projectile!(FishingHook, 90, "FishingHook");
