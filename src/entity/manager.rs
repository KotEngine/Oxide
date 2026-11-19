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
    EntityEventPacket,
    level_event, update_block,
};
use crate::protocol::packet::DataPacket;
use crate::protocol::batch::BatchPacket;
use crate::protocol::types::Item;

#[derive(Debug, Clone)]
pub struct ActiveEntity {
    pub id: i64,
    pub entity_type: i32,
    pub name: String,
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
        let ent = ActiveEntity {
            id: entity_id,
            entity_type,
            name,
            x, y, z,
            motion_x: 0.0,
            motion_y: 0.0,
            motion_z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            health,
            max_health: health,
            fuse: if entity_type == 33 { Some(30) } else { None }, // Creeper default fuse
            lifetime: 0,
            owner_eid: None,
            item_drop: None,
            pickup_delay: 0,
            attack_cooldown: 0,
        };

        self.entities.write().insert(entity_id, ent);

        let mut add_pk = AddEntityPacket::new(entity_id, entity_type as u32, x as f32, y as f32, z as f32);
        add_pk.yaw = 0.0;
        add_pk.pitch = 0.0;
        broadcast_to_all(rak, players, add_pk.encode()).await;
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
        let ent = ActiveEntity {
            id: entity_id,
            entity_type: 65, // PrimedTNT entity type ID
            name: "PrimedTNT".to_string(),
            x, y, z,
            motion_x: 0.0,
            motion_y: 0.2, // Small upward pop
            motion_z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            health: 1.0,
            max_health: 1.0,
            fuse: Some(fuse_ticks),
            lifetime: 0,
            owner_eid: None,
            item_drop: None,
            pickup_delay: 0,
            attack_cooldown: 0,
        };

        self.entities.write().insert(entity_id, ent);

        let mut add_pk = AddEntityPacket::new(entity_id, 65, x as f32, y as f32, z as f32);
        add_pk.speed_y = 0.2;
        broadcast_to_all(rak, players, add_pk.encode()).await;

        let ignite_sound = LevelEventPacket::new(level_event::EVENT_SOUND_IGNITE, x as f32, y as f32, z as f32, 0);
        broadcast_to_all(rak, players, ignite_sound.encode()).await;
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
        let ent = ActiveEntity {
            id: entity_id,
            entity_type,
            name: format!("Projectile_{}", entity_type),
            x, y, z,
            motion_x: vx,
            motion_y: vy,
            motion_z: vz,
            yaw,
            pitch,
            health: 1.0,
            max_health: 1.0,
            fuse: None,
            lifetime: 0,
            owner_eid: Some(owner_eid),
            item_drop: None,
            pickup_delay: 0,
            attack_cooldown: 0,
        };

        self.entities.write().insert(entity_id, ent);

        let mut add_pk = AddEntityPacket::new(entity_id, entity_type as u32, x as f32, y as f32, z as f32);
        add_pk.speed_x = vx;
        add_pk.speed_y = vy;
        add_pk.speed_z = vz;
        add_pk.yaw = yaw;
        add_pk.pitch = pitch;
        broadcast_to_all(rak, players, add_pk.encode()).await;
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
        let ent = ActiveEntity {
            id: entity_id,
            entity_type: 64, // ItemEntity
            name: "Item".to_string(),
            x, y, z,
            motion_x: 0.0,
            motion_y: 0.2,
            motion_z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            health: 5.0,
            max_health: 5.0,
            fuse: None,
            lifetime: 0,
            owner_eid: None,
            item_drop: Some(item.clone()),
            pickup_delay: 15, // 0.75s before pickup
            attack_cooldown: 0,
        };

        self.entities.write().insert(entity_id, ent);

