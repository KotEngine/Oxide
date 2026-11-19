macro_rules! object_entity {
    ($name:ident, $type_id:expr, $str_name:expr) => {
        pub struct $name { pub id: i64 }
        impl $name {
            pub fn new(id: i64) -> Self { Self { id } }
            pub fn entity_type() -> i32 { $type_id }
            pub fn object_name() -> &'static str { $str_name }
        }
    };
}

object_entity!(Boat, 90, "Boat");
object_entity!(Minecart, 90, "Minecart");
object_entity!(MinecartChest, 90, "MinecartChest");
object_entity!(MinecartHopper, 90, "MinecartHopper");
object_entity!(MinecartTnt, 90, "MinecartTnt");
object_entity!(PrimedTnt, 65, "PrimedTnt");
object_entity!(FallingSand, 66, "FallingSand");
object_entity!(ItemEntity, 64, "Item");
object_entity!(XpOrb, 69, "XpOrb");
object_entity!(Painting, 83, "Painting");
object_entity!(EnderCrystal, 71, "EnderCrystal");
object_entity!(Lightning, 93, "Lightning");
