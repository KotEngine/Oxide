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
    EntityEventPacket, RemoveEntityPacket, SetHealthPacket, AnimatePacket,
    level_event, update_block,
};
use crate::protocol::packet::DataPacket;
use crate::protocol::batch::BatchPacket;
use crate::protocol::types::Item;
use crate::entity::loot::LootTable;

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
    pub dead_ticks: u32,
    pub is_dead: bool,
    pub on_fire_ticks: u32,
    pub killer_eid: Option<i64>,
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
            dead_ticks: 0,
            is_dead: false,
            on_fire_ticks: 0,
            killer_eid: None,
        };

        self.entities.write().insert(entity_id, ent);

        let mut add_pk = AddEntityPacket::new(entity_id, entity_type as u32, x as f32, y as f32, z as f32);
        add_pk.yaw = 0.0;
        add_pk.pitch = 0.0;
        broadcast_to_all(rak, players, add_pk.encode()).await;
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
        // 1. Broadcast AddEntityPacket (type 93 = Lightning Bolt)
        let add_pk = AddEntityPacket::new(entity_id, 93, x as f32, y as f32, z as f32);
        broadcast_to_all(rak, players, add_pk.encode()).await;

        // 2. Broadcast ExplodePacket for thunder clap sound and camera shake
        let explode_pk = ExplodePacket::new(x as f32, y as f32, z as f32, 10.0);
        broadcast_to_all(rak, players, explode_pk.encode()).await;

        // 3. Place fire on ground if air above solid block
        let bx = x.floor() as i32;
        let by = y.floor() as i32;
        let bz = z.floor() as i32;
        {
            let mut w = world.write();
            if w.get_block_id_opt(bx, by, bz) != 0 && w.get_block_id_opt(bx, by + 1, bz) == 0 {
                w.set_block(bx, by + 1, bz, 51, 0); // Fire block (51)
            }
        }
        let fire_pk = UpdateBlockPacket::new(bx, (by + 1) as u32, bz, 51, 0, update_block::FLAG_ALL);
        broadcast_to_all(rak, players, fire_pk.encode()).await;

        // 4. Transform mobs near strike (Pig 12 -> PigZombie 36)
        let mut to_transform = Vec::new();
        {
            let mut guard = self.entities.write();
            for ent in guard.values_mut() {
                let dist_sq = (ent.x - x).powi(2) + (ent.z - z).powi(2);
                if dist_sq <= 16.0 {
                    if ent.entity_type == 12 { // Pig
                        ent.entity_type = 36; // PigZombie
                        ent.name = "Zombie Pigman".to_string();
                        ent.health = 20.0;
                        ent.max_health = 20.0;
                        to_transform.push((ent.id, 36, ent.x, ent.y, ent.z));
                    }
                }
            }
        }

        for (mob_id, new_type, mx, my, mz) in to_transform {
            let rem_pk = RemoveEntityPacket { entity_runtime_id: mob_id };
            broadcast_to_all(rak, players, rem_pk.encode()).await;

            let add_pk = AddEntityPacket::new(mob_id, new_type as u32, mx as f32, my as f32, mz as f32);
            broadcast_to_all(rak, players, add_pk.encode()).await;
        }

        // 5. Strike nearby players in survival
        let mut player_hits = Vec::new();
        {
            let p = players.read();
            for (addr, s) in p.iter() {
                if s.gamemode == 0 {
                    let dist_sq = (s.x - x).powi(2) + (s.z - z).powi(2);
                    if dist_sq <= 16.0 {
                        player_hits.push((*addr, s.entity_id, s.health));
                    }
                }
            }
        }

        for (addr, pid, hp) in player_hits {
            let new_hp = hp.saturating_sub(10).max(1);
            {
                let mut p = players.write();
                if let Some(s) = p.get_mut(&addr) {
                    s.health = new_hp;
                }
            }
            let hp_pk = SetHealthPacket::new(new_hp as i32);
            let anim = AnimatePacket::new(1, pid);
            let hurt_snd = LevelSoundEventPacket::new(level_event::SOUND_HIT, x as f32, y as f32, z as f32);
            let batch = BatchPacket::encode(&[hp_pk.encode(), anim.encode(), hurt_snd.encode()]);
            rak.send_encapsulated(addr, batch, true).await;
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
            dead_ticks: 0,
            is_dead: false,
            on_fire_ticks: 0,
            killer_eid: None,
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
            dead_ticks: 0,
            is_dead: false,
            on_fire_ticks: 0,
            killer_eid: None,
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
        let rand_x = (rand::random::<f32>() - 0.5) * 0.15;
        let rand_z = (rand::random::<f32>() - 0.5) * 0.15;

        let ent = ActiveEntity {
            id: entity_id,
            entity_type: 64, // ItemEntity
            name: "Item".to_string(),
            x: x + rand_x as f64,
            y,
            z: z + rand_z as f64,
            motion_x: rand_x,
            motion_y: 0.2,
            motion_z: rand_z,
            yaw: 0.0,
            pitch: 0.0,
            health: 5.0,
            max_health: 5.0,
            fuse: None,
            lifetime: 0,
            owner_eid: None,
            item_drop: Some(item.clone()),
            pickup_delay: 10,
            attack_cooldown: 0,
            dead_ticks: 0,
            is_dead: false,
            on_fire_ticks: 0,
            killer_eid: None,
        };

        self.entities.write().insert(entity_id, ent);

        let mut pk = AddItemEntityPacket::new(entity_id, item, (x + rand_x as f64) as f32, y as f32, (z + rand_z as f64) as f32);
        pk.speed_x = rand_x;
        pk.speed_y = 0.2;
        pk.speed_z = rand_z;
        broadcast_to_all(rak, players, pk.encode()).await;
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
        let mut ent_guard = self.entities.write();
        if let Some(ent) = ent_guard.get_mut(&entity_id) {
            if ent.is_dead {
                return false;
            }
            ent.health -= damage;

            // Knockback calculation
            if let Some((ax, _ay, az)) = attacker_pos {
                let dx = ent.x - ax;
                let dz = ent.z - az;
                let f = (dx * dx + dz * dz).sqrt();
                if f > 0.0 {
                    let inv_f = 1.0 / f;
                    let base = 0.4f32;
                    ent.motion_x /= 2.0;
                    ent.motion_y /= 2.0;
                    ent.motion_z /= 2.0;
                    ent.motion_x += (dx * inv_f) as f32 * base;
                    ent.motion_y += base;
                    ent.motion_z += (dz * inv_f) as f32 * base;
                    if ent.motion_y > base {
                        ent.motion_y = base;
                    }
                }
            }

            if ent.health <= 0.0 {
                ent.is_dead = true;
                ent.dead_ticks = 0;
                ent.killer_eid = Some(attacker_eid);

                let death_anim = EntityEventPacket::new(entity_id, 3); // Death animation
                let death_snd = LevelSoundEventPacket::new(level_event::SOUND_DEATH, ent.x as f32, ent.y as f32, ent.z as f32);
                let mv = MoveEntityPacket::new(ent.id, ent.x as f32, ent.y as f32, ent.z as f32, ent.yaw, ent.pitch);
                drop(ent_guard);

                broadcast_to_all(rak, players, death_anim.encode()).await;
                broadcast_to_all(rak, players, death_snd.encode()).await;
                broadcast_to_all(rak, players, mv.encode()).await;
                return true;
            } else {
                let anim = EntityEventPacket::new(entity_id, 2); // Hurt animation
                let sound = LevelSoundEventPacket::new(level_event::SOUND_HIT, ent.x as f32, ent.y as f32, ent.z as f32);
                let mv = MoveEntityPacket::new(ent.id, ent.x as f32, ent.y as f32, ent.z as f32, ent.yaw, ent.pitch);
                drop(ent_guard);

                broadcast_to_all(rak, players, anim.encode()).await;
                broadcast_to_all(rak, players, sound.encode()).await;
                broadcast_to_all(rak, players, mv.encode()).await;
                return false;
            }
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
        let mut player_attacks: Vec<(SocketAddr, f32, f64, f64)> = Vec::new(); // addr, damage, attacker_x, attacker_z
        let mut skeleton_arrows: Vec<(f64, f64, f64, f32, f32, f32, f32, f32, i64)> = Vec::new(); // x,y,z, vx,vy,vz, yaw,pitch, eid

        // Snapshot current player positions
        let player_snapshots: Vec<(SocketAddr, i64, f64, f64, f64, i32)> = {
            let p = players.read();
            p.iter().map(|(addr, s)| (*addr, s.entity_id, s.x, s.y, s.z, s.gamemode)).collect()
        };

        for ent in &mut list {
            ent.lifetime += 1;

            // Handle dead entities (despawn animation timer)
            if ent.is_dead {
                ent.dead_ticks += 1;
                if ent.dead_ticks >= 20 {
                    to_remove.push(ent.id);
                    // Broadcast remove packet
                    let remove_pk = RemoveEntityPacket { entity_runtime_id: ent.id };
                    broadcast_to_all(rak, players, remove_pk.encode()).await;

                    // Generate loot drops
                    let generated_drops = LootTable::get_drops(
                        ent.entity_type,
                        ent.killer_eid.is_some(),
                        ent.on_fire_ticks > 0,
                        0,
                    );
                    for item in generated_drops {
                        drops.push((item.id, item.count, ent.x, ent.y, ent.z));
                    }
                }
                continue;
            }

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
                for &(p_addr, p_eid, px, py, pz, gm) in &player_snapshots {
                    if Some(p_eid) == ent.owner_eid && ent.lifetime < 5 {
                        continue; // Cannot hit shooter immediately
                    }
                    let dist_sq = (px - ent.x).powi(2) + (py + 1.0 - ent.y).powi(2) + (pz - ent.z).powi(2);
                    if dist_sq < 1.44 {
                        hit_player_opt = Some((p_addr, p_eid, gm));
                        break;
                    }
                }

                if block_hit || hit_player_opt.is_some() || ent.lifetime > 200 {
                    to_remove.push(ent.id);

                    if let Some((p_addr, _p_eid, gm)) = hit_player_opt {
                        if gm == 0 && ent.entity_type == 80 {
                            // Arrow hit player in survival
                            player_attacks.push((p_addr, 3.0, ent.x, ent.z));
                        }
                    }

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

            // 4. Mob AI & Physics
            let is_hostile = ent.entity_type == 32 || ent.entity_type == 33 || ent.entity_type == 34
                || ent.entity_type == 35 || ent.entity_type == 36 || ent.entity_type == 38
                || ent.entity_type == 41 || ent.entity_type == 42 || ent.entity_type == 47;

            let is_passive = ent.entity_type == 10 || ent.entity_type == 11 || ent.entity_type == 12 || ent.entity_type == 13;

            if is_hostile || is_passive {
                // Find nearest player in survival within 36 blocks (willMove check)
                let mut nearest = None;
                let mut min_dist_sq = 36.0 * 36.0;

                for &(p_addr, p_eid, px, py, pz, gm) in &player_snapshots {
                    if gm == 0 { // Survival
                        let d_sq = (px - ent.x).powi(2) + (pz - ent.z).powi(2);
                        if d_sq < min_dist_sq {
                            min_dist_sq = d_sq;
                            nearest = Some((p_addr, p_eid, px, py, pz, d_sq.sqrt()));
                        }
                    }
                }

                if is_hostile {
                    if let Some((target_addr, _target_id, tx, ty, tz, dist)) = nearest {
                        let dx = tx - ent.x;
                        let dz = tz - ent.z;
                        ent.yaw = calculate_yaw(dx, dz);
                        ent.pitch = calculate_pitch(ent.x, ent.y + 1.6, ent.z, tx, ty + 1.0, tz);

                        match ent.entity_type {
                            // Zombie (32) & Husk (47)
                            32 | 47 => {
                                let speed = 0.23f32;
                                ent.motion_x = ((dx / dist) as f32) * speed;
                                ent.motion_z = ((dz / dist) as f32) * speed;

                                // Melee attack within 1.5 blocks
                                if dist <= 1.5 && ent.attack_cooldown == 0 {
                                    ent.attack_cooldown = 20; // 1 second cooldown
                                    player_attacks.push((target_addr, 3.0, ent.x, ent.z));
                                }
                            }

                            // Creeper (33)
                            33 => {
                                if dist > 2.5 {
                                    let speed = 0.22f32;
                                    ent.motion_x = ((dx / dist) as f32) * speed;
                                    ent.motion_z = ((dz / dist) as f32) * speed;
                                }

                                if let Some(ref mut fuse) = ent.fuse {
                                    if dist <= 3.0 {
                                        if *fuse == 30 {
                                            // Play ignite sound
                                            let ignite = LevelSoundEventPacket::new(level_event::SOUND_IGNITE, ent.x as f32, ent.y as f32, ent.z as f32);
                                            broadcast_to_all(rak, players, ignite.encode()).await;
                                        }
                                        if *fuse > 0 {
                                            *fuse -= 1;
                                        } else {
                                            to_remove.push(ent.id);
                                            explosions.push((ent.x, ent.y, ent.z));
                                            continue;
                                        }
                                    } else if dist > 6.0 && *fuse < 30 {
                                        // Player fled - defuse slowly
                                        *fuse += 1;
                                    }
                                }
                            }

                            // Skeleton (34)
                            34 => {
                                if dist > 14.0 {
                                    let speed = 0.20f32;
                                    ent.motion_x = ((dx / dist) as f32) * speed;
                                    ent.motion_z = ((dz / dist) as f32) * speed;
                                } else if dist < 4.0 {
                                    // Back away
                                    let speed = 0.15f32;
                                    ent.motion_x = -((dx / dist) as f32) * speed;
                                    ent.motion_z = -((dz / dist) as f32) * speed;
                                }

                                // Ranged attack with bow every 50 ticks
                                if dist <= 15.0 && dist >= 3.0 && ent.attack_cooldown == 0 {
                                    ent.attack_cooldown = 50;
                                    let shoot_vx = ((dx / dist) * 1.5) as f32;
                                    let shoot_vy = (((ty + 0.5 - ent.y) / dist) * 1.5 + 0.1) as f32;
                                    let shoot_vz = ((dz / dist) * 1.5) as f32;
                                    skeleton_arrows.push((
                                        ent.x, ent.y + 1.6, ent.z,
                                        shoot_vx, shoot_vy, shoot_vz,
                                        ent.yaw, ent.pitch, ent.id,
                                    ));
                                }
                            }

                            // Spider (35)
                            35 => {
                                let speed = 0.28f32;
                                ent.motion_x = ((dx / dist) as f32) * speed;
                                ent.motion_z = ((dz / dist) as f32) * speed;

                                // Leap when within 3 blocks
                                if dist <= 3.0 && dist > 1.2 && ent.y <= floor_level(world, ent.x, ent.y, ent.z) + 0.1 {
                                    ent.motion_y = 0.35;
                                }

                                if dist <= 1.5 && ent.attack_cooldown == 0 {
                                    ent.attack_cooldown = 20;
                                    player_attacks.push((target_addr, 2.0, ent.x, ent.z));
                                }
                            }

                            _ => {
                                let speed = 0.20f32;
                                ent.motion_x = ((dx / dist) as f32) * speed;
                                ent.motion_z = ((dz / dist) as f32) * speed;
                            }
                        }
                    }
                } else if is_passive {
                    // Random wander every ~60 ticks
                    if ent.lifetime % 60 == 0 && rand::random::<f32>() < 0.4 {
                        let angle = rand::random::<f64>() * (std::f64::consts::PI * 2.0);
                        let speed = 0.10f32;
                        ent.motion_x = (angle.cos() as f32) * speed;
                        ent.motion_z = (angle.sin() as f32) * speed;
                        ent.yaw = calculate_yaw(ent.motion_x as f64, ent.motion_z as f64);
                    }
                }

                if ent.attack_cooldown > 0 {
                    ent.attack_cooldown -= 1;
                }
            }

            // 5. Gravity & Floor Collision
            let floor_y = floor_level(world, ent.x, ent.y, ent.z);

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
            ent.motion_x *= 0.85;
            ent.motion_z *= 0.85;
        }

        // Apply player attacks from hostile mobs
        for (target_addr, damage, ax, az) in player_attacks {
            let (target_eid, current_hp) = {
                let p = players.read();
                if let Some(s) = p.get(&target_addr) {
                    (s.entity_id, s.health)
                } else {
                    continue;
                }
            };

            let new_hp = (current_hp as f32 - damage).max(0.0) as u32;
            {
                let mut p = players.write();
                if let Some(s) = p.get_mut(&target_addr) {
                    s.health = new_hp;
                }
            }

            let hp_pk = SetHealthPacket::new(new_hp as i32);
            let batch = BatchPacket::encode(&[hp_pk.encode()]);
            rak.send_encapsulated(target_addr, batch, true).await;

            let anim = AnimatePacket::new(1, target_eid); // Hurt animation
            let hurt_snd = LevelSoundEventPacket::new(level_event::SOUND_HIT, ax as f32, 64.0, az as f32);
            broadcast_to_all(rak, players, anim.encode()).await;
            broadcast_to_all(rak, players, hurt_snd.encode()).await;
        }

        // Spawn skeleton arrows
        for (sx, sy, sz, vx, vy, vz, yaw, pitch, owner_id) in skeleton_arrows {
            let arrow_eid = {
                let mut eid = next_eid.write();
                let id = *eid;
                *eid += 1;
                id
            };
            self.spawn_projectile(arrow_eid, 80, sx, sy, sz, vx, vy, vz, yaw, pitch, owner_id, rak, players).await;
            let shoot_snd = LevelSoundEventPacket::new(level_event::SOUND_BOW, sx as f32, sy as f32, sz as f32);
            broadcast_to_all(rak, players, shoot_snd.encode()).await;
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
            if !to_remove.contains(&ent.id) && !ent.is_dead && (ent.motion_x.abs() > 0.01 || ent.motion_z.abs() > 0.01) {
                let mv = MoveEntityPacket::new(ent.id, ent.x as f32, ent.y as f32, ent.z as f32, ent.yaw, ent.pitch);
                broadcast_to_all(rak, players, mv.encode()).await;
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

fn floor_level(world: &Arc<RwLock<World>>, x: f64, y: f64, z: f64) -> f64 {
    let w = world.read();
    let bx = x.floor() as i32;
    let bz = z.floor() as i32;
    let mut check_y = y.floor() as i32;
    while check_y >= 0 && w.get_block_id_opt(bx, check_y, bz) == 0 {
        check_y -= 1;
    }
    (check_y + 1) as f64
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
