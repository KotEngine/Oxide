use std::net::SocketAddr;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::protocol::packets::{
    AddEntityPacket, RemoveEntityPacket, MoveEntityPacket, MovePlayerPacket,
    LevelEventPacket, LevelSoundEventPacket, SetHealthPacket,
    MobEffectPacket, SetEntityMotionPacket, level_event, move_player,
};
use crate::protocol::packet::DataPacket;
use crate::level::World;
use crate::protocol::handler::PlayerState;
use crate::raknet::RakNetServer;
use crate::entity::manager::EntityManager;
use crate::entity::effect::{effect_id, get_potion_effects};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileKind {
    EnderPearl,
    Snowball,
    Egg,
    Arrow,
    ExpBottle,
    SplashPotion,
}

impl ProjectileKind {
    pub fn network_entity_type(&self) -> u32 {
        match self {
            Self::Arrow => 80,
            Self::Snowball => 81,
            Self::Egg => 82,
            Self::ExpBottle => 68,
            Self::SplashPotion => 86,
            Self::EnderPearl => 87,
        }
    }

    pub fn gravity(&self) -> f64 {
        match self {
            Self::Arrow => 0.05,
            Self::SplashPotion => 0.1,
            Self::ExpBottle => 0.07,
            _ => 0.03,
        }
    }

