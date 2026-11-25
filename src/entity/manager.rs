use std::net::SocketAddr;
use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::RwLock;

use crate::level::World;
use crate::protocol::handler::PlayerState;
use crate::raknet::RakNetServer;
use crate::protocol::packets::{
    AddEntityPacket, MoveEntityPacket, LevelEventPacket, LevelSoundEventPacket,
    ExplodePacket, UpdateBlockPacket, AddItemEntityPacket, TakeItemEntityPacket,
    EntityEventPacket, RemoveEntityPacket, SetEntityDataPacket, ContainerSetContentPacket,
    UpdateAttributesPacket, UpdateTradePacket, Attribute,
    level_event, update_block,
};
use crate::protocol::packet::DataPacket;
use crate::protocol::batch::BatchPacket;
use crate::protocol::types::{ContainerIds, EntityMetadata, Item, WindowTypes};
use crate::entity::loot::LootTable;
use crate::entity::physics;

pub const ORB_SPLIT_SIZES: &[u32] = &[2477, 1237, 617, 307, 149, 73, 37, 17, 7, 3, 1];

pub fn split_xp_orb_sizes(mut amount: u32) -> Vec<u32> {
    let mut out = Vec::new();
    while amount > 0 {
        let size = ORB_SPLIT_SIZES.iter().copied().find(|&s| amount >= s).unwrap_or(1);
        out.push(size);
        amount -= size;
    }
    out
}

#[derive(Debug, Clone)]
pub struct ActiveEntity {
    pub id: i64,
    pub entity_type: i32,
    pub name: String,
    pub dimension: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub motion_x: f32,
    pub motion_y: f32,
    pub motion_z: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub health: f32,
    pub max_health: f32,
    pub fuse: Option<u32>,
    pub lifetime: u32,
    pub owner_eid: Option<i64>,
    pub item_drop: Option<Item>,
    pub pickup_delay: u32,
    pub attack_cooldown: u32,
    pub dead_ticks: u32,
    pub is_dead: bool,
    pub on_fire_ticks: u32,
    pub killer_eid: Option<i64>,
    pub fall_distance: f32,
    pub on_ground: bool,
    pub experience: u32,
    pub age_ticks: i32,
    pub love_ticks: u32,
    pub breed_cooldown: u32,
    pub sheared: bool,
    pub color: u8,
    pub tamed_owner: Option<i64>,
    pub sitting: bool,
    pub size: u8,
}

impl ActiveEntity {
    fn base(id: i64, entity_type: i32, name: String, dimension: i32, x: f64, y: f64, z: f64, health: f32) -> Self {
        Self {
            id,
            entity_type,
            name,
            dimension,
            x,
            y,
            z,
            motion_x: 0.0,
            motion_y: 0.0,
            motion_z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            health,
            max_health: health,
            fuse: if entity_type == 33 { Some(30) } else { None },
            lifetime: 0,
            owner_eid: None,
            item_drop: None,
            pickup_delay: 0,
            attack_cooldown: 0,
            dead_ticks: 0,
            is_dead: false,
            on_fire_ticks: 0,
            killer_eid: None,
            fall_distance: 0.0,
            on_ground: true,
            experience: 0,
            age_ticks: 0,
            love_ticks: 0,
            breed_cooldown: 0,
            sheared: false,
            color: 0,
            tamed_owner: None,
            sitting: false,
            size: if matches!(entity_type, 37 | 42) { 4 } else { 1 },
        }
    }
}

pub fn build_entity_metadata(ent: &ActiveEntity) -> EntityMetadata {
    let mut meta = EntityMetadata::new();
    if ent.on_fire_ticks > 0 {
        meta.set_generic_flag(0, true); // DATA_FLAG_ONFIRE
    }
    if ent.age_ticks < 0 {
        meta.set_generic_flag(11, true); // DATA_FLAG_BABY
        meta.set_float(39, 0.55); // DATA_SCALE
    } else if matches!(ent.entity_type, 37 | 42) {
        let scale = (ent.size as f32) * 0.5;
        meta.set_float(39, scale);
        meta.set_int(2, ent.size as i32); // DATA_VARIANT
    }
    if ent.love_ticks > 0 {
        meta.set_generic_flag(7, true); // DATA_FLAG_INLOVE
    }
    if ent.entity_type == 13 {
        meta.set_byte(3, ent.color as i8); // DATA_COLOUR
        if ent.sheared {
            meta.set_generic_flag(29, true); // DATA_FLAG_SHEARED
        }
    }
    if ent.entity_type == 14 {
        if let Some(owner) = ent.tamed_owner {
            meta.set_generic_flag(27, true); // DATA_FLAG_TAMED
            meta.set_long(5, owner); // DATA_OWNER_EID
        }
        if ent.sitting {
            meta.set_generic_flag(23, true); // DATA_FLAG_SITTING
        }
    }
    if ent.entity_type == 69 {
        meta.set_generic_flag(16, true); // DATA_FLAG_NO_AI
        meta.set_int(15, ent.experience as i32); // DATA_EXPERIENCE_VALUE
    }
    if !ent.name.is_empty() && !matches!(ent.entity_type, 64 | 65 | 69 | 80 | 81 | 82 | 86 | 87) {
        meta.set_string(4, ent.name.clone()); // DATA_NAMETAG
    }
    meta
}

/// Calculate facing yaw angle matching Bedrock format.
pub fn calculate_yaw(mx: f64, mz: f64) -> f32 {
    let yaw = if mz.abs() < 1e-6 {
        if mx < 0.0 { -90.0 } else { 90.0 }
    } else if mx >= 0.0 && mz > 0.0 {
        (mx / mz).atan().to_degrees()
    } else if mx >= 0.0 && mz < 0.0 {
        180.0 - (mx / mz.abs()).atan().to_degrees()
    } else if mx < 0.0 && mz < 0.0 {
        -(180.0 - (mx / mz).atan().to_degrees())
    } else if mx < 0.0 && mz > 0.0 {
        -(mx.abs() / mz).atan().to_degrees()
    } else {
        0.0
    };
    (-yaw) as f32
}

/// Calculate look pitch angle from origin to target.
pub fn calculate_pitch(from_x: f64, from_y: f64, from_z: f64, to_x: f64, to_y: f64, to_z: f64) -> f32 {
    let dist = ((to_x - from_x).powi(2) + (to_y - from_y).powi(2) + (to_z - from_z).powi(2)).sqrt();
    if dist < 1e-6 {
        return 0.0;
    }
    let height = to_y - from_y;
    if height > 0.0 {
        -(height / dist).asin().to_degrees() as f32
    } else if height < 0.0 {
        (-height / dist).asin().to_degrees() as f32
    } else {
        0.0
    }
}

#[derive(Debug, Clone)]
pub struct RightClickOutcome {
    pub consume_held: bool,
    pub replace_held: Option<Item>,
    pub damage_held: bool,
}

pub struct EntityManager {
    entities: RwLock<HashMap<i64, ActiveEntity>>,
}