        let mut pk = AddItemEntityPacket::new(entity_id, item, x as f32, y as f32, z as f32);
        pk.speed_y = 0.2;
        broadcast_to_all(rak, players, pk.encode()).await;
    }

    pub async fn damage_entity(
        &self,
        entity_id: i64,
        damage: f32,
        _attacker_eid: i64,
        rak: &RakNetServer,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    ) -> bool {
        let mut ent_guard = self.entities.write();
        if let Some(ent) = ent_guard.get_mut(&entity_id) {
            ent.health -= damage;
            let dead = ent.health <= 0.0;

            let anim = EntityEventPacket::new(entity_id, 2); // Hurt animation
            let sound = LevelSoundEventPacket::new(level_event::SOUND_HIT, ent.x as f32, ent.y as f32, ent.z as f32);
            drop(ent_guard);

            broadcast_to_all(rak, players, anim.encode()).await;
            broadcast_to_all(rak, players, sound.encode()).await;

            return dead;
        }
        false
    }

    pub async fn tick(
        &self,
        world: &Arc<RwLock<World>>,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
        rak: &RakNetServer,
        next_eid: &Arc<RwLock<i64>>,
    ) {
        let mut list: Vec<ActiveEntity> = {
            let guard = self.entities.read();
            if guard.is_empty() {
                return;
            }
            guard.values().cloned().collect()
        };

        let mut to_remove = Vec::new();
        let mut explosions = Vec::new();
        let mut drops: Vec<(i32, u32, f64, f64, f64)> = Vec::new();
        let mut teleports: Vec<(i64, f64, f64, f64)> = Vec::new();
        let mut player_pickups: Vec<(SocketAddr, i64, Item)> = Vec::new();

        // Snapshot current player positions
        let player_snapshots: Vec<(SocketAddr, i64, f64, f64, f64, i32)> = {
            let p = players.read();
            p.iter().map(|(addr, s)| (*addr, s.entity_id, s.x, s.y, s.z, s.gamemode)).collect()
        };

        for ent in &mut list {
            ent.lifetime += 1;

            // 1. TNT Fuse Logic
            if let Some(ref mut fuse) = ent.fuse {
                if ent.entity_type == 65 {
                    if *fuse > 0 {
                        *fuse -= 1;
                    } else {
                        to_remove.push(ent.id);
                        explosions.push((ent.x, ent.y, ent.z));
                        continue;
                    }
                }
            }

            // 2. Projectile Logic (Arrow 80, Snowball 81, Egg 82, Ender Pearl 87)
            if ent.entity_type == 80 || ent.entity_type == 81 || ent.entity_type == 82 || ent.entity_type == 87 {
                ent.x += ent.motion_x as f64;
                ent.y += ent.motion_y as f64;
                ent.z += ent.motion_z as f64;
                ent.motion_y -= 0.05; // Gravity
                ent.motion_x *= 0.99;
                ent.motion_z *= 0.99;

                // Check collision with solid blocks
                let bx = ent.x.floor() as i32;
                let by = ent.y.floor() as i32;
                let bz = ent.z.floor() as i32;
                let block_hit = {
                    let w = world.read();
                    w.get_block_id_opt(bx, by, bz) != 0
                };

                // Check collision with players
                let mut hit_player_opt = None;
                for &(p_addr, p_eid, px, py, pz, _gm) in &player_snapshots {
                    if Some(p_eid) == ent.owner_eid && ent.lifetime < 5 {
                        continue; // Cannot hit shooter immediately
                    }
                    let dist_sq = (px - ent.x).powi(2) + (py + 1.0 - ent.y).powi(2) + (pz - ent.z).powi(2);
                    if dist_sq < 1.44 {
                        hit_player_opt = Some((p_addr, p_eid));
                        break;
                    }
                }

                if block_hit || hit_player_opt.is_some() || ent.lifetime > 200 {
                    to_remove.push(ent.id);

                    if ent.entity_type == 87 {
                        // Ender pearl teleports owner
                        if let Some(owner) = ent.owner_eid {
                            teleports.push((owner, ent.x, ent.y + 0.1, ent.z));
                        }
                    } else if ent.entity_type == 82 {
                        // Egg 1/8 chance to spawn baby chicken
                        if (ent.lifetime % 8) == 0 {
                            drops.push((365, 1, ent.x, ent.y, ent.z));
                        }
                    } else if ent.entity_type == 80 {
                        // Arrow hit sound
                        let hit_snd = LevelSoundEventPacket::new(level_event::SOUND_BOW_HIT, ent.x as f32, ent.y as f32, ent.z as f32);
                        broadcast_to_all(rak, players, hit_snd.encode()).await;
                    }
                    continue;
                }
            }

            // 3. Item Entity Logic (64)
            if ent.entity_type == 64 {
                if ent.pickup_delay > 0 {
                    ent.pickup_delay -= 1;
                } else if let Some(ref it) = ent.item_drop {
                    for &(p_addr, _p_eid, px, py, pz, _gm) in &player_snapshots {
                        let dist_sq = (px - ent.x).powi(2) + (py - ent.y).powi(2) + (pz - ent.z).powi(2);
                        if dist_sq <= 2.25 { // within 1.5 blocks
                            player_pickups.push((p_addr, ent.id, it.clone()));
                            to_remove.push(ent.id);
                            break;
                        }
                    }
                }

                // Despawn item after 5 minutes (6000 ticks)
                if ent.lifetime > 6000 {
                    to_remove.push(ent.id);
                    continue;
                }
            }

            // 4. Mob AI & Physics (Zombie 32, Skeleton 34, Creeper 33, Spider 35, Cow 11, Pig 12, Sheep 13, Chicken 10)
            let is_mob = ent.entity_type == 32 || ent.entity_type == 33 || ent.entity_type == 34
                || ent.entity_type == 35 || ent.entity_type == 11 || ent.entity_type == 12
                || ent.entity_type == 13 || ent.entity_type == 10;

            if is_mob {
                // Find nearest player in survival
                let mut nearest = None;
                let mut min_dist_sq = 256.0f64; // 16 block sight range

                for &(_p_addr, p_eid, px, py, pz, gm) in &player_snapshots {
                    if gm == 0 { // Survival
                        let d_sq = (px - ent.x).powi(2) + (pz - ent.z).powi(2);
                        if d_sq < min_dist_sq {
                            min_dist_sq = d_sq;
                            nearest = Some((p_eid, px, py, pz, d_sq.sqrt()));
                        }
                    }
                }

                if let Some((_target_id, tx, _ty, tz, dist)) = nearest {
                    let is_hostile = ent.entity_type == 32 || ent.entity_type == 33
                        || ent.entity_type == 34 || ent.entity_type == 35;

                    if is_hostile {
                        let dx = tx - ent.x;
                        let dz = tz - ent.z;
                        let speed = 0.22f32;
                        ent.motion_x = ((dx / dist) as f32) * speed;
                        ent.motion_z = ((dz / dist) as f32) * speed;
                        ent.yaw = ((-dx).atan2(dz).to_degrees()) as f32;

                        // Creeper flash & explode within 3 blocks
                        if ent.entity_type == 33 && dist < 3.0 {
                            if let Some(ref mut fuse) = ent.fuse {
                                if *fuse > 0 {
                                    *fuse -= 1;
                                } else {
                                    to_remove.push(ent.id);
                                    explosions.push((ent.x, ent.y, ent.z));
                                    continue;
                                }
                            }
                        }
                    }
                }

                if ent.attack_cooldown > 0 {
                    ent.attack_cooldown -= 1;
                }
            }

            // 5. Gravity & Floor Collision
            let floor_y = {
                let w = world.read();
                let bx = ent.x.floor() as i32;
                let bz = ent.z.floor() as i32;
                let mut check_y = ent.y.floor() as i32;
                while check_y >= 0 && w.get_block_id_opt(bx, check_y, bz) == 0 {
                    check_y -= 1;
                }
                (check_y + 1) as f64
            };

            if ent.y > floor_y {
                ent.motion_y = (ent.motion_y - 0.04).max(-1.5);
                ent.y += ent.motion_y as f64;
                if ent.y < floor_y {
                    ent.y = floor_y;
                    ent.motion_y = 0.0;
                }
            } else {
                ent.y = floor_y;
                ent.motion_y = 0.0;
            }

            ent.x += ent.motion_x as f64;
            ent.z += ent.motion_z as f64;
            ent.motion_x *= 0.90;
            ent.motion_z *= 0.90;
        }

        // Apply explosions
        for (ex, ey, ez) in explosions {
            let radius = 3.5f32;
            let sound = LevelEventPacket::new(level_event::EVENT_SOUND_FIZZ, ex as f32, ey as f32, ez as f32, 0);
            broadcast_to_all(rak, players, sound.encode()).await;

            let mut destroyed_blocks = Vec::new();
            let min_x = (ex - radius as f64).floor() as i32;
            let max_x = (ex + radius as f64).ceil() as i32;
            let min_y = ((ey - radius as f64).floor() as i32).max(1);
            let max_y = ((ey + radius as f64).ceil() as i32).min(255);
            let min_z = (ez - radius as f64).floor() as i32;
            let max_z = (ez + radius as f64).ceil() as i32;

            {
                let mut w = world.write();
                for bx in min_x..=max_x {
                    for by in min_y..=max_y {
                        for bz in min_z..=max_z {
                            let dist_sq = (bx as f64 - ex).powi(2) + (by as f64 - ey).powi(2) + (bz as f64 - ez).powi(2);
                            if dist_sq <= (radius as f64).powi(2) {
                                let id = w.get_block_id(bx, by, bz);
                                if id != 0 && id != 7 && id != 49 { // Bedrock and obsidian resist explosions
                                    w.set_block(bx, by, bz, 0, 0);
                                    destroyed_blocks.push((bx, by as u32, bz));
                                }
                            }
                        }
                    }
                }
            }

            // Broadcast block destruction
            for (bx, by, bz) in destroyed_blocks {
                let pk = UpdateBlockPacket::new(bx, by, bz, 0, 0, update_block::FLAG_ALL);
                broadcast_to_all(rak, players, pk.encode()).await;
            }

            let explode_pk = ExplodePacket::new(ex as f32, ey as f32, ez as f32, radius);
            broadcast_to_all(rak, players, explode_pk.encode()).await;
        }

        // Handle item pickups by players
        for (p_addr, item_eid, item) in player_pickups {
            let p_eid = players.read().get(&p_addr).map(|s| s.entity_id).unwrap_or(0);
            let take_pk = TakeItemEntityPacket { target: p_eid, entity_id: item_eid };
            broadcast_to_all(rak, players, take_pk.encode()).await;

            let pop_snd = LevelSoundEventPacket::new(level_event::SOUND_POP, 0.0, 0.0, 0.0);
            broadcast_to_all(rak, players, pop_snd.encode()).await;

            if let Some(state) = players.write().get_mut(&p_addr) {
                let _ = state.inventory.add_item(item);
            }
        }

        // Remove despawned or dead entities
        if !to_remove.is_empty() {
            let mut guard = self.entities.write();
            for id in &to_remove {
                guard.remove(id);
            }
        }

        // Spawn drops
        for (item_id, count, x, y, z) in drops {
            let drop_eid = {
                let mut eid = next_eid.write();
                let id = *eid;
                *eid += 1;
                id
            };
            self.spawn_item_drop(drop_eid, Item::new(item_id, 0, count), x, y, z, rak, players).await;
        }

        // Broadcast motion updates
        for ent in &list {
            if !to_remove.contains(&ent.id) && (ent.motion_x.abs() > 0.01 || ent.motion_z.abs() > 0.01) {
                let mv = MoveEntityPacket::new(ent.id, ent.x as f32, ent.y as f32, ent.z as f32, ent.yaw, ent.pitch);
                broadcast_to_all(rak, players, mv.encode()).await;
            }
        }
    }
}

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