    pub fn drag(&self) -> f64 {
        match self {
            Self::SplashPotion => 0.05,
            _ => 0.01,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Projectile {
    pub entity_id: i64,
    pub dimension: i32,
    pub kind: ProjectileKind,
    pub owner_addr: SocketAddr,
    pub owner_eid: i64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub motion_x: f64,
    pub motion_y: f64,
    pub motion_z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub age: u32,
    pub damage: f32,
    pub knockback: f32,
    pub fire_seconds: u32,
    pub is_critical: bool,
    pub potion_id: Option<i32>,
}

pub struct ProjectileManager {
    projectiles: RwLock<Vec<Projectile>>,
}

impl Default for ProjectileManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectileManager {
    pub fn new() -> Self {
        Self {
            projectiles: RwLock::new(Vec::new()),
        }
    }

    pub fn launch(
        &self,
        entity_id: i64,
        kind: ProjectileKind,
        owner_addr: SocketAddr,
        owner_eid: i64,
        pos_x: f64,
        pos_y: f64,
        pos_z: f64,
        yaw: f32,
        pitch: f32,
        force: f64,
        rak: &Arc<RakNetServer>,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    ) {
        let dimension = players.read().get(&owner_addr).map(|s| s.dimension).unwrap_or(0);
        let default_dmg = if matches!(kind, ProjectileKind::Arrow) { 2.0 } else { 0.0 };
        self.launch_configured(
            entity_id,
            dimension,
            kind,
            owner_addr,
            owner_eid,
            pos_x,
            pos_y,
            pos_z,
            yaw,
            pitch,
            force,
            default_dmg,
            0.0,
            0,
            false,
            None,
            rak,
            players,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn launch_configured(
        &self,
        entity_id: i64,
        dimension: i32,
        kind: ProjectileKind,
        owner_addr: SocketAddr,
        owner_eid: i64,
        pos_x: f64,
        pos_y: f64,
        pos_z: f64,
        yaw: f32,
        pitch: f32,
        force: f64,
        damage: f32,
        knockback: f32,
        fire_seconds: u32,
        is_critical: bool,
        potion_id: Option<i32>,
        rak: &Arc<RakNetServer>,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    ) {
        let yaw_rad = (yaw as f64).to_radians();
        let pitch_rad = (pitch as f64).to_radians();
        let motion_x = -yaw_rad.sin() * pitch_rad.cos() * force;
        let motion_y = -pitch_rad.sin() * force;
        let motion_z = yaw_rad.cos() * pitch_rad.cos() * force;

        // Spawn position at eye level
        let spawn_x = pos_x;
        let spawn_y = pos_y + 1.5;
        let spawn_z = pos_z;

        let mut add_entity = AddEntityPacket::new(
            entity_id,
            kind.network_entity_type(),
            spawn_x as f32,
            spawn_y as f32,
            spawn_z as f32,
        );
        add_entity.speed_x = motion_x as f32;
        add_entity.speed_y = motion_y as f32;
        add_entity.speed_z = motion_z as f32;
        add_entity.yaw = yaw;
        add_entity.pitch = pitch;

        let rak_clone = rak.clone();
        let players_clone = players.clone();
        let add_bytes = add_entity.encode();
        tokio::spawn(async move {
            broadcast_packet_dim(&rak_clone, &players_clone, dimension, add_bytes, true).await;
        });

        let sound_type = match kind {
            ProjectileKind::Arrow => level_event::SOUND_BOW,
            _ => level_event::SOUND_THROW,
        };
        let sound_pk = LevelSoundEventPacket::new(sound_type, spawn_x as f32, spawn_y as f32, spawn_z as f32);
        let rak_clone2 = rak.clone();
        let players_clone2 = players.clone();
        let sound_bytes = sound_pk.encode();
        tokio::spawn(async move {
            broadcast_packet_dim(&rak_clone2, &players_clone2, dimension, sound_bytes, true).await;
        });

        let proj = Projectile {
            entity_id,
            dimension,
            kind,
            owner_addr,
            owner_eid,
            x: spawn_x,
            y: spawn_y,
            z: spawn_z,
            motion_x,
            motion_y,
            motion_z,
            yaw,
            pitch,
            age: 0,
            damage,
            knockback,
            fire_seconds,
            is_critical,
            potion_id,
        };

        self.projectiles.write().push(proj);
    }

    pub async fn tick(
        &self,
        world: &Arc<RwLock<World>>,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
        rak: &RakNetServer,
        next_eid: &Arc<RwLock<i64>>,
    ) {
        self.tick_internal(&[world, world, world], None, players, rak, next_eid).await;
    }

    pub async fn tick_all(
        &self,
        worlds: &[&Arc<RwLock<World>>; 3],
        entity_manager: &Arc<EntityManager>,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
        rak: &RakNetServer,
        next_eid: &Arc<RwLock<i64>>,
    ) {
        self.tick_internal(worlds, Some(entity_manager), players, rak, next_eid).await;
    }

    async fn tick_internal(
        &self,
        worlds: &[&Arc<RwLock<World>>; 3],
        entity_manager: Option<&Arc<EntityManager>>,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
        rak: &RakNetServer,
        next_eid: &Arc<RwLock<i64>>,
    ) {
        let mut projs = std::mem::take(&mut *self.projectiles.write());
        let mut surviving = Vec::new();
        let mut move_batches: HashMap<i32, Vec<Vec<u8>>> = HashMap::new();

        for mut proj in projs.drain(..) {
            let dim = proj.dimension.clamp(0, 2);
            let world = worlds[dim as usize];
            proj.age += 1;
            if proj.age > 300 {
                let remove = RemoveEntityPacket::new(proj.entity_id);
                broadcast_packet_dim(rak, players, dim, remove.encode(), true).await;
                continue;
            }

            // Apply gravity and drag
            proj.motion_y -= proj.kind.gravity();
            proj.motion_x *= 1.0 - proj.kind.drag();
            proj.motion_y *= 1.0 - proj.kind.drag();
            proj.motion_z *= 1.0 - proj.kind.drag();

            let next_x = proj.x + proj.motion_x;
            let next_y = proj.y + proj.motion_y;
            let next_z = proj.z + proj.motion_z;

            let hit_bx = next_x.floor() as i32;
            let hit_by = next_y.floor() as i32;
            let hit_bz = next_z.floor() as i32;

            let block_id = if (0..256).contains(&hit_by) {
                let w = world.read();
                w.get_block_id_opt(hit_bx, hit_by, hit_bz)
            } else {
                0
            };

            let is_solid = is_collidable_block(block_id);
            let below_void = next_y < -10.0;

            if is_solid || below_void {
                self.handle_hit(proj, next_x, next_y, next_z, world, entity_manager, players, rak, next_eid).await;
                continue;
            }

            // Check collision with players in same dimension
            let mut player_hit = None;
            {
                let guard = players.read();
                for (addr, state) in guard.iter() {
                    if state.dimension != dim || !state.spawned {
                        continue;
                    }
                    if state.entity_id == proj.owner_eid && proj.age < 5 {
                        continue;
                    }
                    let dx = state.x - next_x;
                    let dy = (state.y + 0.9) - next_y;
                    let dz = state.z - next_z;
                    if dx * dx + dy * dy + dz * dz < 1.0 {
                        player_hit = Some((*addr, state.entity_id));
                        break;
                    }
                }
            }

            if let Some((hit_addr, hit_eid)) = player_hit {
                self.handle_player_hit(proj, hit_addr, hit_eid, next_x, next_y, next_z, entity_manager, players, rak).await;
                continue;
            }

            // Check collision with mobs/entities in same dimension
            let mob_hit = entity_manager.and_then(|em| {
                em.find_hit_entity(
                    dim,
                    next_x,
                    next_y,
                    next_z,
                    1.0,
                    if proj.age < 5 { Some(proj.owner_eid) } else { None },
                )
            });

            if let Some(target_eid) = mob_hit {
                self.handle_mob_hit(proj, target_eid, next_x, next_y, next_z, entity_manager, players, rak, next_eid).await;
                continue;
            }

            proj.x = next_x;
            proj.y = next_y;
            proj.z = next_z;

            let move_pk = MoveEntityPacket::new(
                proj.entity_id,
                proj.x as f32,
                proj.y as f32,
                proj.z as f32,
                proj.yaw,
                proj.pitch,
            );
            move_batches.entry(dim).or_default().push(move_pk.encode());
            surviving.push(proj);
        }

        for (dim, packets) in move_batches {
            if packets.is_empty() {
                continue;
            }
            let recipients: Vec<SocketAddr> = {
                let guard = players.read();
                guard.iter()
                    .filter(|(_, s)| s.dimension == dim && s.spawned)
                    .map(|(addr, _)| *addr)
                    .collect()
            };
            if !recipients.is_empty() {
                let batch = crate::protocol::batch::BatchPacket::encode(&packets);
                for addr in recipients {
                    rak.send_encapsulated(addr, batch.clone(), false).await;
                }
            }
        }

        *self.projectiles.write() = surviving;
    }

    #[allow(clippy::too_many_arguments)]
    async fn handle_hit(
        &self,
        proj: Projectile,
        hit_x: f64,
        hit_y: f64,
        hit_z: f64,
        _world: &Arc<RwLock<World>>,
        entity_manager: Option<&Arc<EntityManager>>,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
        rak: &RakNetServer,
        next_eid: &Arc<RwLock<i64>>,
    ) {
        let dim = proj.dimension;
        let remove = RemoveEntityPacket::new(proj.entity_id);
        broadcast_packet_dim(rak, players, dim, remove.encode(), true).await;

        match proj.kind {
            ProjectileKind::EnderPearl => {
                let part_pk = LevelEventPacket::new(
                    level_event::EVENT_PARTICLE_ENDERMAN_TELEPORT,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                    0,
                );
                broadcast_packet_dim(rak, players, dim, part_pk.encode(), true).await;

                let sound_pk = LevelSoundEventPacket::new(
                    level_event::SOUND_TELEPORT,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                );
                broadcast_packet_dim(rak, players, dim, sound_pk.encode(), true).await;

                let (owner_name, yaw, pitch, is_survival) = {
                    let mut guard = players.write();
                    if let Some(state) = guard.get_mut(&proj.owner_addr) {
                        state.x = hit_x;
                        state.y = hit_y;
                        state.z = hit_z;
                        (state.username.clone(), state.yaw, state.pitch, state.gamemode == 0)
                    } else {
                        (String::new(), 0.0, 0.0, false)
                    }
                };

                let mut tp_pk = MovePlayerPacket::new(
                    proj.owner_eid,
                    hit_x as f32,
                    (hit_y + 1.62) as f32,
                    hit_z as f32,
                    yaw,
                    pitch,
                );
                tp_pk.head_yaw = yaw;
                tp_pk.mode = move_player::MODE_RESET;
                send_packet(rak, proj.owner_addr, tp_pk.encode()).await;

                if is_survival {
                    let msg = format!("{} hit the ground too hard", owner_name);
                    crate::entity::damage::hurt_player(
                        players,
                        rak,
                        proj.owner_addr,
                        5.0,
                        crate::entity::damage::DamageCause::Fall,
                        false,
                        None,
                        None,
                        Some(&msg),
                    )
                    .await;
                }
            }
            ProjectileKind::Snowball => {
                let part = LevelEventPacket::new(
                    level_event::EVENT_PARTICLE_SPLASH,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                    0,
                );
                broadcast_packet_dim(rak, players, dim, part.encode(), true).await;

                let sound = LevelSoundEventPacket::new(
                    level_event::SOUND_POP,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                );
                broadcast_packet_dim(rak, players, dim, sound.encode(), true).await;
            }
            ProjectileKind::Egg => {
                let part = LevelEventPacket::new(
                    level_event::EVENT_PARTICLE_SPLASH,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                    0,
                );
                broadcast_packet_dim(rak, players, dim, part.encode(), true).await;

                let sound = LevelSoundEventPacket::new(
                    level_event::SOUND_POP,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                );
                broadcast_packet_dim(rak, players, dim, sound.encode(), true).await;

                use rand::Rng;
                let spawn_chicken = rand::thread_rng().gen_range(0..8) == 0;
                if spawn_chicken {
                    let chicken_id = {
                        let mut eid_guard = next_eid.write();
                        let id = *eid_guard;
                        *eid_guard += 1;
                        id
                    };
                    if let Some(em) = entity_manager {
                        em.spawn_mob_in_dim(chicken_id, dim, 10, "Chicken".to_string(), hit_x, hit_y, hit_z, 4.0, rak, players).await;
                    } else {
                        let chicken = AddEntityPacket::new(chicken_id, 10, hit_x as f32, hit_y as f32, hit_z as f32);
                        broadcast_packet_dim(rak, players, dim, chicken.encode(), true).await;
                    }
                }
            }
            ProjectileKind::Arrow => {
                let sound = LevelSoundEventPacket::new(
                    level_event::SOUND_BOW_HIT,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                );
                broadcast_packet_dim(rak, players, dim, sound.encode(), true).await;
            }
            ProjectileKind::ExpBottle => {
                let sound = LevelSoundEventPacket::new(
                    level_event::SOUND_GLASS,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                );
                broadcast_packet_dim(rak, players, dim, sound.encode(), true).await;

                let part = LevelEventPacket::new(
                    level_event::EVENT_PARTICLE_SPLASH,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                    0,
                );
                broadcast_packet_dim(rak, players, dim, part.encode(), true).await;
            }
            ProjectileKind::SplashPotion => {
                apply_splash_potion(
                    dim,
                    proj.potion_id.unwrap_or(0),
                    hit_x,
                    hit_y,
                    hit_z,
                    proj.owner_eid,
                    entity_manager,
                    players,
                    rak,
                ).await;
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn handle_player_hit(
        &self,
        proj: Projectile,
        hit_addr: SocketAddr,
        hit_eid: i64,
        hit_x: f64,
        hit_y: f64,
        hit_z: f64,
        entity_manager: Option<&Arc<EntityManager>>,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
        rak: &RakNetServer,
    ) {
        let dim = proj.dimension;
        let remove = RemoveEntityPacket::new(proj.entity_id);
        broadcast_packet_dim(rak, players, dim, remove.encode(), true).await;

        match proj.kind {
            ProjectileKind::EnderPearl => {
                let (owner_name, yaw, pitch, is_survival) = {
                    let mut guard = players.write();
                    if let Some(state) = guard.get_mut(&proj.owner_addr) {
                        state.x = hit_x;
                        state.y = hit_y;
                        state.z = hit_z;
                        (state.username.clone(), state.yaw, state.pitch, state.gamemode == 0)
                    } else {
                        (String::new(), 0.0, 0.0, false)
                    }
                };

                let mut tp_pk = MovePlayerPacket::new(
                    proj.owner_eid,
                    hit_x as f32,
                    (hit_y + 1.62) as f32,
                    hit_z as f32,
                    yaw,
                    pitch,
                );
                tp_pk.head_yaw = yaw;
                tp_pk.mode = move_player::MODE_RESET;
                send_packet(rak, proj.owner_addr, tp_pk.encode()).await;

                let tp_sound = LevelSoundEventPacket::new(
                    level_event::SOUND_TELEPORT,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                );
                broadcast_packet_dim(rak, players, dim, tp_sound.encode(), true).await;

                if is_survival {
                    let msg = format!("{} hit the ground too hard", owner_name);
                    crate::entity::damage::hurt_player(
                        players,
                        rak,
                        proj.owner_addr,
                        5.0,
                        crate::entity::damage::DamageCause::Fall,
                        false,
                        None,
                        None,
                        Some(&msg),
                    )
                    .await;
                }
            }
            ProjectileKind::Arrow => {
                let dmg = arrow_damage(&proj) as f32;
                let (target_name, is_survival) = {
                    let mut guard = players.write();
                    if let Some(state) = guard.get_mut(&hit_addr) {
                        let surv = state.gamemode == 0;
                        if surv && proj.fire_seconds > 0 {
                            state.fire_ticks = state.fire_ticks.max((proj.fire_seconds * 20) as i32);
                        }
                        (state.username.clone(), surv)
                    } else {
                        (String::new(), false)
                    }
                };

                if is_survival {
                    let msg = format!("{} was shot", target_name);
                    let outcome = crate::entity::damage::hurt_player(
                        players,
                        rak,
                        hit_addr,
                        dmg,
                        crate::entity::damage::DamageCause::Projectile,
                        proj.is_critical,
                        Some((proj.x, proj.y, proj.z)),
                        None,
                        Some(&msg),
                    )
                    .await;
                    if proj.knockback > 0.0 && !outcome.died && outcome.final_damage > 0.0 {
                        let mot_mag = (proj.motion_x * proj.motion_x + proj.motion_z * proj.motion_z).sqrt().max(0.01);
                        let p_mot = SetEntityMotionPacket::new(
                            hit_eid,
                            (proj.motion_x / mot_mag) as f32 * (0.4 * proj.knockback),
                            0.25,
                            (proj.motion_z / mot_mag) as f32 * (0.4 * proj.knockback),
                        );
                        broadcast_packet_dim(rak, players, dim, p_mot.encode(), true).await;
                    }
                }

                let bow_hit = LevelSoundEventPacket::new(
                    level_event::SOUND_BOW_HIT,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                );
                broadcast_packet_dim(rak, players, dim, bow_hit.encode(), true).await;
            }
            ProjectileKind::Snowball | ProjectileKind::Egg => {
                let pop_sound = LevelSoundEventPacket::new(
                    level_event::SOUND_POP,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                );
                broadcast_packet_dim(rak, players, dim, pop_sound.encode(), true).await;

                let part = LevelEventPacket::new(
                    level_event::EVENT_PARTICLE_SPLASH,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                    0,
                );
                broadcast_packet_dim(rak, players, dim, part.encode(), true).await;
            }
            ProjectileKind::ExpBottle => {
                let glass_sound = LevelSoundEventPacket::new(
                    level_event::SOUND_GLASS,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                );
                broadcast_packet_dim(rak, players, dim, glass_sound.encode(), true).await;
            }
            ProjectileKind::SplashPotion => {
                apply_splash_potion(
                    dim,
                    proj.potion_id.unwrap_or(0),
                    hit_x,
                    hit_y,
                    hit_z,
                    proj.owner_eid,
                    entity_manager,
                    players,
                    rak,
                ).await;
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn handle_mob_hit(
        &self,
        proj: Projectile,
        target_eid: i64,
        hit_x: f64,
        hit_y: f64,
        hit_z: f64,
        entity_manager: Option<&Arc<EntityManager>>,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
        rak: &RakNetServer,
        next_eid: &Arc<RwLock<i64>>,
    ) {
        let dim = proj.dimension;
        let remove = RemoveEntityPacket::new(proj.entity_id);
        broadcast_packet_dim(rak, players, dim, remove.encode(), true).await;

        match proj.kind {
            ProjectileKind::Arrow => {
                if let Some(em) = entity_manager {
                    let dmg = arrow_damage(&proj) as f32;
                    em.damage_entity_ext(
                        target_eid,
                        dmg,
                        proj.owner_eid,
                        Some((proj.x, proj.y, proj.z)),
                        proj.fire_seconds,
                        proj.knockback,
                        rak,
                        players,
                    ).await;
                }
                let bow_hit = LevelSoundEventPacket::new(
                    level_event::SOUND_BOW_HIT,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                );
                broadcast_packet_dim(rak, players, dim, bow_hit.encode(), true).await;
            }
            ProjectileKind::Snowball | ProjectileKind::Egg => {
                if let Some(em) = entity_manager {
                    em.damage_entity_ext(
                        target_eid,
                        0.0,
                        proj.owner_eid,
                        Some((proj.x, proj.y, proj.z)),
                        0,
                        0.0,
                        rak,
                        players,
                    ).await;
                }
                let pop_sound = LevelSoundEventPacket::new(
                    level_event::SOUND_POP,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                );
                broadcast_packet_dim(rak, players, dim, pop_sound.encode(), true).await;

                let part = LevelEventPacket::new(
                    level_event::EVENT_PARTICLE_SPLASH,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                    0,
                );
                broadcast_packet_dim(rak, players, dim, part.encode(), true).await;
                let _ = next_eid;
            }
            ProjectileKind::SplashPotion => {
                apply_splash_potion(
                    dim,
                    proj.potion_id.unwrap_or(0),
                    hit_x,
                    hit_y,
                    hit_z,
                    proj.owner_eid,
                    entity_manager,
                    players,
                    rak,
                ).await;
            }
            ProjectileKind::ExpBottle => {
                let glass_sound = LevelSoundEventPacket::new(
                    level_event::SOUND_GLASS,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                );
                broadcast_packet_dim(rak, players, dim, glass_sound.encode(), true).await;
            }
            ProjectileKind::EnderPearl => {
                let _ = entity_manager;
                let _ = next_eid;
            }
        }
    }
}

fn arrow_damage(proj: &Projectile) -> u32 {
    let mag = (proj.motion_x.powi(2) + proj.motion_y.powi(2) + proj.motion_z.powi(2)).sqrt() as f32;
    let mut dmg = (mag * proj.damage).ceil() as u32;
    if proj.is_critical && dmg > 0 {
        use rand::Rng;
        let bonus = rand::thread_rng().gen_range(0..=(dmg / 2 + 1));
        dmg += bonus;
    }
    dmg.max(1)
}

#[allow(clippy::too_many_arguments)]
async fn apply_splash_potion(
    dim: i32,
    potion_id: i32,
    hit_x: f64,
    hit_y: f64,
    hit_z: f64,
    owner_eid: i64,
    entity_manager: Option<&Arc<EntityManager>>,
    players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    rak: &RakNetServer,
) {
    let glass_sound = LevelSoundEventPacket::new(
        level_event::SOUND_GLASS,
        hit_x as f32,
        hit_y as f32,
        hit_z as f32,
    );
    broadcast_packet_dim(rak, players, dim, glass_sound.encode(), true).await;

    let part = LevelEventPacket::new(
        level_event::EVENT_PARTICLE_SPLASH,
        hit_x as f32,
        hit_y as f32,
        hit_z as f32,
        potion_id,
    );
    broadcast_packet_dim(rak, players, dim, part.encode(), true).await;

    let effects = get_potion_effects(potion_id);
    if effects.is_empty() {
        return;
    }

    let mut player_updates = Vec::new();
    let mut magic_hits = Vec::new();
    {
        let mut guard = players.write();
        for (addr, state) in guard.iter_mut() {
            if state.dimension != dim || !state.spawned {
                continue;
            }
            let dist_sq = (state.x - hit_x).powi(2) + (state.y + 0.9 - hit_y).powi(2) + (state.z - hit_z).powi(2);
            if dist_sq > 16.0 {
                continue;
            }
            let factor = (1.0 - dist_sq.sqrt() / 4.0).clamp(0.0, 1.0);
            let mut hp_changed = false;
            let mut sent_effects = Vec::new();
            for eff in &effects {
                if eff.is_instant() {
                    if eff.id == effect_id::INSTANT_HEALTH {
                        let base = (4 << eff.amplifier) as f64;
                        let heal = (base * factor + 0.5) as u32;
                        state.health = (state.health + heal).min(20);
                        hp_changed = true;
                    } else if eff.id == effect_id::INSTANT_DAMAGE && state.gamemode == 0 {
                        let base = (6 << eff.amplifier) as f64;
                        let dmg = (base * factor) as f32;
                        magic_hits.push((*addr, dmg, state.username.clone()));
                    }
                } else {
                    let scaled_dur = ((eff.duration as f64) * 0.75 * factor) as i32;
                    if scaled_dur > 20 {
                        let mut scaled = eff.clone();
                        scaled.duration = scaled_dur;
                        state.effects.add_effect(scaled.clone());
                        sent_effects.push(scaled);
                    }
                }
            }
            if hp_changed || !sent_effects.is_empty() {
                player_updates.push((*addr, state.entity_id, state.health, hp_changed, sent_effects));
            }
        }
    }

    for (addr, dmg, uname) in magic_hits {
        let msg = format!("{} was killed by magic", uname);
        crate::entity::damage::hurt_player(
            players,
            rak,
            addr,
            dmg,
            crate::entity::damage::DamageCause::Magic,
            false,
            None,
            None,
            Some(&msg),
        )
        .await;
    }

    for (addr, eid, hp, hp_changed, sent_effects) in player_updates {
        if hp_changed {
            let hp_pk = SetHealthPacket::new(hp as i32);
            send_packet(rak, addr, hp_pk.encode()).await;
        }
        for eff in sent_effects {
            let eff_pk = MobEffectPacket {
                entity_runtime_id: eid,
                event_id: 1,
                effect_id: eff.id as i32,
                amplifier: eff.amplifier as i32,
                particles: eff.show_particles,
                duration: eff.duration,
            };
            send_packet(rak, addr, eff_pk.encode()).await;
        }
    }

    if let Some(em) = entity_manager {
        let targets = em.get_entities_in_radius(dim, hit_x, hit_y, hit_z, 4.0);
        for eid in targets {
            for eff in &effects {
                if eff.id == effect_id::INSTANT_DAMAGE || eff.id == effect_id::POISON {
                    let dmg = if eff.id == effect_id::INSTANT_DAMAGE {
                        (6 << eff.amplifier) as f32 * 0.75
                    } else {
                        2.0
                    };
                    em.damage_entity_ext(eid, dmg, owner_eid, Some((hit_x, hit_y, hit_z)), 0, 0.0, rak, players).await;
                }
            }
        }
    }
}

fn is_collidable_block(block_id: u8) -> bool {
    match block_id {
        0 | 8 | 9 | 10 | 11 | 31 | 37 | 38 | 51 | 78 => false,
        _ => true,
    }
}

async fn send_packet(rak: &RakNetServer, addr: SocketAddr, data: Vec<u8>) {
    let batch = crate::protocol::batch::BatchPacket::encode(&[data]);
    rak.send_encapsulated(addr, batch, true).await;
}

async fn broadcast_packet_dim(
    rak: &RakNetServer,
    players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    dimension: i32,
    data: Vec<u8>,
    reliable: bool,
) {
    let recipients: Vec<SocketAddr> = {
        let guard = players.read();
        guard.iter()
            .filter(|(_, s)| s.dimension == dimension && s.spawned)
            .map(|(addr, _)| *addr)
            .collect()
    };
    if recipients.is_empty() {
        return;
    }
    let batch = crate::protocol::batch::BatchPacket::encode(&[data]);
    for addr in recipients {
        rak.send_encapsulated(addr, batch.clone(), reliable).await;
    }
}

#[allow(dead_code)]
async fn broadcast_packet(
    rak: &RakNetServer,
    players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    data: Vec<u8>,
) {
    let recipients: Vec<SocketAddr> = {
        let guard = players.read();
        guard.keys().copied().collect()
    };
    let batch = crate::protocol::batch::BatchPacket::encode(&[data]);
    for addr in recipients {
        rak.send_encapsulated(addr, batch.clone(), true).await;
    }
}
