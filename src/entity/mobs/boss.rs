pub struct EnderDragonBoss { pub id: i64, pub health: f32 }
impl EnderDragonBoss {
    pub fn new(id: i64) -> Self { Self { id, health: 200.0 } }
    pub fn max_health() -> f32 { 200.0 }
    pub fn entity_type() -> i32 { 53 }
}

pub struct WitherBoss { pub id: i64, pub health: f32 }
impl WitherBoss {
    pub fn new(id: i64) -> Self { Self { id, health: 300.0 } }
    pub fn max_health() -> f32 { 300.0 }
    pub fn entity_type() -> i32 { 52 }
}