impl Default for EntityManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            entities: RwLock::new(HashMap::new()),
        }
    }

    pub fn count(&self) -> usize {
        self.entities.read().len()
    }

    pub async fn spawn_mob(
        &self,
        entity_id: i64,
        entity_type: i32,
        name: String,
        x: f64,
        y: f64,
        z: f64,
        health: f32,
        rak: &RakNetServer,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    ) {
        self.spawn_mob_in_dim(entity_id, 0, entity_type, name, x, y, z, health, rak, players).await;
    }

    pub async fn spawn_mob_in_dim(
        &self,
        entity_id: i64,
        dimension: i32,
        entity_type: i32,
        name: String,
        x: f64,
        y: f64,
        z: f64,
        health: f32,
        rak: &RakNetServer,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    ) {
        self.spawn_mob_custom_in_dim(entity_id, dimension, entity_type, name, x, y, z, health, 0, 1, None, rak, players).await;
    }

    pub async fn spawn_mob_custom_in_dim(
        &self,
        entity_id: i64,
        dimension: i32,
        entity_type: i32,
        name: String,
        x: f64,
        y: f64,
        z: f64,
        health: f32,
        age_ticks: i32,
        size: u8,
        color_override: Option<u8>,
        rak: &RakNetServer,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    ) {
        let mut ent = ActiveEntity::base(entity_id, entity_type, name, dimension, x, y, z, health);
        ent.age_ticks = age_ticks;
        if matches!(entity_type, 37 | 42) {
            ent.size = size.max(1);
            ent.health = (ent.size * ent.size) as f32;
            ent.max_health = ent.health;
        }
        if entity_type == 13 {
            ent.color = color_override.unwrap_or_else(|| {
                let roll = rand::random::<u32>() % 1000;
                if roll < 818 { 0 }
                else if roll < 868 { 8 }
                else if roll < 918 { 7 }
                else if roll < 968 { 15 }
                else if roll < 998 { 12 }
                else { 6 }
            });
        }
        let metadata = build_entity_metadata(&ent);
        self.entities.write().insert(entity_id, ent);

        let mut add_pk = AddEntityPacket::new(entity_id, entity_type as u32, x as f32, y as f32, z as f32);
        add_pk.metadata = metadata;
        broadcast_to_dim(rak, players, dimension, add_pk.encode()).await;
    }

    pub async fn spawn_xp_orb_in_dim(
        &self,
        entity_id: i64,
        dimension: i32,
        x: f64,
        y: f64,
        z: f64,
        experience: u32,
        rak: &RakNetServer,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    ) {
        let mut ent = ActiveEntity::base(entity_id, 69, "XPOrb".to_string(), dimension, x, y, z, 5.0);
        ent.experience = experience;
        ent.motion_x = (rand::random::<f32>() - 0.5) * 0.2;
        ent.motion_y = rand::random::<f32>() * 0.25;
        ent.motion_z = (rand::random::<f32>() - 0.5) * 0.2;
        let metadata = build_entity_metadata(&ent);
        let vx = ent.motion_x;
        let vy = ent.motion_y;
        let vz = ent.motion_z;
        self.entities.write().insert(entity_id, ent);

        let mut add_pk = AddEntityPacket::new(entity_id, 69, x as f32, y as f32, z as f32);
        add_pk.speed_x = vx;
        add_pk.speed_y = vy;
        add_pk.speed_z = vz;
        add_pk.metadata = metadata;
        broadcast_to_dim(rak, players, dimension, add_pk.encode()).await;
    }

    pub async fn spawn_xp_orbs_split_in_dim(
        &self,
        dimension: i32,
        x: f64,
        y: f64,
        z: f64,
        total_xp: u32,
        next_eid: &Arc<RwLock<i64>>,
        rak: &RakNetServer,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    ) {
        for chunk_xp in split_xp_orb_sizes(total_xp) {
            let eid = {
                let mut g = next_eid.write();
                let val = *g;
                *g += 1;
                val
            };
            self.spawn_xp_orb_in_dim(eid, dimension, x, y, z, chunk_xp, rak, players).await;
        }
    }

    pub async fn right_click_entity(
        &self,
        target_eid: i64,
        player_addr: SocketAddr,
        player_eid: i64,
        player_dim: i32,
        held: &Item,
        rak: &RakNetServer,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
        next_eid: &Arc<RwLock<i64>>,
    ) -> Option<RightClickOutcome> {
        let mut drop_to_spawn: Option<(Item, f64, f64, f64)> = None;
        let mut meta_sync: Option<(i64, EntityMetadata)> = None;
        let mut ev_sync: Option<Vec<u8>> = None;
        let mut sound_sync: Option<(u8, f32, f32, f32)> = None;
        let mut villager_trade: Option<(i64, String)> = None;
        let mut outcome: Option<RightClickOutcome> = None;

        {
            let mut guard = self.entities.write();
            if let Some(ent) = guard.get_mut(&target_eid) {
                if ent.dimension != player_dim || ent.is_dead {
                    return None;
                }
                match ent.entity_type {
                    13 => {
                        // Sheep
                        if held.id == 359 && !ent.sheared && ent.age_ticks >= 0 {
                            ent.sheared = true;
                            let count = 1 + (rand::random::<u32>() % 3);
                            drop_to_spawn = Some((Item::new(35, ent.color as i32, count), ent.x, ent.y + 0.5, ent.z));
                            meta_sync = Some((ent.id, build_entity_metadata(ent)));
                            outcome = Some(RightClickOutcome { consume_held: false, replace_held: None, damage_held: true });
                        } else if held.id == 351 {
                            ent.color = (15 - (held.meta & 0x0f)) as u8;
                            meta_sync = Some((ent.id, build_entity_metadata(ent)));
                            outcome = Some(RightClickOutcome { consume_held: true, replace_held: None, damage_held: false });
                        } else if held.id == 296 && ent.age_ticks >= 0 && ent.breed_cooldown == 0 && ent.love_ticks == 0 {
                            ent.love_ticks = 600;
                            meta_sync = Some((ent.id, build_entity_metadata(ent)));
                            outcome = Some(RightClickOutcome { consume_held: true, replace_held: None, damage_held: false });
                        }
                    }
                    11 | 16 => {
                        // Cow & Mooshroom
                        if held.id == 325 && held.meta == 0 {
                            outcome = Some(RightClickOutcome { consume_held: false, replace_held: Some(Item::new(325, 1, 1)), damage_held: false });
                            sound_sync = Some((level_event::SOUND_BUCKET_FILL_WATER, ent.x as f32, ent.y as f32, ent.z as f32));
                        } else if ent.entity_type == 16 && held.id == 281 {
                            outcome = Some(RightClickOutcome { consume_held: false, replace_held: Some(Item::new(282, 0, 1)), damage_held: false });
                        } else if held.id == 296 && ent.age_ticks >= 0 && ent.breed_cooldown == 0 && ent.love_ticks == 0 {
                            ent.love_ticks = 600;
                            meta_sync = Some((ent.id, build_entity_metadata(ent)));
                            outcome = Some(RightClickOutcome { consume_held: true, replace_held: None, damage_held: false });
                        }
                    }
                    12 => {
                        // Pig
                        if matches!(held.id, 391 | 392 | 457) && ent.age_ticks >= 0 && ent.breed_cooldown == 0 && ent.love_ticks == 0 {
                            ent.love_ticks = 600;
                            meta_sync = Some((ent.id, build_entity_metadata(ent)));
                            outcome = Some(RightClickOutcome { consume_held: true, replace_held: None, damage_held: false });
                        }
                    }
                    10 => {
                        // Chicken
                        if matches!(held.id, 295 | 361 | 362 | 458) && ent.age_ticks >= 0 && ent.breed_cooldown == 0 && ent.love_ticks == 0 {
                            ent.love_ticks = 600;
                            meta_sync = Some((ent.id, build_entity_metadata(ent)));
                            outcome = Some(RightClickOutcome { consume_held: true, replace_held: None, damage_held: false });
                        }
                    }
                    14 => {
                        // Wolf
                        if ent.tamed_owner.is_none() && held.id == 352 {
                            let tamed = (rand::random::<u32>() % 3) == 0;
                            if tamed {
                                ent.tamed_owner = Some(player_eid);
                                ent.health = 20.0;
                                ent.max_health = 20.0;
                                ev_sync = Some(EntityEventPacket::new(ent.id, EntityEventPacket::TAME_SUCCESS).encode());
                            } else {
                                ev_sync = Some(EntityEventPacket::new(ent.id, EntityEventPacket::TAME_FAIL).encode());
                            }
                            meta_sync = Some((ent.id, build_entity_metadata(ent)));
                            outcome = Some(RightClickOutcome { consume_held: true, replace_held: None, damage_held: false });
                        } else if ent.tamed_owner == Some(player_eid) {
                            if matches!(held.id, 319 | 320 | 363 | 364 | 365 | 366 | 423 | 424) && ent.health < ent.max_health {
                                ent.health = (ent.health + 4.0).min(ent.max_health);
                                if ent.age_ticks >= 0 && ent.breed_cooldown == 0 && ent.love_ticks == 0 {
                                    ent.love_ticks = 600;
                                }
                                meta_sync = Some((ent.id, build_entity_metadata(ent)));
                                outcome = Some(RightClickOutcome { consume_held: true, replace_held: None, damage_held: false });
                            } else {
                                ent.sitting = !ent.sitting;
                                meta_sync = Some((ent.id, build_entity_metadata(ent)));
                                outcome = Some(RightClickOutcome { consume_held: false, replace_held: None, damage_held: false });
                            }
                        }
                    }
                    15 => {
                        // Villager
                        villager_trade = Some((ent.id, ent.name.clone()));
                        outcome = Some(RightClickOutcome { consume_held: false, replace_held: None, damage_held: false });
                    }
                    _ => {}
                }

                // Name Tag (421) renames any living mob
                if outcome.is_none() && held.id == 421 {
                    if let Some(custom) = crate::item::enchantment::parse_item_custom_name(&held.nbt) {
                        ent.name = custom;
                        meta_sync = Some((ent.id, build_entity_metadata(ent)));
                        outcome = Some(RightClickOutcome { consume_held: true, replace_held: None, damage_held: false });
                    }
                }
            }
        }

        if let Some((drop_item, dx, dy, dz)) = drop_to_spawn {
            let deid = {
                let mut g = next_eid.write();
                let v = *g;
                *g += 1;
                v
            };
            self.spawn_item_drop_in_dim(deid, player_dim, drop_item, dx, dy, dz, rak, players).await;
        }
        if let Some((eid, meta)) = meta_sync {
            let set_meta = SetEntityDataPacket { entity_runtime_id: eid, metadata: meta };
            broadcast_to_dim(rak, players, player_dim, set_meta.encode()).await;
        }
        if let Some(ev_pk) = ev_sync {
            broadcast_to_dim(rak, players, player_dim, ev_pk).await;
        }
        if let Some((sid, sx, sy, sz)) = sound_sync {
            let snd = LevelSoundEventPacket::new(sid, sx, sy, sz);
            broadcast_to_dim(rak, players, player_dim, snd.encode()).await;
        }
        if let Some((trader_eid, name)) = villager_trade {
            let trade_pk = UpdateTradePacket {
                window_id: 1,
                window_type: WindowTypes::TRADING,
                varint1: 0,
                varint2: 0,
                is_willing: true,
                trader_eid,
                player_eid,
                display_name: if name.is_empty() { "Villager".to_string() } else { name },
                offers: Vec::new(),
            };
            let batch = BatchPacket::encode(&[trade_pk.encode()]);
            rak.send_encapsulated(player_addr, batch, true).await;
        }
        outcome
    }

    pub async fn spawn_lightning(
        &self,
        entity_id: i64,
        x: f64,
        y: f64,
        z: f64,
        world: &Arc<RwLock<World>>,
        rak: &RakNetServer,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    ) {
        self.spawn_lightning_in_dim(entity_id, 0, x, y, z, world, rak, players).await;
    }

    pub async fn spawn_lightning_in_dim(
        &self,
        entity_id: i64,
        dimension: i32,
        x: f64,
        y: f64,
        z: f64,
        world: &Arc<RwLock<World>>,
        rak: &RakNetServer,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    ) {
        let add_pk = AddEntityPacket::new(entity_id, 93, x as f32, y as f32, z as f32);
        broadcast_to_dim(rak, players, dimension, add_pk.encode()).await;

        let explode_pk = ExplodePacket::new(x as f32, y as f32, z as f32, 10.0);
        broadcast_to_dim(rak, players, dimension, explode_pk.encode()).await;

        let bx = x.floor() as i32;
        let by = y.floor() as i32;
        let bz = z.floor() as i32;
        let placed_fire = {
            let mut w = world.write();
            if w.get_block_id_opt(bx, by, bz) != 0 && w.get_block_id_opt(bx, by + 1, bz) == 0 {
                w.set_block(bx, by + 1, bz, 51, 0);
                true
            } else {
                false
            }
        };
        if placed_fire {
            let upk = UpdateBlockPacket::new(bx, (by + 1) as u32, bz, 51, 0, update_block::FLAG_ALL);
            broadcast_to_dim(rak, players, dimension, upk.encode()).await;
        }

        let mut to_transform = Vec::new();
        {
            let mut entities = self.entities.write();
            for ent in entities.values_mut() {
                if ent.dimension != dimension {
                    continue;
                }
                let dist_sq = (ent.x - x).powi(2) + (ent.z - z).powi(2);
                if dist_sq <= 16.0 && ent.entity_type == 12 {
                    ent.entity_type = 36;
                    ent.name = "Zombie Pigman".to_string();
                    ent.health = 20.0;
                    ent.max_health = 20.0;
                    to_transform.push((ent.id, 36, ent.x, ent.y, ent.z));
                }
            }
        }

        for (mob_id, new_type, mx, my, mz) in to_transform {
            let rem_pk = RemoveEntityPacket { entity_runtime_id: mob_id };
            broadcast_to_dim(rak, players, dimension, rem_pk.encode()).await;

            let add_pk = AddEntityPacket::new(mob_id, new_type as u32, mx as f32, my as f32, mz as f32);
            broadcast_to_dim(rak, players, dimension, add_pk.encode()).await;
        }

        let mut player_hits = Vec::new();
        {
            let p = players.read();
            for (addr, s) in p.iter() {
                if s.dimension == dimension && s.gamemode == 0 {
                    let dist_sq = (s.x - x).powi(2) + (s.z - z).powi(2);
                    if dist_sq <= 16.0 {
                        player_hits.push((*addr, s.username.clone()));
                    }
                }
            }
        }

        for (addr, uname) in player_hits {
            {
                let mut p = players.write();
                if let Some(s) = p.get_mut(&addr) {
                    s.fire_ticks = s.fire_ticks.max(100);
                }
            }
            let msg = format!("{} was struck by lightning", uname);
            crate::entity::damage::hurt_player(
                players,
                rak,
                addr,
                10.0,
                crate::entity::damage::DamageCause::EntityAttack,
                false,
                Some((x, y, z)),
                None,
                Some(&msg),
            )
            .await;
        }
    }

    pub async fn spawn_tnt(
        &self,
        entity_id: i64,
        x: f64,
        y: f64,
        z: f64,
        fuse_ticks: u32,
        rak: &RakNetServer,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    ) {
        self.spawn_tnt_in_dim(entity_id, 0, x, y, z, fuse_ticks, rak, players).await;
    }

    pub async fn spawn_tnt_in_dim(
        &self,
        entity_id: i64,
        dimension: i32,
        x: f64,
        y: f64,
        z: f64,
        fuse_ticks: u32,
        rak: &RakNetServer,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    ) {
        let mut ent = ActiveEntity::base(entity_id, 65, "PrimedTNT".to_string(), dimension, x, y, z, 1.0);
        ent.motion_y = 0.2;
        ent.fuse = Some(fuse_ticks);
        self.entities.write().insert(entity_id, ent);

        let mut add_pk = AddEntityPacket::new(entity_id, 65, x as f32, y as f32, z as f32);
        add_pk.speed_y = 0.2;
        broadcast_to_dim(rak, players, dimension, add_pk.encode()).await;

        let ignite_sound = LevelEventPacket::new(level_event::EVENT_SOUND_IGNITE, x as f32, y as f32, z as f32, 0);
        broadcast_to_dim(rak, players, dimension, ignite_sound.encode()).await;
    }

    pub async fn spawn_projectile(
        &self,
        entity_id: i64,
        entity_type: i32,
        x: f64,
        y: f64,
        z: f64,
        vx: f32,
        vy: f32,
        vz: f32,
        yaw: f32,
        pitch: f32,
        owner_eid: i64,
        rak: &RakNetServer,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    ) {
        self.spawn_projectile_in_dim(
            entity_id, 0, entity_type, x, y, z, vx, vy, vz, yaw, pitch, owner_eid, rak, players,
        ).await;
    }

    pub async fn spawn_projectile_in_dim(
        &self,
        entity_id: i64,
        dimension: i32,
        entity_type: i32,
        x: f64,
        y: f64,
        z: f64,
        vx: f32,
        vy: f32,
        vz: f32,
        yaw: f32,
        pitch: f32,
        owner_eid: i64,
        rak: &RakNetServer,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    ) {
        let mut ent = ActiveEntity::base(entity_id, entity_type, format!("Projectile_{}", entity_type), dimension, x, y, z, 1.0);
        ent.motion_x = vx;
        ent.motion_y = vy;
        ent.motion_z = vz;
        ent.yaw = yaw;
        ent.pitch = pitch;
        ent.owner_eid = Some(owner_eid);
        self.entities.write().insert(entity_id, ent);

        let mut add_pk = AddEntityPacket::new(entity_id, entity_type as u32, x as f32, y as f32, z as f32);
        add_pk.speed_x = vx;
        add_pk.speed_y = vy;
        add_pk.speed_z = vz;
        add_pk.yaw = yaw;
        add_pk.pitch = pitch;
        broadcast_to_dim(rak, players, dimension, add_pk.encode()).await;
    }

    pub async fn spawn_item_drop(
        &self,
        entity_id: i64,
        item: Item,
        x: f64,
        y: f64,
        z: f64,
        rak: &RakNetServer,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    ) {
        self.spawn_item_drop_in_dim(entity_id, 0, item, x, y, z, rak, players).await;
    }

    pub async fn spawn_item_drop_in_dim(
        &self,
        entity_id: i64,
        dimension: i32,
        item: Item,
        x: f64,
        y: f64,
        z: f64,
        rak: &RakNetServer,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    ) {
        let rand_x = (rand::random::<f32>() - 0.5) * 0.15;
        let rand_z = (rand::random::<f32>() - 0.5) * 0.15;

        let mut ent = ActiveEntity::base(entity_id, 64, "Item".to_string(), dimension, x + rand_x as f64, y, z + rand_z as f64, 5.0);
        ent.motion_x = rand_x;
        ent.motion_y = 0.2;
        ent.motion_z = rand_z;
        ent.item_drop = Some(item.clone());
        ent.pickup_delay = 10;
        self.entities.write().insert(entity_id, ent);

        let mut pk = AddItemEntityPacket::new(entity_id, item, (x + rand_x as f64) as f32, y as f32, (z + rand_z as f64) as f32);
        pk.speed_x = rand_x;
        pk.speed_y = 0.2;
        pk.speed_z = rand_z;
        broadcast_to_dim(rak, players, dimension, pk.encode()).await;
    }

    pub async fn send_entities_in_dim_to(
        &self,
        addr: SocketAddr,
        dimension: i32,
        rak: &RakNetServer,
    ) {
        let packets: Vec<Vec<u8>> = {
            let guard = self.entities.read();
            let mut out = Vec::new();
            for ent in guard.values() {
                if ent.dimension != dimension || ent.is_dead {
                    continue;
                }
                if ent.entity_type == 64 {
                    if let Some(ref item) = ent.item_drop {
                        let mut pk = AddItemEntityPacket::new(
                            ent.id,
                            item.clone(),
                            ent.x as f32,
                            ent.y as f32,
                            ent.z as f32,
                        );
                        pk.speed_x = ent.motion_x;
                        pk.speed_y = ent.motion_y;
                        pk.speed_z = ent.motion_z;
                        out.push(pk.encode());
                    }
                } else {
                    let mut pk = AddEntityPacket::new(
                        ent.id,
                        ent.entity_type as u32,
                        ent.x as f32,
                        ent.y as f32,
                        ent.z as f32,
                    );
                    pk.speed_x = ent.motion_x;
                    pk.speed_y = ent.motion_y;
                    pk.speed_z = ent.motion_z;
                    pk.yaw = ent.yaw;
                    pk.pitch = ent.pitch;
                    pk.metadata = build_entity_metadata(ent);
                    out.push(pk.encode());
                }
            }
            out
        };
        if !packets.is_empty() {
            let batch = BatchPacket::encode(&packets);
            rak.send_encapsulated(addr, batch, true).await;
        }
    }

    pub async fn despawn_entities_in_dim_for(
        &self,
        addr: SocketAddr,
        dimension: i32,
        rak: &RakNetServer,
    ) {
        let packets: Vec<Vec<u8>> = {
            let guard = self.entities.read();
            guard.values()
                .filter(|ent| ent.dimension == dimension)
                .map(|ent| RemoveEntityPacket { entity_runtime_id: ent.id }.encode())
                .collect()
        };
        if !packets.is_empty() {
            let batch = BatchPacket::encode(&packets);
            rak.send_encapsulated(addr, batch, true).await;
        }
    }

    pub fn find_hit_entity(
        &self,
        dimension: i32,
        x: f64,
        y: f64,
        z: f64,
        radius_sq: f64,
        ignore_eid: Option<i64>,
    ) -> Option<i64> {
        let guard = self.entities.read();
        let mut best: Option<(f64, i64)> = None;
        for ent in guard.values() {
            if ent.dimension != dimension || ent.is_dead || Some(ent.id) == ignore_eid {
                continue;
            }
            if matches!(ent.entity_type, 64 | 65 | 68 | 69 | 80 | 81 | 82 | 86 | 87 | 93) {
                continue;
            }
            let (hw, h) = physics::entity_half_size(ent.entity_type);
            let horiz_sq = (ent.x - x).powi(2) + (ent.z - z).powi(2);
            let vertical_ok = y >= ent.y - 0.15 && y <= ent.y + h + 0.15;
            let center_sq = horiz_sq + (ent.y + h * 0.5 - y).powi(2);
            if (vertical_ok && horiz_sq <= hw * hw + 0.25) || center_sq <= radius_sq {
                if best.map_or(true, |(b_sq, _)| center_sq < b_sq) {
                    best = Some((center_sq, ent.id));
                }
            }
        }
        best.map(|(_, id)| id)
    }

    pub fn get_entities_in_radius(&self, dimension: i32, x: f64, y: f64, z: f64, radius: f64) -> Vec<i64> {
        let r_sq = radius * radius;
        let guard = self.entities.read();
        guard.values()
            .filter(|ent| {
                ent.dimension == dimension
                    && !ent.is_dead
                    && !matches!(ent.entity_type, 64 | 65 | 68 | 69 | 80 | 81 | 82 | 86 | 87 | 93)
                    && (ent.x - x).powi(2) + (ent.y + 0.9 - y).powi(2) + (ent.z - z).powi(2) <= r_sq
            })
            .map(|ent| ent.id)
            .collect()
    }

    pub async fn damage_entity(
        &self,
        entity_id: i64,
        damage: f32,
        attacker_eid: i64,
        attacker_pos: Option<(f64, f64, f64)>,
        rak: &RakNetServer,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    ) -> bool {
        self.damage_entity_ext(entity_id, damage, attacker_eid, attacker_pos, 0, 0.0, rak, players).await
    }

    pub async fn damage_entity_ext(
        &self,
        entity_id: i64,
        damage: f32,
        attacker_eid: i64,
        attacker_pos: Option<(f64, f64, f64)>,
        fire_seconds: u32,
        knockback_extra: f32,
        rak: &RakNetServer,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    ) -> bool {
        enum DamageResult {
            NotFound,
            Alive { dim: i32, x: f32, y: f32, z: f32, yaw: f32, pitch: f32 },
            Died { dim: i32, x: f32, y: f32, z: f32, yaw: f32, pitch: f32 },
        }

        let outcome = {
            let mut ent_guard = self.entities.write();
            if let Some(ent) = ent_guard.get_mut(&entity_id) {
                if ent.is_dead || matches!(ent.entity_type, 64 | 65 | 69 | 80 | 81 | 82 | 86 | 87 | 93) {
                    DamageResult::NotFound
                } else {
                    ent.health -= damage;
                    if fire_seconds > 0 {
                        ent.on_fire_ticks = ent.on_fire_ticks.max(fire_seconds * 20);
                    }
                    let dim = ent.dimension;

                    if let Some((ax, _ay, az)) = attacker_pos {
                        let dx = ent.x - ax;
                        let dz = ent.z - az;
                        let f = (dx * dx + dz * dz).sqrt();
                        if f > 0.0 {
                            let inv_f = 1.0 / f;
                            let base = 0.4f32 + knockback_extra * 0.25;
                            ent.motion_x /= 2.0;
                            ent.motion_y /= 2.0;
                            ent.motion_z /= 2.0;
                            ent.motion_x += (dx * inv_f) as f32 * base;
                            ent.motion_y += base.min(0.5);
                            ent.motion_z += (dz * inv_f) as f32 * base;
                            if ent.motion_y > 0.45 {
                                ent.motion_y = 0.45;
                            }
                        }
                    }

                    if ent.health <= 0.0 {
                        ent.is_dead = true;
                        ent.dead_ticks = 0;
                        ent.killer_eid = Some(attacker_eid);
                        DamageResult::Died {
                            dim,
                            x: ent.x as f32,
                            y: ent.y as f32,
                            z: ent.z as f32,
                            yaw: ent.yaw,
                            pitch: ent.pitch,
                        }
                    } else {
                        DamageResult::Alive {
                            dim,
                            x: ent.x as f32,
                            y: ent.y as f32,
                            z: ent.z as f32,
                            yaw: ent.yaw,
                            pitch: ent.pitch,
                        }
                    }
                }
            } else {
                DamageResult::NotFound
            }
        };

        match outcome {
            DamageResult::NotFound => false,
            DamageResult::Died { dim, x, y, z, yaw, pitch } => {
                let death_anim = EntityEventPacket::new(entity_id, 3);
                let death_snd = LevelSoundEventPacket::new(level_event::SOUND_DEATH, x, y, z);
                let mv = MoveEntityPacket::new(entity_id, x, y, z, yaw, pitch);
                broadcast_to_dim(rak, players, dim, death_anim.encode()).await;
                broadcast_to_dim(rak, players, dim, death_snd.encode()).await;
                broadcast_to_dim(rak, players, dim, mv.encode()).await;
                true
            }
            DamageResult::Alive { dim, x, y, z, yaw, pitch } => {
                let anim = EntityEventPacket::new(entity_id, 2);
                let sound = LevelSoundEventPacket::new(level_event::SOUND_HIT, x, y, z);
                let mv = MoveEntityPacket::new(entity_id, x, y, z, yaw, pitch);
                broadcast_to_dim(rak, players, dim, anim.encode()).await;
                broadcast_to_dim(rak, players, dim, sound.encode()).await;
                broadcast_to_dim(rak, players, dim, mv.encode()).await;
                false
            }
        }
    }

    pub async fn tick_all(
        &self,
        worlds: &[&Arc<RwLock<World>>; 3],
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
        rak: &RakNetServer,
        next_eid: &Arc<RwLock<i64>>,
    ) {
        let mut list: Vec<ActiveEntity> = {
            let guard = self.entities.read();
            guard.values().cloned().collect()
        };

        let mut to_remove = Vec::new();
        let mut explosions: Vec<(i32, f64, f64, f64, f32)> = Vec::new();
        let mut drops: Vec<(i32, Item, f64, f64, f64)> = Vec::new();
        let mut xp_spawns: Vec<(i32, u32, f64, f64, f64)> = Vec::new();
        let mut xp_pickups: Vec<(i32, SocketAddr, u32, f64, f64, f64)> = Vec::new();
        let mut child_mobs: Vec<(i32, i32, String, f64, f64, f64, f32, i32, u8, Option<u8>)> = Vec::new();
        let mut player_pickups: Vec<(i32, SocketAddr, i64, Item, f64, f64, f64)> = Vec::new();
        let mut player_attacks: Vec<(i32, i64, String, SocketAddr, f32, f64, f64, f64)> = Vec::new();
        let mut mob_vs_mob: Vec<(i64, i64, f32, f64, f64, f64)> = Vec::new();
        let mut spawned_projectiles: Vec<(i32, i32, f64, f64, f64, f32, f32, f32, f32, f32, i64)> = Vec::new();
        let mut block_changes: Vec<(i32, i32, i32, i32, u8, u8)> = Vec::new();

        let player_snapshots: Vec<(SocketAddr, i64, i32, f64, f64, f64, i32)> = {
            let p = players.read();
            p.iter()
                .filter(|(_, s)| s.spawned)
                .map(|(addr, s)| (*addr, s.entity_id, s.dimension, s.x, s.y, s.z, s.gamemode))
                .collect()
        };

        // Snapshot hostile mob positions for IronGolem / SnowGolem / Tamed Wolf AI
        let hostile_targets: Vec<(i64, i32, f64, f64, f64)> = list
            .iter()
            .filter(|e| {
                !e.is_dead
                    && matches!(e.entity_type, 32 | 33 | 34 | 35 | 37 | 38 | 40 | 41 | 42 | 43 | 45 | 46 | 47 | 48)
            })
            .map(|e| (e.id, e.dimension, e.x, e.y, e.z))
            .collect();

        // Check breeding pairs among nearby animals in love mode
        let mut bred_ids = std::collections::HashSet::new();
        for i in 0..list.len() {
            if list[i].love_ticks == 0 || list[i].is_dead || bred_ids.contains(&list[i].id) {
                continue;
            }
            for j in (i + 1)..list.len() {
                if list[j].love_ticks == 0
                    || list[j].is_dead
                    || list[j].dimension != list[i].dimension
                    || list[j].entity_type != list[i].entity_type
                    || bred_ids.contains(&list[j].id)
                {
                    continue;
                }
                let d_sq = (list[i].x - list[j].x).powi(2)
                    + (list[i].y - list[j].y).powi(2)
                    + (list[i].z - list[j].z).powi(2);
                if d_sq <= 6.25 {
                    bred_ids.insert(list[i].id);
                    bred_ids.insert(list[j].id);
                    list[i].love_ticks = 0;
                    list[j].love_ticks = 0;
                    list[i].breed_cooldown = 6000;
                    list[j].breed_cooldown = 6000;
                    child_mobs.push((
                        list[i].dimension,
                        list[i].entity_type,
                        list[i].name.clone(),
                        (list[i].x + list[j].x) * 0.5,
                        list[i].y,
                        (list[i].z + list[j].z) * 0.5,
                        list[i].max_health,
                        -24000,
                        1,
                        if list[i].entity_type == 13 { Some(list[i].color) } else { None },
                    ));
                    xp_spawns.push((list[i].dimension, 3, list[i].x, list[i].y + 0.3, list[i].z));
                    break;
                }
            }
        }

        for ent in &mut list {
            ent.lifetime += 1;
            let dim = ent.dimension.clamp(0, 2);
            let world = worlds[dim as usize];

            // Handle dead entities (despawn animation timer)
            if ent.is_dead {
                ent.dead_ticks += 1;
                if ent.dead_ticks >= 20 {
                    to_remove.push(ent.id);
                    let remove_pk = RemoveEntityPacket { entity_runtime_id: ent.id };
                    broadcast_to_dim(rak, players, dim, remove_pk.encode()).await;

                    if matches!(ent.entity_type, 37 | 42) && ent.size > 1 {
                        let new_size = ent.size / 2;
                        for offset in [-0.4f64, 0.4f64] {
                            child_mobs.push((
                                dim,
                                ent.entity_type,
                                ent.name.clone(),
                                ent.x + offset,
                                ent.y,
                                ent.z - offset,
                                (new_size * new_size) as f32,
                                0,
                                new_size,
                                None,
                            ));
                        }
                    } else {
                        let generated_drops = LootTable::get_drops(
                            ent.entity_type,
                            ent.killer_eid.is_some(),
                            ent.on_fire_ticks > 0,
                            0,
                        );
                        for item in generated_drops {
                            drops.push((dim, item, ent.x, ent.y, ent.z));
                        }
                    }

                    if ent.killer_eid.is_some() {
                        let xp = LootTable::xp_reward(ent.entity_type, ent.size);
                        if xp > 0 {
                            xp_spawns.push((dim, xp, ent.x, ent.y + 0.2, ent.z));
                        }
                    }
                }
                continue;
            }

            if ent.age_ticks < 0 {
                ent.age_ticks += 1;
            }
            if ent.love_ticks > 0 {
                ent.love_ticks -= 1;
            }
            if ent.breed_cooldown > 0 {
                ent.breed_cooldown -= 1;
            }

            // Fire, lava & daylight undead burn
            let fireproof = matches!(ent.entity_type, 36 | 41 | 42 | 43 | 48 | 52 | 64 | 65 | 69 | 80 | 81 | 82 | 87);
            let (in_water, in_lava, is_daytime_exposed) = {
                let w = world.read();
                let water = physics::is_in_water(&w, ent.x, ent.y + 0.2, ent.z);
                let lava = physics::is_in_lava(&w, ent.x, ent.y + 0.2, ent.z);
                let t = w.time % 24000;
                let day_sun = dim == 0
                    && t < 12000
                    && matches!(ent.entity_type, 32 | 34 | 46)
                    && w.get_full_light_opt(ent.x.floor() as i32, ent.y.floor() as i32 + 2, ent.z.floor() as i32) >= 14;
                (water, lava, day_sun)
            };

            if in_water && ent.on_fire_ticks > 0 {
                ent.on_fire_ticks = 0;
            } else if !fireproof && (in_lava || is_daytime_exposed) {
                ent.on_fire_ticks = ent.on_fire_ticks.max(160);
            }

            if ent.on_fire_ticks > 0 {
                ent.on_fire_ticks -= 1;
                if !fireproof && !matches!(ent.entity_type, 64 | 65 | 69 | 80 | 81 | 82 | 87) && ent.lifetime % 20 == 0 {
                    ent.health -= 1.0;
                    if ent.health <= 0.0 {
                        ent.is_dead = true;
                        ent.dead_ticks = 0;
                        continue;
                    }
                }
            }
            if !fireproof && !matches!(ent.entity_type, 64 | 65 | 69 | 80 | 81 | 82 | 87) && in_lava && ent.lifetime % 10 == 0 {
                ent.health -= 4.0;
                if ent.health <= 0.0 {
                    ent.is_dead = true;
                    ent.dead_ticks = 0;
                    continue;
                }
            }

            // 1. TNT Fuse Logic
            if let Some(ref mut fuse) = ent.fuse {
                if ent.entity_type == 65 {
                    if *fuse > 0 {
                        *fuse -= 1;
                    } else {
                        to_remove.push(ent.id);
                        let rem_pk = RemoveEntityPacket { entity_runtime_id: ent.id };
                        broadcast_to_dim(rak, players, dim, rem_pk.encode()).await;
                        explosions.push((dim, ent.x, ent.y, ent.z, 4.0));
                        continue;
                    }
                }
            }

            // 1b. XPOrb (69) attraction & pickup
            if ent.entity_type == 69 {
                let mut best_target: Option<(SocketAddr, f64, f64, f64, f64)> = None;
                for &(p_addr, _p_eid, p_dim, px, py, pz, gm) in &player_snapshots {
                    if p_dim != dim || gm == 3 {
                        continue;
                    }
                    let dist_sq = (px - ent.x).powi(2) + (py + 0.8 - ent.y).powi(2) + (pz - ent.z).powi(2);
                    if dist_sq <= 64.0 && best_target.map_or(true, |b| dist_sq < b.4) {
                        best_target = Some((p_addr, px, py + 0.8, pz, dist_sq));
                    }
                }
                if let Some((p_addr, tx, ty, tz, dist_sq)) = best_target {
                    if dist_sq <= 1.25 {
                        to_remove.push(ent.id);
                        let rem_pk = RemoveEntityPacket { entity_runtime_id: ent.id };
                        broadcast_to_dim(rak, players, dim, rem_pk.encode()).await;
                        xp_pickups.push((dim, p_addr, ent.experience, ent.x, ent.y, ent.z));
                        continue;
                    }
                    let d = dist_sq.sqrt().max(0.01);
                    let factor = 1.0 - (d / 8.0);
                    let strength = factor * factor * 0.12;
                    ent.motion_x += ((tx - ent.x) / d * strength) as f32;
                    ent.motion_y += ((ty - ent.y) / d * strength) as f32;
                    ent.motion_z += ((tz - ent.z) / d * strength) as f32;
                }
                if ent.lifetime > 6000 {
                    to_remove.push(ent.id);
                    let rem_pk = RemoveEntityPacket { entity_runtime_id: ent.id };
                    broadcast_to_dim(rak, players, dim, rem_pk.encode()).await;
                    continue;
                }
            }

            // 2. Legacy Entity-based Projectiles (Arrow 80, Snowball 81, Egg 82, Ender Pearl 87)
            if matches!(ent.entity_type, 80 | 81 | 82 | 87) {
                ent.x += ent.motion_x as f64;
                ent.y += ent.motion_y as f64;
                ent.z += ent.motion_z as f64;
                ent.motion_y -= 0.05;
                ent.motion_x *= 0.99;
                ent.motion_z *= 0.99;

                let block_hit = {
                    let w = world.read();
                    w.get_block_id_opt(ent.x.floor() as i32, ent.y.floor() as i32, ent.z.floor() as i32) != 0
                };

                let mut hit_player_opt = None;
                for &(p_addr, p_eid, p_dim, px, py, pz, gm) in &player_snapshots {
                    if p_dim != dim || (Some(p_eid) == ent.owner_eid && ent.lifetime < 5) {
                        continue;
                    }
                    let dist_sq = (px - ent.x).powi(2) + (py + 1.0 - ent.y).powi(2) + (pz - ent.z).powi(2);
                    if dist_sq < 1.44 {
                        hit_player_opt = Some((p_addr, p_eid, gm));
                        break;
                    }
                }

                if block_hit || hit_player_opt.is_some() || ent.lifetime > 200 {
                    to_remove.push(ent.id);
                    let rem_pk = RemoveEntityPacket { entity_runtime_id: ent.id };
                    broadcast_to_dim(rak, players, dim, rem_pk.encode()).await;

                    if let Some((p_addr, _p_eid, gm)) = hit_player_opt {
                        if gm == 0 && matches!(ent.entity_type, 80 | 81) {
                            let dmg = if ent.entity_type == 80 { 3.0 } else { 0.0 };
                            if dmg > 0.0 {
                                player_attacks.push((dim, ent.id, ent.name.clone(), p_addr, dmg, ent.x, ent.y, ent.z));
                            }
                        }
                    } else if ent.entity_type == 81 {
                        // Snowball hitting a hostile mob
                        for &(hid, hdim, hx, hy, hz) in &hostile_targets {
                            if hdim == dim && Some(hid) != ent.owner_eid {
                                let d_sq = (hx - ent.x).powi(2) + (hy + 1.0 - ent.y).powi(2) + (hz - ent.z).powi(2);
                                if d_sq < 2.0 {
                                    mob_vs_mob.push((hid, ent.owner_eid.unwrap_or(ent.id), 1.0, ent.x, ent.y, ent.z));
                                    break;
                                }
                            }
                        }
                    }
                    continue;
                }
            }

            // 3. Item Entity Logic (64)
            if ent.entity_type == 64 {
                if ent.pickup_delay > 0 {
                    ent.pickup_delay -= 1;
                } else if let Some(ref it) = ent.item_drop {
                    for &(p_addr, _p_eid, p_dim, px, py, pz, gm) in &player_snapshots {
                        if p_dim != dim || gm == 3 {
                            continue;
                        }
                        let dist_sq = (px - ent.x).powi(2) + (py + 0.5 - ent.y).powi(2) + (pz - ent.z).powi(2);
                        if dist_sq <= 2.25 {
                            player_pickups.push((dim, p_addr, ent.id, it.clone(), px, py, pz));
                            to_remove.push(ent.id);
                            break;
                        }
                    }
                }
                if ent.lifetime > 6000 {
                    to_remove.push(ent.id);
                    let rem_pk = RemoveEntityPacket { entity_runtime_id: ent.id };
                    broadcast_to_dim(rak, players, dim, rem_pk.encode()).await;
                    continue;
                }
            }

            // 4. Mob AI
            let is_hostile = matches!(ent.entity_type, 32 | 33 | 34 | 35 | 36 | 37 | 38 | 40 | 41 | 42 | 43 | 45 | 46 | 47 | 48);
            let is_guardian = matches!(ent.entity_type, 20 | 21) || (ent.entity_type == 14 && ent.tamed_owner.is_some());
            let is_passive = matches!(ent.entity_type, 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 22) && !is_guardian;

            if is_hostile {
                let mut nearest = None;
                let mut min_dist_sq = 32.0 * 32.0;
                for &(p_addr, p_eid, p_dim, px, py, pz, gm) in &player_snapshots {
                    if p_dim == dim && gm == 0 {
                        let d_sq = (px - ent.x).powi(2) + (pz - ent.z).powi(2);
                        if d_sq < min_dist_sq {
                            min_dist_sq = d_sq;
                            nearest = Some((p_addr, p_eid, px, py, pz, d_sq.sqrt()));
                        }
                    }
                }

                if let Some((target_addr, _target_id, tx, ty, tz, dist)) = nearest {
                    let dx = tx - ent.x;
                    let dz = tz - ent.z;
                    ent.yaw = calculate_yaw(dx, dz);
                    ent.pitch = calculate_pitch(ent.x, ent.y + 1.6, ent.z, tx, ty + 1.0, tz);

                    match ent.entity_type {
                        32 | 47 | 48 => {
                            let speed = 0.23f32;
                            ent.motion_x = ((dx / dist.max(0.01)) as f32) * speed;
                            ent.motion_z = ((dz / dist.max(0.01)) as f32) * speed;
                            if dist <= 1.6 && ent.attack_cooldown == 0 {
                                ent.attack_cooldown = 20;
                                let dmg = if ent.entity_type == 48 { 5.0 } else { 3.0 };
                                player_attacks.push((dim, ent.id, ent.name.clone(), target_addr, dmg, ent.x, ty, ent.z));
                            }
                        }
                        33 => {
                            if dist > 2.5 {
                                let speed = 0.22f32;
                                ent.motion_x = ((dx / dist.max(0.01)) as f32) * speed;
                                ent.motion_z = ((dz / dist.max(0.01)) as f32) * speed;
                            }
                            if let Some(ref mut fuse) = ent.fuse {
                                if dist <= 3.0 {
                                    if *fuse == 30 {
                                        let ignite = LevelSoundEventPacket::new(level_event::SOUND_IGNITE, ent.x as f32, ent.y as f32, ent.z as f32);
                                        broadcast_to_dim(rak, players, dim, ignite.encode()).await;
                                    }
                                    if *fuse > 0 {
                                        *fuse -= 1;
                                    } else {
                                        to_remove.push(ent.id);
                                        let rem_pk = RemoveEntityPacket { entity_runtime_id: ent.id };
                                        broadcast_to_dim(rak, players, dim, rem_pk.encode()).await;
                                        explosions.push((dim, ent.x, ent.y, ent.z, 3.0));
                                        continue;
                                    }
                                } else if dist > 6.0 && *fuse < 30 {
                                    *fuse += 1;
                                }
                            }
                        }
                        34 | 46 => {
                            if dist > 14.0 {
                                ent.motion_x = ((dx / dist.max(0.01)) as f32) * 0.20;
                                ent.motion_z = ((dz / dist.max(0.01)) as f32) * 0.20;
                            } else if dist < 4.0 {
                                ent.motion_x = -((dx / dist.max(0.01)) as f32) * 0.15;
                                ent.motion_z = -((dz / dist.max(0.01)) as f32) * 0.15;
                            }
                            if dist <= 15.0 && dist >= 3.0 && ent.attack_cooldown == 0 {
                                ent.attack_cooldown = 50;
                                spawned_projectiles.push((
                                    dim, 80, ent.x, ent.y + 1.6, ent.z,
                                    ((dx / dist.max(0.01)) * 1.4) as f32,
                                    (((ty + 0.5 - ent.y) / dist.max(0.01)) * 1.4 + 0.1) as f32,
                                    ((dz / dist.max(0.01)) * 1.4) as f32,
                                    ent.yaw, ent.pitch, ent.id,
                                ));
                            }
                        }
                        35 | 40 => {
                            ent.motion_x = ((dx / dist.max(0.01)) as f32) * 0.28;
                            ent.motion_z = ((dz / dist.max(0.01)) as f32) * 0.28;
                            if dist <= 3.0 && dist > 1.2 && ent.on_ground {
                                ent.motion_y = 0.35;
                            }
                            if dist <= 1.5 && ent.attack_cooldown == 0 {
                                ent.attack_cooldown = 20;
                                player_attacks.push((dim, ent.id, ent.name.clone(), target_addr, 2.0, ent.x, ty, ent.z));
                            }
                        }
                        37 | 42 => {
                            if ent.on_ground && (ent.lifetime % 30) == 0 {
                                ent.motion_x = ((dx / dist.max(0.01)) as f32) * 0.25;
                                ent.motion_z = ((dz / dist.max(0.01)) as f32) * 0.25;
                                ent.motion_y = 0.38;
                            }
                            if dist <= (0.8 * ent.size as f64).max(1.2) && ent.attack_cooldown == 0 {
                                ent.attack_cooldown = 20;
                                let dmg = ent.size.max(1) as f32;
                                player_attacks.push((dim, ent.id, ent.name.clone(), target_addr, dmg, ent.x, ty, ent.z));
                            }
                        }
                        43 | 41 => {
                            // Blaze & Ghast hover + ranged
                            ent.motion_y = (ty + 2.5 - ent.y).clamp(-0.08, 0.08) as f32;
                            if dist > 12.0 {
                                ent.motion_x = ((dx / dist.max(0.01)) as f32) * 0.16;
                                ent.motion_z = ((dz / dist.max(0.01)) as f32) * 0.16;
                            }
                            if dist <= 20.0 && ent.attack_cooldown == 0 {
                                ent.attack_cooldown = if ent.entity_type == 41 { 80 } else { 55 };
                                spawned_projectiles.push((
                                    dim, 80, ent.x, ent.y + 1.4, ent.z,
                                    ((dx / dist.max(0.01)) * 1.4) as f32,
                                    (((ty + 0.8 - ent.y) / dist.max(0.01)) * 1.4) as f32,
                                    ((dz / dist.max(0.01)) * 1.4) as f32,
                                    ent.yaw, ent.pitch, ent.id,
                                ));
                            }
                        }
                        45 => {
                            // Witch
                            if dist > 9.0 {
                                ent.motion_x = ((dx / dist.max(0.01)) as f32) * 0.18;
                                ent.motion_z = ((dz / dist.max(0.01)) as f32) * 0.18;
                            }
                            if dist <= 10.0 && ent.attack_cooldown == 0 {
                                ent.attack_cooldown = 60;
                                player_attacks.push((dim, ent.id, ent.name.clone(), target_addr, 3.0, ent.x, ty, ent.z));
                            }
                        }
                        _ => {
                            ent.motion_x = ((dx / dist.max(0.01)) as f32) * 0.20;
                            ent.motion_z = ((dz / dist.max(0.01)) as f32) * 0.20;
                        }
                    }
                }
            } else if is_guardian {
                // Iron Golem (20), Snow Golem (21), Tamed Wolf (14)
                let mut best_hostile: Option<(i64, f64, f64, f64, f64)> = None;
                for &(hid, hdim, hx, hy, hz) in &hostile_targets {
                    if hdim != dim {
                        continue;
                    }
                    let d_sq = (hx - ent.x).powi(2) + (hz - ent.z).powi(2);
                    if d_sq <= 256.0 && best_hostile.map_or(true, |b| d_sq < b.4) {
                        best_hostile = Some((hid, hx, hy, hz, d_sq));
                    }
                }

                if ent.entity_type == 14 && ent.sitting {
                    ent.motion_x = 0.0;
                    ent.motion_z = 0.0;
                } else if let Some((hid, hx, hy, hz, d_sq)) = best_hostile {
                    let dist = d_sq.sqrt().max(0.01);
                    let dx = hx - ent.x;
                    let dz = hz - ent.z;
                    ent.yaw = calculate_yaw(dx, dz);
                    if ent.entity_type == 21 {
                        if dist <= 12.0 && ent.attack_cooldown == 0 {
                            ent.attack_cooldown = 25;
                            spawned_projectiles.push((
                                dim, 81, ent.x, ent.y + 1.5, ent.z,
                                ((dx / dist) * 1.2) as f32,
                                (((hy + 0.8 - ent.y) / dist) * 1.2 + 0.1) as f32,
                                ((dz / dist) * 1.2) as f32,
                                ent.yaw, ent.pitch, ent.id,
                            ));
                        }
                    } else {
                        ent.motion_x = ((dx / dist) as f32) * 0.25;
                        ent.motion_z = ((dz / dist) as f32) * 0.25;
                        if dist <= 2.2 && ent.attack_cooldown == 0 {
                            ent.attack_cooldown = 25;
                            let dmg = if ent.entity_type == 20 { 10.0 } else { 4.0 };
                            mob_vs_mob.push((hid, ent.id, dmg, ent.x, ent.y, ent.z));
                        }
                    }
                } else if ent.entity_type == 14 {
                    if let Some(owner_eid) = ent.tamed_owner {
                        if let Some(&(_, _, p_dim, px, py, pz, _)) = player_snapshots.iter().find(|p| p.1 == owner_eid) {
                            if p_dim == dim {
                                let d_sq = (px - ent.x).powi(2) + (pz - ent.z).powi(2);
                                if d_sq > 196.0 {
                                    ent.x = px;
                                    ent.y = py;
                                    ent.z = pz;
                                } else if d_sq > 12.0 {
                                    let d = d_sq.sqrt();
                                    ent.motion_x = ((px - ent.x) / d * 0.26) as f32;
                                    ent.motion_z = ((pz - ent.z) / d * 0.26) as f32;
                                    ent.yaw = calculate_yaw(px - ent.x, pz - ent.z);
                                }
                            }
                        }
                    }
                }
            } else if is_passive {
                // Sheep grazes grass to regrow wool
                if ent.entity_type == 13 && (ent.lifetime % 200) == 0 && rand::random::<f32>() < 0.25 {
                    let gx = ent.x.floor() as i32;
                    let gy = ent.y.floor() as i32 - 1;
                    let gz = ent.z.floor() as i32;
                    let on_grass = world.read().get_block_id_opt(gx, gy, gz) == 2;
                    if on_grass {
                        world.write().set_block(gx, gy, gz, 3, 0);
                        block_changes.push((dim, gx, gy, gz, 3, 0));
                        ent.sheared = false;
                        let eat_ev = EntityEventPacket::new(ent.id, EntityEventPacket::EAT_GRASS_ANIMATION);
                        let meta_pk = SetEntityDataPacket { entity_runtime_id: ent.id, metadata: build_entity_metadata(ent) };
                        broadcast_to_dim(rak, players, dim, eat_ev.encode()).await;
                        broadcast_to_dim(rak, players, dim, meta_pk.encode()).await;
                    }
                }
                if ent.lifetime % 60 == 0 && rand::random::<f32>() < 0.4 {
                    let angle = rand::random::<f64>() * (std::f64::consts::PI * 2.0);
                    let speed = 0.12f32;
                    ent.motion_x = (angle.cos() as f32) * speed;
                    ent.motion_z = (angle.sin() as f32) * speed;
                    ent.yaw = calculate_yaw(ent.motion_x as f64, ent.motion_z as f64);
                }
            }

            if ent.attack_cooldown > 0 {
                ent.attack_cooldown -= 1;
            }

            // 5. AABB Physics, Gravity, Step-up & Fall Damage
            let no_gravity = matches!(ent.entity_type, 41 | 43 | 80 | 81 | 82 | 87);
            if !no_gravity && !matches!(ent.entity_type, 80 | 81 | 82 | 87) {
                ent.motion_y = (ent.motion_y - 0.05).max(-1.5);
                let (hw, h) = physics::entity_half_size(ent.entity_type);
                let step = if matches!(ent.entity_type, 35 | 40) { 1.0 } else if ent.entity_type == 64 || ent.entity_type == 69 { 0.0 } else { 0.55 };
                let res = {
                    let w = world.read();
                    physics::move_entity_with_collision(
                        &w,
                        ent.x,
                        ent.y,
                        ent.z,
                        ent.motion_x as f64,
                        ent.motion_y as f64,
                        ent.motion_z as f64,
                        hw,
                        h,
                        step,
                        ent.on_ground,
                    )
                };

                ent.x = res.x;
                ent.y = res.y;
                ent.z = res.z;

                if res.on_ground {
                    if ent.fall_distance > 3.0 && !matches!(ent.entity_type, 10 | 20 | 21 | 37 | 38 | 41 | 42 | 43 | 64 | 65 | 69) {
                        let fd = (ent.fall_distance - 3.0).floor();
                        if fd > 0.0 {
                            ent.health -= fd;
                            if ent.health <= 0.0 {
                                ent.is_dead = true;
                                ent.dead_ticks = 0;
                            }
                        }
                    }
                    ent.fall_distance = 0.0;
                    ent.motion_y = 0.0;
                    if res.collided_horizontally && (is_hostile || is_guardian || is_passive) {
                        ent.motion_y = 0.42;
                    }
                } else if res.dy < 0.0 {
                    ent.fall_distance += (-res.dy) as f32;
                }
                ent.on_ground = res.on_ground;
                if res.collided_horizontally {
                    ent.motion_x = 0.0;
                    ent.motion_z = 0.0;
                } else {
                    ent.motion_x *= 0.82;
                    ent.motion_z *= 0.82;
                }
            }
        }

        // Apply grass eating block updates
        for (dim, bx, by, bz, bid, bmeta) in block_changes {
            let pk = UpdateBlockPacket::new(bx, by as u32, bz, bid as u32, bmeta as u32, update_block::FLAG_ALL);
            broadcast_to_dim(rak, players, dim, pk.encode()).await;
        }

        // Apply hostile player attacks (with full armor/protection/thorns/totem pipeline)
        for (_dim, attacker_id, attacker_name, target_addr, damage, ax, ay, az) in player_attacks {
            let (target_eid, victim_name, tx, ty, tz) = {
                let p = players.read();
                if let Some(s) = p.get(&target_addr) {
                    (s.entity_id, s.username.clone(), s.x, s.y, s.z)
                } else {
                    continue;
                }
            };
            let msg = format!("{} was slain by {}", victim_name, attacker_name);
            let outcome = crate::entity::damage::hurt_player(
                players,
                rak,
                target_addr,
                damage,
                crate::entity::damage::DamageCause::EntityAttack,
                false,
                Some((ax, ay, az)),
                None,
                Some(&msg),
            )
            .await;

            if outcome.thorns_damage > 0.0 {
                self.damage_entity(attacker_id, outcome.thorns_damage, target_eid, Some((tx, ty, tz)), rak, players).await;
            }
        }

        // Spawn skeleton/blaze/snowgolem projectiles
        for (dim, etype, sx, sy, sz, vx, vy, vz, yaw, pitch, owner_id) in spawned_projectiles {
            let proj_eid = {
                let mut eid = next_eid.write();
                let id = *eid;
                *eid += 1;
                id
            };
            self.spawn_projectile_in_dim(proj_eid, dim, etype, sx, sy, sz, vx, vy, vz, yaw, pitch, owner_id, rak, players).await;
            let snd = LevelSoundEventPacket::new(level_event::SOUND_BOW, sx as f32, sy as f32, sz as f32);
            broadcast_to_dim(rak, players, dim, snd.encode()).await;
        }

        // Apply explosions
        for (dim, ex, ey, ez, radius) in explosions {
            let world = worlds[dim.clamp(0, 2) as usize];
            let res = {
                let mut w = world.write();
                crate::level::explosion::compute_explosion(&mut w, ex, ey, ez, radius)
            };

            for (bx, by, bz, nid, nmeta) in res.destroyed_blocks {
                let pk = UpdateBlockPacket::new(bx, by as u32, bz, nid as u32, nmeta as u32, update_block::FLAG_ALL);
                broadcast_to_dim(rak, players, dim, pk.encode()).await;
            }

            for (tx, ty, tz, fuse) in res.ignited_tnts {
                let tid = {
                    let mut eid = next_eid.write();
                    let id = *eid;
                    *eid += 1;
                    id
                };
                self.spawn_tnt_in_dim(tid, dim, tx, ty, tz, fuse, rak, players).await;
            }

            for (dx, dy, dz, item) in res.drops {
                drops.push((dim, item, dx as f64 + 0.5, dy as f64 + 0.5, dz as f64 + 0.5));
            }

            let mut blasted_players = Vec::new();
            {
                let p = players.read();
                for (addr, s) in p.iter() {
                    if s.dimension != dim || !s.spawned || s.gamemode != 0 {
                        continue;
                    }
                    if let Some((dmg, _vx, _vy, _vz)) = crate::level::explosion::explosion_damage_and_knockback(s.x, s.y, s.z, ex, ey, ez, radius) {
                        blasted_players.push((*addr, dmg, s.username.clone()));
                    }
                }
            }
            for (addr, dmg, uname) in blasted_players {
                let msg = format!("{} blew up", uname);
                crate::entity::damage::hurt_player(
                    players,
                    rak,
                    addr,
                    dmg,
                    crate::entity::damage::DamageCause::EntityExplosion,
                    false,
                    Some((ex, ey, ez)),
                    None,
                    Some(&msg),
                )
                .await;
            }

            for ent in &mut list {
                if ent.dimension != dim || ent.is_dead || to_remove.contains(&ent.id) {
                    continue;
                }
                if matches!(ent.entity_type, 64 | 65 | 68 | 69 | 80 | 81 | 82 | 86 | 87 | 93) {
                    continue;
                }
                if let Some((dmg, vx, vy, vz)) = crate::level::explosion::explosion_damage_and_knockback(ent.x, ent.y, ent.z, ex, ey, ez, radius) {
                    ent.health -= dmg;
                    ent.motion_x += vx;
                    ent.motion_y += vy;
                    ent.motion_z += vz;
                    if ent.health <= 0.0 {
                        ent.is_dead = true;
                        ent.dead_ticks = 0;
                    }
                }
            }

            let explode_pk = ExplodePacket::new(ex as f32, ey as f32, ez as f32, radius);
            broadcast_to_dim(rak, players, dim, explode_pk.encode()).await;
            let sound = LevelSoundEventPacket::new(level_event::SOUND_EXPLODE, ex as f32, ey as f32, ez as f32);
            broadcast_to_dim(rak, players, dim, sound.encode()).await;
        }

        // Apply XP Orb pickups
        for (dim, p_addr, xp_val, ox, oy, oz) in xp_pickups {
            let orb_snd = LevelEventPacket::new(level_event::EVENT_SOUND_ORB, ox as f32, oy as f32, oz as f32, 0);
            broadcast_to_dim(rak, players, dim, orb_snd.encode()).await;

            let attr_up = {
                let mut p = players.write();
                p.get_mut(&p_addr).map(|s| {
                    s.xp_points += xp_val;
                    let req = |lvl: u32| -> f32 {
                        if lvl <= 16 { (2 * lvl + 7) as f32 } else if lvl <= 31 { (5 * lvl - 38) as f32 } else { (9 * lvl - 158) as f32 }
                    };
                    let mut prog_xp = s.xp_progress * req(s.xp_level) + xp_val as f32;
                    while prog_xp >= req(s.xp_level) {
                        prog_xp -= req(s.xp_level);
                        s.xp_level += 1;
                    }
                    s.xp_progress = (prog_xp / req(s.xp_level)).clamp(0.0, 0.9999);
                    (s.entity_id, s.xp_level as f32, s.xp_progress)
                })
            };
            if let Some((peid, lvl, prog)) = attr_up {
                let mut ap = UpdateAttributesPacket::new(peid);
                ap.attributes = vec![Attribute::experience_level(lvl), Attribute::experience_progress(prog)];
                let batch = BatchPacket::encode(&[ap.encode()]);
                rak.send_encapsulated(p_addr, batch, true).await;
            }
        }

        // Handle item pickups by players + sync player inventory
        for (dim, p_addr, item_eid, item, px, py, pz) in player_pickups {
            let p_eid = players.read().get(&p_addr).map(|s| s.entity_id).unwrap_or(0);
            let take_pk = TakeItemEntityPacket { target: p_eid, entity_id: item_eid };
            broadcast_to_dim(rak, players, dim, take_pk.encode()).await;

            let pop_snd = LevelSoundEventPacket::new(level_event::SOUND_POP, px as f32, py as f32, pz as f32);
            broadcast_to_dim(rak, players, dim, pop_snd.encode()).await;

            let inv_pk = {
                let mut p = players.write();
                p.get_mut(&p_addr).map(|state| {
                    let _ = state.inventory.add_item(item);
                    let mut pk = ContainerSetContentPacket::new(ContainerIds::INVENTORY as u32);
                    pk.target_eid = state.entity_id;
                    pk.items = state.inventory.wire_contents();
                    pk.hotbar = state.inventory.wire_hotbar_links();
                    pk.encode()
                })
            };
            if let Some(encoded) = inv_pk {
                rak.send_encapsulated(p_addr, BatchPacket::encode(&[encoded]), true).await;
            }
        }

        // Remove despawned or dead entities
        if !to_remove.is_empty() {
            let mut guard = self.entities.write();
            for id in &to_remove {
                guard.remove(id);
            }
        }

        // Spawn drops, XP orbs, and child mobs
        for (dim, item, x, y, z) in drops {
            let drop_eid = {
                let mut eid = next_eid.write();
                let id = *eid;
                *eid += 1;
                id
            };
            self.spawn_item_drop_in_dim(drop_eid, dim, item, x, y, z, rak, players).await;
        }
        for (dim, xp, x, y, z) in xp_spawns {
            self.spawn_xp_orbs_split_in_dim(dim, x, y, z, xp, next_eid, rak, players).await;
        }
        for (dim, etype, name, x, y, z, hp, age, size, col) in child_mobs {
            let cid = {
                let mut eid = next_eid.write();
                let id = *eid;
                *eid += 1;
                id
            };
            self.spawn_mob_custom_in_dim(cid, dim, etype, name, x, y, z, hp, age, size, col, rak, players).await;
        }

        // Apply guardian/mob-vs-mob hits
        for (target_id, attacker_id, dmg, ax, ay, az) in mob_vs_mob {
            self.damage_entity(target_id, dmg, attacker_id, Some((ax, ay, az)), rak, players).await;
        }

        // Broadcast motion updates grouped by dimension
        let mut by_dim: HashMap<i32, Vec<Vec<u8>>> = HashMap::new();
        for ent in &list {
            if !to_remove.contains(&ent.id) && !ent.is_dead && (ent.motion_x.abs() > 0.01 || ent.motion_y.abs() > 0.01 || ent.motion_z.abs() > 0.01) {
                let mv = MoveEntityPacket::new(ent.id, ent.x as f32, ent.y as f32, ent.z as f32, ent.yaw, ent.pitch);
                by_dim.entry(ent.dimension).or_default().push(mv.encode());
            }
        }
        for (dim, packets) in by_dim {
            if packets.is_empty() {
                continue;
            }
            let targets: Vec<SocketAddr> = {
                let p = players.read();
                p.iter()
                    .filter(|(_, s)| s.dimension == dim && s.spawned)
                    .map(|(addr, _)| *addr)
                    .collect()
            };
            if !targets.is_empty() {
                let batch = BatchPacket::encode(&packets);
                for addr in targets {
                    rak.send_encapsulated(addr, batch.clone(), false).await;
                }
            }
        }

        // Write back updated entities into the map
        {
            let mut guard = self.entities.write();
            for ent in list {
                if !to_remove.contains(&ent.id) {
                    guard.insert(ent.id, ent);
                }
            }
        }
    }
}

async fn broadcast_to_dim(
    rak: &RakNetServer,
    players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    dimension: i32,
    packet: Vec<u8>,
) {
    let targets: Vec<SocketAddr> = {
        let p = players.read();
        p.iter()
            .filter(|(_, s)| s.dimension == dimension && s.spawned)
            .map(|(addr, _)| *addr)
            .collect()
    };
    if targets.is_empty() {
        return;
    }
    let batch = BatchPacket::encode(&[packet]);
    for addr in targets {
        rak.send_encapsulated(addr, batch.clone(), true).await;
    }
}

#[allow(dead_code)]
async fn broadcast_to_all(
    rak: &RakNetServer,
    players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    packet: Vec<u8>,
) {
    let targets: Vec<SocketAddr> = {
        let p = players.read();
        p.keys().copied().collect()
    };
    if targets.is_empty() { return; }
    let batch = BatchPacket::encode(&[packet]);
    for addr in targets {
        rak.send_encapsulated(addr, batch.clone(), true).await;
    }
}
