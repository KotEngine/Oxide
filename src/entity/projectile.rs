use std::net::SocketAddr;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::protocol::packets::{
    AddEntityPacket, RemoveEntityPacket, MoveEntityPacket, MovePlayerPacket,
    LevelEventPacket, LevelSoundEventPacket, SetHealthPacket, AnimatePacket,
    level_event, move_player,
};
use crate::protocol::packet::DataPacket;
use crate::level::World;
use crate::protocol::handler::PlayerState;
use crate::raknet::RakNetServer;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileKind {
    EnderPearl,
    Snowball,
    Egg,
    Arrow,
    ExpBottle,
}

impl ProjectileKind {
    pub fn network_entity_type(&self) -> u32 {
        match self {
            Self::Arrow => 80,
            Self::Snowball => 81,
            Self::Egg => 82,
            Self::ExpBottle => 68,
            Self::EnderPearl => 87,
        }
    }

    pub fn gravity(&self) -> f64 {
        match self {
            Self::Arrow => 0.05,
            _ => 0.03,
        }
    }

    pub fn drag(&self) -> f64 {
        0.01
    }
}

pub struct Projectile {
    pub entity_id: i64,
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
            broadcast_packet(&rak_clone, &players_clone, add_bytes).await;
        });

        // Broadcast throw sound
        let sound_type = match kind {
            ProjectileKind::Arrow => level_event::SOUND_BOW,
            _ => level_event::SOUND_THROW,
        };
        let sound_pk = LevelSoundEventPacket::new(sound_type, spawn_x as f32, spawn_y as f32, spawn_z as f32);
        let rak_clone2 = rak.clone();
        let players_clone2 = players.clone();
        let sound_bytes = sound_pk.encode();
        tokio::spawn(async move {
            broadcast_packet(&rak_clone2, &players_clone2, sound_bytes).await;
        });

        let proj = Projectile {
            entity_id,
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
        let mut projs = std::mem::take(&mut *self.projectiles.write());
        let mut surviving = Vec::new();

        for mut proj in projs.drain(..) {
            proj.age += 1;
            if proj.age > 300 {
                // Despawn projectile after 15 seconds
                let remove = RemoveEntityPacket::new(proj.entity_id);
                broadcast_packet(rak, players, remove.encode()).await;
                continue;
            }

            // Apply gravity and drag
            proj.motion_y -= proj.kind.gravity();
            proj.motion_x *= 1.0 - proj.kind.drag();
            proj.motion_y *= 1.0 - proj.kind.drag();
            proj.motion_z *= 1.0 - proj.kind.drag();

            // Next step position
            let next_x = proj.x + proj.motion_x;
            let next_y = proj.y + proj.motion_y;
            let next_z = proj.z + proj.motion_z;

            // Check block collision
            let hit_bx = next_x.floor() as i32;
            let hit_by = next_y.floor() as i32;
            let hit_bz = next_z.floor() as i32;

            let block_id = if hit_by >= 0 && hit_by < 256 {
                let mut w = world.write();
                w.get_block_id(hit_bx, hit_by, hit_bz)
            } else {
                0
            };

            let is_solid = is_collidable_block(block_id);
            let below_void = next_y < -10.0;

            if is_solid || below_void {
                // Collided with block
                self.handle_hit(proj, next_x, next_y, next_z, world, players, rak, next_eid).await;
            } else {
                // Check collision with other players
                let mut player_hit = None;
                {
                    let guard = players.read();
                    for (addr, state) in guard.iter() {
                        if state.entity_id == proj.owner_eid && proj.age < 5 {
                            // Don't collide with thrower in the first 5 ticks
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
                    self.handle_player_hit(proj, hit_addr, hit_eid, next_x, next_y, next_z, players, rak).await;
                } else {
                    // Update projectile position
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
                    broadcast_packet(rak, players, move_pk.encode()).await;
                    surviving.push(proj);
                }
            }
        }

        *self.projectiles.write() = surviving;
    }

    async fn handle_hit(
        &self,
        proj: Projectile,
        hit_x: f64,
        hit_y: f64,
        hit_z: f64,
        _world: &Arc<RwLock<World>>,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
        rak: &RakNetServer,
        next_eid: &Arc<RwLock<i64>>,
    ) {
        // Despawn projectile entity
        let remove = RemoveEntityPacket::new(proj.entity_id);
        broadcast_packet(rak, players, remove.encode()).await;

        match proj.kind {
            ProjectileKind::EnderPearl => {
                // Particle and sound at landing spot
                let part_pk = LevelEventPacket::new(
                    level_event::EVENT_PARTICLE_ENDERMAN_TELEPORT,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                    0,
                );
                broadcast_packet(rak, players, part_pk.encode()).await;

                let sound_pk = LevelSoundEventPacket::new(
                    level_event::SOUND_TELEPORT,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                );
                broadcast_packet(rak, players, sound_pk.encode()).await;

                // Teleport the thrower to impact location
                let mut current_health = 20u32;
                let mut yaw = 0.0f32;
                let mut pitch = 0.0f32;
                let mut is_survival = true;

                {
                    let mut guard = players.write();
                    if let Some(state) = guard.get_mut(&proj.owner_addr) {
                        state.x = hit_x;
                        state.y = hit_y;
                        state.z = hit_z;
                        yaw = state.yaw;
                        pitch = state.pitch;
                        is_survival = state.gamemode == 0;

                        if is_survival {
                            state.health = state.health.saturating_sub(5);
                            current_health = state.health;
                        } else {
                            current_health = state.health;
                        }
                    }
                }

                // Send teleport packet to player
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

                // Broadcast teleport sound at target position
                let tp_sound = LevelSoundEventPacket::new(
                    level_event::SOUND_TELEPORT,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                );
                broadcast_packet(rak, players, tp_sound.encode()).await;

                if is_survival {
                    // Send health update to player
                    let hp_pk = SetHealthPacket::new(current_health as i32);
                    send_packet(rak, proj.owner_addr, hp_pk.encode()).await;

                    // Hurt sound & animation
                    let hurt_sound = LevelSoundEventPacket::new(
                        level_event::SOUND_HURT,
                        hit_x as f32,
                        hit_y as f32,
                        hit_z as f32,
                    );
                    broadcast_packet(rak, players, hurt_sound.encode()).await;

                    let anim = AnimatePacket::new(1, proj.owner_eid);
                    broadcast_packet(rak, players, anim.encode()).await;
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
                broadcast_packet(rak, players, part.encode()).await;

                let sound = LevelSoundEventPacket::new(
                    level_event::SOUND_POP,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                );
                broadcast_packet(rak, players, sound.encode()).await;
            }
            ProjectileKind::Egg => {
                let part = LevelEventPacket::new(
                    level_event::EVENT_PARTICLE_SPLASH,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                    0,
                );
                broadcast_packet(rak, players, part.encode()).await;

                let sound = LevelSoundEventPacket::new(
                    level_event::SOUND_POP,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                );
                broadcast_packet(rak, players, sound.encode()).await;

                // 1 in 8 chance to spawn a baby chicken
                use rand::Rng;
                if rand::thread_rng().gen_range(0..8) == 0 {
                    let chicken_id = {
                        let mut eid_guard = next_eid.write();
                        let id = *eid_guard;
                        *eid_guard += 1;
                        id
                    };
                    let chicken = AddEntityPacket::new(
                        chicken_id,
                        10, // Chicken entity ID
                        hit_x as f32,
                        hit_y as f32,
                        hit_z as f32,
                    );
                    broadcast_packet(rak, players, chicken.encode()).await;
                }
            }
            ProjectileKind::Arrow => {
                let sound = LevelSoundEventPacket::new(
                    level_event::SOUND_BOW_HIT,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                );
                broadcast_packet(rak, players, sound.encode()).await;
            }
            ProjectileKind::ExpBottle => {
                let sound = LevelSoundEventPacket::new(
                    level_event::SOUND_GLASS,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                );
                broadcast_packet(rak, players, sound.encode()).await;

                let part = LevelEventPacket::new(
                    level_event::EVENT_PARTICLE_SPLASH,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                    0,
                );
                broadcast_packet(rak, players, part.encode()).await;
            }
        }
    }

    async fn handle_player_hit(
        &self,
        proj: Projectile,
        hit_addr: SocketAddr,
        hit_eid: i64,
        hit_x: f64,
        hit_y: f64,
        hit_z: f64,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
        rak: &RakNetServer,
    ) {
        let remove = RemoveEntityPacket::new(proj.entity_id);
        broadcast_packet(rak, players, remove.encode()).await;

        match proj.kind {
            ProjectileKind::EnderPearl => {
                let mut current_health = 20u32;
                let mut yaw = 0.0f32;
                let mut pitch = 0.0f32;
                let mut is_survival = true;

                {
                    let mut guard = players.write();
                    if let Some(state) = guard.get_mut(&proj.owner_addr) {
                        state.x = hit_x;
                        state.y = hit_y;
                        state.z = hit_z;
                        yaw = state.yaw;
                        pitch = state.pitch;
                        is_survival = state.gamemode == 0;
                        if is_survival {
                            state.health = state.health.saturating_sub(5);
                            current_health = state.health;
                        } else {
                            current_health = state.health;
                        }
                    }
                }

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
                broadcast_packet(rak, players, tp_sound.encode()).await;

                if is_survival {
                    let hp_pk = SetHealthPacket::new(current_health as i32);
                    send_packet(rak, proj.owner_addr, hp_pk.encode()).await;

                    let hurt_sound = LevelSoundEventPacket::new(
                        level_event::SOUND_HURT,
                        hit_x as f32,
                        hit_y as f32,
                        hit_z as f32,
                    );
                    broadcast_packet(rak, players, hurt_sound.encode()).await;
                }
            }
            ProjectileKind::Arrow => {
                let mut new_hp = 20u32;
                let is_survival = {
                    let mut guard = players.write();
                    if let Some(state) = guard.get_mut(&hit_addr) {
                        let surv = state.gamemode == 0;
                        if surv {
                            state.health = state.health.saturating_sub(6);
                            new_hp = state.health;
                        }
                        surv
                    } else {
                        false
                    }
                };

                if is_survival {
                    let hp_pk = SetHealthPacket::new(new_hp as i32);
                    send_packet(rak, hit_addr, hp_pk.encode()).await;

                    let hurt_sound = LevelSoundEventPacket::new(
                        level_event::SOUND_HURT,
                        hit_x as f32,
                        hit_y as f32,
                        hit_z as f32,
                    );
                    broadcast_packet(rak, players, hurt_sound.encode()).await;

                    let anim = AnimatePacket::new(1, hit_eid);
                    broadcast_packet(rak, players, anim.encode()).await;
                }

                let bow_hit = LevelSoundEventPacket::new(
                    level_event::SOUND_BOW_HIT,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                );
                broadcast_packet(rak, players, bow_hit.encode()).await;
            }
            ProjectileKind::Snowball | ProjectileKind::Egg => {
                let pop_sound = LevelSoundEventPacket::new(
                    level_event::SOUND_POP,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                );
                broadcast_packet(rak, players, pop_sound.encode()).await;

                let part = LevelEventPacket::new(
                    level_event::EVENT_PARTICLE_SPLASH,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                    0,
                );
                broadcast_packet(rak, players, part.encode()).await;
            }
            ProjectileKind::ExpBottle => {
                let glass_sound = LevelSoundEventPacket::new(
                    level_event::SOUND_GLASS,
                    hit_x as f32,
                    hit_y as f32,
                    hit_z as f32,
                );
                broadcast_packet(rak, players, glass_sound.encode()).await;
            }
        }
    }
}

fn is_collidable_block(block_id: u8) -> bool {
    // Air, liquids, flowers, grass, snow layers are not solid colliders
    match block_id {
        0 | 8 | 9 | 10 | 11 | 31 | 37 | 38 | 51 | 78 => false,
        _ => true,
    }
}

async fn send_packet(rak: &RakNetServer, addr: SocketAddr, data: Vec<u8>) {
    let batch = crate::protocol::batch::BatchPacket::encode(&[data]);
    rak.send_encapsulated(addr, batch, true).await;
}

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
