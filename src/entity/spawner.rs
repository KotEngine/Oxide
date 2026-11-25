use std::sync::Arc;
use std::collections::HashMap;
use std::net::SocketAddr;
use parking_lot::RwLock;
use rand::Rng;

use crate::level::World;
use crate::protocol::handler::PlayerState;
use crate::raknet::RakNetServer;
use crate::entity::manager::EntityManager;

/// Manages natural spawning of hostile and passive mobs around active players.
pub struct MobSpawner;

impl Default for MobSpawner {
    fn default() -> Self {
        Self::new()
    }
}

impl MobSpawner {
    pub fn new() -> Self {
        Self
    }

    /// Spawns mobs in the world near active players if conditions are met.
    pub async fn tick(
        &self,
        dimension: i32,
        world: &Arc<RwLock<World>>,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
        entity_manager: &Arc<EntityManager>,
        rak: &RakNetServer,
        next_eid: &Arc<RwLock<i64>>,
    ) {

        // Limit maximum mobs currently alive
        let current_mob_count = entity_manager.count();
        if current_mob_count >= 60 {
            return;
        }

        let player_snapshots: Vec<(f64, f64, f64, i32)> = {
            let guard = players.read();
            guard.values()
                .filter(|s| s.dimension == dimension && s.spawned)
                .map(|s| (s.x, s.y, s.z, s.gamemode))
                .collect()
        };

        if player_snapshots.is_empty() {
            return;
        }

        // Collect mobs to spawn inside a scoped block so ThreadRng is dropped before any await
        let to_spawn: Vec<(i32, String, f64, f64, f64, f32)> = {
            let mut rng = rand::thread_rng();
            let mut results = Vec::new();

            for (px, py, pz, _gm) in player_snapshots {
                if !rng.gen_bool(0.4) {
                    continue;
                }

                let angle = rng.gen_range(0.0..(std::f64::consts::PI * 2.0));
                let dist = rng.gen_range(24.0..48.0);
                let spawn_x = px + angle.cos() * dist;
                let spawn_z = pz + angle.sin() * dist;

                let bx = spawn_x.floor() as i32;
                let bz = spawn_z.floor() as i32;

                let (surface_y, ground_id) = {
                    let w = world.read();
                    let mut sy = (py.floor() as i32 + 10).min(125);
                    while sy > 1 {
                        let id = w.get_block_id_opt(bx, sy, bz);
                        if id != 0 && id != 8 && id != 9 && id != 10 && id != 11 {
                            break;
                        }
                        sy -= 1;
                    }
                    (sy, w.get_block_id_opt(bx, sy, bz))
                };

                if surface_y <= 1 || surface_y >= 126 {
                    continue;
                }

                let space_clear = {
                    let w = world.read();
                    w.get_block_id_opt(bx, surface_y + 1, bz) == 0 &&
                    w.get_block_id_opt(bx, surface_y + 2, bz) == 0
                };

                if !space_clear {
                    continue;
                }

                let spawn_y = (surface_y + 1) as f64;

                match dimension {
                    0 => {
                        let is_grass = ground_id == 2;
                        let is_night = {
                            let w = world.read();
                            let time = w.time;
                            let time_in_day = time % 24000;
                            time_in_day >= 13000 && time_in_day <= 23000
                        };

                        if is_night || surface_y < 55 {
                            let (mob_type, name, hp) = match rng.gen_range(0..12) {
                                0..=3 => (32, "Zombie".to_string(), 20.0),
                                4..=6 => (34, "Skeleton".to_string(), 20.0),
                                7..=8 => (33, "Creeper".to_string(), 20.0),
                                9..=10 => (35, "Spider".to_string(), 16.0),
                                _ => (45, "Witch".to_string(), 26.0),
                            };
                            results.push((mob_type, name, spawn_x, spawn_y, spawn_z, hp));
                        } else if is_grass && rng.gen_bool(0.35) {
                            let (mob_type, name, hp) = match rng.gen_range(0..10) {
                                0..=1 => (11, "Cow".to_string(), 10.0),
                                2..=3 => (12, "Pig".to_string(), 10.0),
                                4..=5 => (13, "Sheep".to_string(), 8.0),
                                6..=7 => (10, "Chicken".to_string(), 4.0),
                                8 => (14, "Wolf".to_string(), 8.0),
                                _ => (18, "Rabbit".to_string(), 3.0),
                            };
                            results.push((mob_type, name, spawn_x, spawn_y, spawn_z, hp));
                        }
                    }
                    1 => {
                        let (mob_type, name, hp) = match rng.gen_range(0..12) {
                            0..=5 => (36, "Zombie Pigman".to_string(), 20.0),
                            6..=7 => (42, "Magma Cube".to_string(), 16.0),
                            8..=9 => (43, "Blaze".to_string(), 20.0),
                            10 => (41, "Ghast".to_string(), 10.0),
                            _ => (48, "Wither Skeleton".to_string(), 20.0),
                        };
                        results.push((mob_type, name, spawn_x, spawn_y, spawn_z, hp));
                    }
                    2 => {
                        results.push((38, "Enderman".to_string(), spawn_x, spawn_y, spawn_z, 40.0));
                    }
                    _ => {}
                }
            }

            results
        };

        // Now spawn the entities asynchronously
        for (mob_type, name, sx, sy, sz, hp) in to_spawn {
            let eid = {
                let mut guard = next_eid.write();
                let id = *guard;
                *guard += 1;
                id
            };

            entity_manager.spawn_mob_in_dim(
                eid,
                dimension,
                mob_type,
                name,
                sx,
                sy,
                sz,
                hp,
                rak,
                players,
            ).await;
        }
    }
}
