use std::net::SocketAddr;
use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::RwLock;
use tracing::{info, warn, debug};

use crate::level::World;
use crate::protocol::info::ProtocolInfo;
use crate::protocol::packet::DataPacket;
use crate::protocol::batch::BatchPacket;
use crate::protocol::handler::{PacketHandler, IncomingPacket, PlayerState};
use crate::protocol::packets::*;
use crate::protocol::types::{EntityMetadata, GameRules};
use crate::raknet::{RakNetServer, RakNetEvent};
use crate::utils::{Config, UUID};

pub struct OxideServer {
    pub rak: Arc<RakNetServer>,
    pub world: Arc<RwLock<World>>,
    pub nether_world: Arc<RwLock<World>>,
    pub end_world: Arc<RwLock<World>>,
    pub players: Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    pub config: Arc<RwLock<Config>>,
    pub command_manager: Arc<crate::command::CommandManager>,
    pub block_registry: Arc<crate::block::BlockRegistry>,
    pub item_registry: Arc<crate::item::ItemRegistry>,
    pub crafting_manager: Arc<crate::inventory::CraftingManager>,
    pub event_bus: Arc<crate::event::EventBus>,
    pub scheduler: Arc<crate::scheduler::Scheduler>,
    pub plugin_manager: Arc<crate::plugin::PluginManager>,
    pub permission_manager: Arc<crate::permission::PermissionManager>,
    pub projectile_manager: Arc<crate::entity::ProjectileManager>,
    pub entity_manager: Arc<crate::entity::EntityManager>,
    pub mob_spawner: Arc<crate::entity::MobSpawner>,
    pub tile_manager: Arc<crate::tile::TileManager>,
    pub liquid_sim: Arc<crate::level::LiquidSimulation>,
    pub weather: Arc<crate::level::WeatherSimulation>,
    pub bans: Arc<RwLock<HashMap<String, String>>>,
    pub banned_ips: Arc<RwLock<HashMap<String, String>>>,
    pub whitelist: Arc<RwLock<std::collections::HashSet<String>>>,
    pub whitelist_enabled: Arc<std::sync::atomic::AtomicBool>,
    pub next_entity_id: Arc<RwLock<i64>>,
    pub server_tick: Arc<std::sync::atomic::AtomicU64>,
    pub data_dir: std::path::PathBuf,
    pub shutdown_notify: Arc<tokio::sync::Notify>,
    pub redstone_sched: Arc<crate::level::redstone::RedstoneScheduler>,
}

impl OxideServer {
    pub async fn new(addr: SocketAddr) -> anyhow::Result<Self> {
        let data_dir = std::env::current_dir().unwrap_or_default();
        let config = load_or_create_config();
        let player_count = 0i64;
        let max_players = config.get_int("max_players");
        let motd_name = config.get_string("motd");
        let server_guid: i64 = {
            use rand::Rng;
            rand::thread_rng().gen::<u64>() as i64
        };
        let server_name = format!(
            "MCPE;{};{};{};{};{};{};Oxide;Survival;",
            motd_name,
            ProtocolInfo::CURRENT_PROTOCOL,
            ProtocolInfo::MINECRAFT_VERSION_NETWORK,
            player_count,
            max_players,
            server_guid,
        );
        let rak = Arc::new(RakNetServer::new(addr, server_name).await?);
        info!("Oxide server starting on {}", addr);

        let level_type = config.get_string("level_type");
        let level_seed = config.get_int("level_seed") as u64;
        let worlds_dir = data_dir.join("worlds");
        let world = World::with_generator_and_dir(&level_type, level_seed, Some(worlds_dir.join("world")), "world");
        let nether_world = World::with_generator_and_dir("hell", level_seed.wrapping_add(100), Some(worlds_dir.join("world_nether")), "world_nether");
        let end_world = World::with_generator_and_dir("ender", level_seed.wrapping_add(200), Some(worlds_dir.join("world_end")), "world_end");

        let block_registry = Arc::new(crate::block::BlockRegistry::default());
        let item_registry = Arc::new(crate::item::ItemRegistry::default());
        let crafting_manager = Arc::new(crate::inventory::CraftingManager::default());
        let event_bus = Arc::new(crate::event::EventBus::new());
        let scheduler = Arc::new(crate::scheduler::Scheduler::new());
        let plugin_manager = Arc::new(crate::plugin::PluginManager::new());
        plugin_manager.load_directory("plugins");
        let permission_manager = Arc::new(crate::permission::PermissionManager::new());
        let wl_enabled = config.get_bool("white_list");

        let server = Self {
            rak,
            world: Arc::new(RwLock::new(world)),
            nether_world: Arc::new(RwLock::new(nether_world)),
            end_world: Arc::new(RwLock::new(end_world)),
            players: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(config)),
            command_manager: Arc::new(crate::command::CommandManager::new()),
            block_registry,
            item_registry,
            crafting_manager,
            event_bus,
            scheduler,
            plugin_manager,
            permission_manager,
            projectile_manager: Arc::new(crate::entity::ProjectileManager::new()),
            entity_manager: Arc::new(crate::entity::EntityManager::new()),
            mob_spawner: Arc::new(crate::entity::MobSpawner::new()),
            tile_manager: Arc::new(crate::tile::TileManager::new()),
            liquid_sim: Arc::new(crate::level::LiquidSimulation::new()),
            weather: Arc::new(crate::level::WeatherSimulation::new()),
            bans: Arc::new(RwLock::new(HashMap::new())),
            banned_ips: Arc::new(RwLock::new(HashMap::new())),
            whitelist: Arc::new(RwLock::new(std::collections::HashSet::new())),
            whitelist_enabled: Arc::new(std::sync::atomic::AtomicBool::new(wl_enabled)),
            next_entity_id: Arc::new(RwLock::new(1)),
            server_tick: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            data_dir,
            shutdown_notify: Arc::new(tokio::sync::Notify::new()),
            redstone_sched: Arc::new(crate::level::redstone::RedstoneScheduler::new()),
        };
        server.load_persistent_state();
        Ok(server)
    }

    pub async fn start(self) -> anyhow::Result<()> {
        let rak = self.rak.clone();
        rak.clone().start().await?;

        let tick_handles = TickHandles {
            rak: self.rak.clone(),
            world: self.world.clone(),
            nether_world: self.nether_world.clone(),
            end_world: self.end_world.clone(),
            players: self.players.clone(),
            scheduler: self.scheduler.clone(),
            projectile_manager: self.projectile_manager.clone(),
            entity_manager: self.entity_manager.clone(),
            mob_spawner: self.mob_spawner.clone(),
            tile_manager: self.tile_manager.clone(),
            crafting_manager: self.crafting_manager.clone(),
            liquid_sim: self.liquid_sim.clone(),
            weather: self.weather.clone(),
            next_entity_id: self.next_entity_id.clone(),
            server_tick: self.server_tick.clone(),
            data_dir: self.data_dir.clone(),
            redstone_sched: self.redstone_sched.clone(),
        };
        tokio::spawn(async move {
            tick_loop(tick_handles).await;
        });

        let event_self = self.clone();
        let notify = self.shutdown_notify.clone();
        tokio::select! {
            res = event_loop(event_self) => {
                let _ = self.save_all();
                res
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Ctrl+C received: saving worlds, tiles, and online players...");
                self.graceful_disconnect_all("Server closed").await;
                let saved = self.save_all();
                info!("Saved {} chunks and all persistent state. Goodbye.", saved);
                Ok(())
            }
            _ = notify.notified() => {
                tokio::time::sleep(tokio::time::Duration::from_millis(120)).await;
                self.graceful_disconnect_all("Server closed").await;
                let saved = self.save_all();
                info!("Saved {} chunks and all persistent state. Goodbye.", saved);
                Ok(())
            }
        }
    }

    pub fn request_shutdown(&self) {
        self.shutdown_notify.notify_waiters();
    }

    async fn graceful_disconnect_all(&self, reason: &str) {
        let addrs: Vec<SocketAddr> = self.players.read().keys().copied().collect();
        let dc = DisconnectPacket::new(reason.to_string()).encode();
        for addr in &addrs {
            send_packet(&self.rak, *addr, dc.clone()).await;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    pub fn load_persistent_state(&self) {
        // Tiles
        let tiles_path = self.data_dir.join("data").join("tiles.dat");
        if let Some(raw) = crate::level::storage::load_bytes_compressed(&tiles_path) {
            let _ = self.tile_manager.import_tiles_bytes(&raw);
        }

        // Ops
        let ops_path = self.data_dir.join("ops.json");
        if let Ok(content) = std::fs::read_to_string(&ops_path) {
            if let Ok(names) = serde_json::from_str::<Vec<String>>(&content) {
                for name in names {
                    let key = format!("op.{}", name.to_lowercase());
                    self.permission_manager.add(crate::permission::Permission::new(key));
                }
            }
        }

        // Player Bans
        let bans_path = self.data_dir.join("banned-players.json");
        if let Ok(content) = std::fs::read_to_string(&bans_path) {
            if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&content) {
                *self.bans.write() = map;
            }
        }

        // IP Bans
        let ip_bans_path = self.data_dir.join("banned-ips.json");
        if let Ok(content) = std::fs::read_to_string(&ip_bans_path) {
            if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&content) {
                *self.banned_ips.write() = map;
            }
        }

        self.load_whitelist();
    }

    pub fn load_whitelist(&self) {
        let wl_path = self.data_dir.join("white-list.txt");
        if let Ok(content) = std::fs::read_to_string(&wl_path) {
            let mut wl = self.whitelist.write();
            wl.clear();
            for line in content.lines() {
                let clean = line.trim().to_lowercase();
                if !clean.is_empty() && !clean.starts_with('#') {
                    wl.insert(clean);
                }
            }
        }
    }

    pub fn save_ops(&self) {
        let mut names: Vec<String> = self
            .permission_manager
            .keys()
            .into_iter()
            .filter_map(|k| k.strip_prefix("op.").map(|s| s.to_string()))
            .collect();
        names.sort();
        if let Ok(json) = serde_json::to_string_pretty(&names) {
            let _ = crate::level::storage::atomic_write(&self.data_dir.join("ops.json"), json.as_bytes());
        }
    }

    pub fn save_bans(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&*self.bans.read()) {
            let _ = crate::level::storage::atomic_write(&self.data_dir.join("banned-players.json"), json.as_bytes());
        }
        if let Ok(json) = serde_json::to_string_pretty(&*self.banned_ips.read()) {
            let _ = crate::level::storage::atomic_write(&self.data_dir.join("banned-ips.json"), json.as_bytes());
        }
    }

    pub fn save_whitelist(&self) {
        let mut names: Vec<String> = self.whitelist.read().iter().cloned().collect();
        names.sort();
        let text = names.join("\n");
        let _ = crate::level::storage::atomic_write(&self.data_dir.join("white-list.txt"), text.as_bytes());
        let wl_on = self.whitelist_enabled.load(std::sync::atomic::Ordering::SeqCst);
        let mut cfg = self.config.write();
        cfg.set("white_list", serde_json::Value::Bool(wl_on));
        let _ = cfg.save(self.data_dir.join("oxide.json"));
    }

    pub fn is_op(&self, name: &str) -> bool {
        self.permission_manager.get(&format!("op.{}", name.to_lowercase())).is_some()
    }

    pub fn add_op(&self, name: &str) {
        let lower = name.to_lowercase();
        self.permission_manager.add(crate::permission::Permission::new(format!("op.{}", lower)));
        self.save_ops();
        if let Some((addr, _)) = self.find_player_by_name(&lower) {
            if let Some(s) = self.players.read().get(&addr) {
                let mut settings = AdventureSettingsPacket::new();
                settings.world_builder = true;
                settings.allow_flight = s.allow_flight || s.gamemode == 1 || s.gamemode == 3;
                settings.is_flying = s.is_flying;
                settings.user_permission = adventure_settings::PERMISSION_OPERATOR;
                self.send_packet_to(addr, settings.encode());
            }
        }
    }

    pub fn remove_op(&self, name: &str) -> bool {
        let lower = name.to_lowercase();
        let existed = self.permission_manager.remove(&format!("op.{}", lower));
        self.save_ops();
        if let Some((addr, _)) = self.find_player_by_name(&lower) {
            if let Some(s) = self.players.read().get(&addr) {
                let mut settings = AdventureSettingsPacket::new();
                settings.world_builder = true;
                settings.allow_flight = s.allow_flight || s.gamemode == 1 || s.gamemode == 3;
                settings.is_flying = s.is_flying;
                settings.user_permission = adventure_settings::PERMISSION_NORMAL;
                self.send_packet_to(addr, settings.encode());
            }
        }
        existed
    }

    pub fn save_all(&self) -> usize {
        let mut total_chunks = 0usize;
        total_chunks += self.world.write().save_all();
        total_chunks += self.nether_world.write().save_all();
        total_chunks += self.end_world.write().save_all();

        let tiles_bytes = self.tile_manager.export_tiles_bytes();
        let _ = crate::level::storage::save_bytes_compressed(
            &self.data_dir.join("data").join("tiles.dat"),
            &tiles_bytes,
        );

        let snapshots: Vec<PlayerState> = {
            let p = self.players.read();
            p.values().filter(|s| s.logged_in).map(clone_state).collect()
        };
        for st in &snapshots {
            let _ = crate::level::storage::save_player_dat(&self.data_dir, st);
        }

        self.save_ops();
        self.save_bans();
        total_chunks
    }

    pub fn send_message_to(&self, addr: SocketAddr, msg: &str) {
        let pk = TextPacket::system(msg.to_string());
        self.send_packet_to(addr, pk.encode());
    }

    pub fn send_packet_to(&self, addr: SocketAddr, data: Vec<u8>) {
        let rak = self.rak.clone();
        tokio::spawn(async move {
            send_packet(&rak, addr, data).await;
        });
    }

    pub fn broadcast_packet(&self, data: Vec<u8>) {
        let rak = self.rak.clone();
        let players = self.players.clone();
        tokio::spawn(async move {
            broadcast_except(&rak, &players, None, data).await;
        });
    }

    pub fn broadcast_message(&self, msg: &str) {
        let pk = TextPacket::system(msg.to_string());
        self.broadcast_packet(pk.encode());
    }

    pub fn set_time(&self, time: i32) {
        {
            let mut w = self.world.write();
            w.time = time;
        }
        let pk = SetTimePacket::new(time);
        self.broadcast_packet(pk.encode());
    }

    pub fn add_time(&self, delta: i32) {
        let new_time = {
            let mut w = self.world.write();
            w.time = (w.time + delta) % 24000;
            w.time
        };
        let pk = SetTimePacket::new(new_time);
        self.broadcast_packet(pk.encode());
    }

    pub fn set_player_gamemode(&self, addr: SocketAddr, mode: i32) {
        let mode_name = match mode {
            0 => "Survival",
            1 => "Creative",
            2 => "Adventure",
            3 => "Spectator",
            _ => "Survival",
        };

        let (is_op, is_flying) = {
            let mut guard = self.players.write();
            if let Some(state) = guard.get_mut(&addr) {
                state.gamemode = mode;
                state.allow_flight = mode == 1 || mode == 3;
                if !state.allow_flight {
                    state.is_flying = false;
                }
                (self.is_op(&state.username), state.is_flying)
            } else {
                return;
            }
        };

        let gt = SetPlayerGameTypePacket::new(mode);
        self.send_packet_to(addr, gt.encode());

        let mut settings = AdventureSettingsPacket::new();
        settings.world_builder = true;
        settings.allow_flight = mode == 1 || mode == 3;
        settings.is_flying = is_flying;
        settings.user_permission = if is_op {
            adventure_settings::PERMISSION_OPERATOR
        } else {
            adventure_settings::PERMISSION_NORMAL
        };
        self.send_packet_to(addr, settings.encode());

        if mode == 1 {
            let eid = self.players.read().get(&addr).map(|s| s.entity_id).unwrap_or(0);
            let mut creative_pk = ContainerSetContentPacket::new(crate::protocol::types::ContainerIds::CREATIVE as u32);
            creative_pk.target_eid = eid;
            creative_pk.items = crate::item::creative::get_creative_items().to_vec();
            self.send_packet_to(addr, creative_pk.encode());
        }

        self.send_message_to(addr, &format!("Your game mode has been updated to {}", mode_name));
    }

    pub fn teleport_player(&self, addr: SocketAddr, x: f64, y: f64, z: f64) {
        let (entity_id, yaw, pitch, dim) = {
            let mut guard = self.players.write();
            if let Some(state) = guard.get_mut(&addr) {
                state.x = x;
                state.y = y;
                state.z = z;
                (state.entity_id, state.yaw, state.pitch, state.dimension)
            } else {
                return;
            }
        };

        let mut mv = MovePlayerPacket::new(entity_id, x as f32, (y + 1.62) as f32, z as f32, yaw, pitch);
        mv.mode = crate::protocol::packets::move_player::MODE_TELEPORT;
        self.send_packet_to(addr, mv.encode());

        let mut other_mv = MovePlayerPacket::new(entity_id, x as f32, (y + 1.62) as f32, z as f32, yaw, pitch);
        other_mv.mode = crate::protocol::packets::move_player::MODE_TELEPORT;
        let rak = self.rak.clone();
        let players = self.players.clone();
        tokio::spawn(async move {
            broadcast_in_dim(&rak, &players, dim, Some(addr), other_mv.encode()).await;
        });
    }

    pub fn kill_player(&self, addr: SocketAddr) {
        let name = {
            let mut guard = self.players.write();
            if let Some(state) = guard.get_mut(&addr) {
                state.health = 0;
                state.username.clone()
            } else {
                return;
            }
        };
        let hp = SetHealthPacket::new(0);
        self.send_packet_to(addr, hp.encode());
        self.broadcast_message(&format!("{} died", name));
    }

    pub fn kick_player(&self, addr: SocketAddr, reason: &str) {
        let dc = DisconnectPacket::new(reason.to_string());
        self.send_packet_to(addr, dc.encode());
        let rak = self.rak.clone();
        let reason_str = reason.to_string();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            let _ = rak.close_session(addr, reason_str).await;
        });
    }

    pub fn find_player_by_name(&self, name: &str) -> Option<(SocketAddr, String)> {
        let clean = name.trim().trim_matches('"');
        let guard = self.players.read();
        let lower = clean.to_lowercase();
        for (addr, state) in guard.iter() {
            if state.username.to_lowercase() == lower {
                return Some((*addr, state.username.clone()));
            }
        }
        for (addr, state) in guard.iter() {
            if state.username.to_lowercase().starts_with(&lower) {
                return Some((*addr, state.username.clone()));
            }
        }
        None
    }

    pub fn get_world(&self, dimension: i32) -> Arc<RwLock<World>> {
        match dimension {
            1 => self.nether_world.clone(),
            2 => self.end_world.clone(),
            _ => self.world.clone(),
        }
    }

    pub async fn change_player_dimension(
        &self,
        addr: SocketAddr,
        new_dim: i32,
        target_x: f64,
        target_y: f64,
        target_z: f64,
    ) {
        change_player_dimension_raw_with_em(
            &self.rak,
            &self.players,
            Some(&self.entity_manager),
            Some(&self.tile_manager),
            &self.world,
            &self.nether_world,
            &self.end_world,
            addr,
            new_dim,
            target_x,
            target_y,
            target_z,
        ).await;
    }
}

#[allow(dead_code, clippy::too_many_arguments)]
async fn change_player_dimension_raw(
    rak: &RakNetServer,
    players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    world: &Arc<RwLock<World>>,
    nether_world: &Arc<RwLock<World>>,
    end_world: &Arc<RwLock<World>>,
    addr: SocketAddr,
    new_dim: i32,
    target_x: f64,
    target_y: f64,
    target_z: f64,
) {
    change_player_dimension_raw_with_em(
        rak, players, None, None, world, nether_world, end_world, addr, new_dim, target_x, target_y, target_z,
    ).await;
}

#[allow(clippy::too_many_arguments)]
async fn change_player_dimension_raw_with_em(
    rak: &RakNetServer,
    players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    entity_manager: Option<&Arc<crate::entity::EntityManager>>,
    tile_manager: Option<&Arc<crate::tile::TileManager>>,
    world: &Arc<RwLock<World>>,
    nether_world: &Arc<RwLock<World>>,
    end_world: &Arc<RwLock<World>>,
    addr: SocketAddr,
    new_dim: i32,
    target_x: f64,
    target_y: f64,
    target_z: f64,
) {
    let (eid, old_dim, username, uuid, held_item, armor, metadata) = {
        let mut p = players.write();
        let Some(s) = p.get_mut(&addr) else { return; };
        let old = s.dimension;
        s.dimension = new_dim;
        s.sent_chunks.clear();
        s.x = target_x;
        s.y = target_y;
        s.z = target_z;
        s.portal_ticks = 100;
        (
            s.entity_id,
            old,
            s.username.clone(),
            s.uuid,
            s.inventory.held_item(),
            [s.inventory.helmet(), s.inventory.chestplate(), s.inventory.leggings(), s.inventory.boots()],
            s.metadata.clone(),
        )
    };

    if old_dim != new_dim {
        // Despawn this player from old dimension peers
        let rm_self = RemoveEntityPacket::new(eid);
        broadcast_in_dim(rak, players, old_dim, Some(addr), rm_self.encode()).await;

        // Despawn old dimension players from this client
        let old_peer_removals: Vec<Vec<u8>> = {
            let guard = players.read();
            guard.iter()
                .filter(|(a, s)| **a != addr && s.logged_in && s.dimension == old_dim)
                .map(|(_, s)| RemoveEntityPacket::new(s.entity_id).encode())
                .collect()
        };
        if !old_peer_removals.is_empty() {
            send_raw(rak, addr, BatchPacket::encode(&old_peer_removals)).await;
        }

        if let Some(em) = entity_manager {
            em.despawn_entities_in_dim_for(addr, old_dim, rak).await;
        }
    }

    let change_dim_pk = crate::protocol::packets::ChangeDimensionPacket::new(
        new_dim,
        target_x as f32,
        target_y as f32,
        target_z as f32,
    );
    send_packet(rak, addr, change_dim_pk.encode()).await;

    let play_status = crate::protocol::packets::PlayStatusPacket::new(
        crate::protocol::packets::play_status::PLAYER_SPAWN,
    );
    send_packet(rak, addr, play_status.encode()).await;

    let wanted = {
        let center_cx = (target_x as i32) >> 4;
        let center_cz = (target_z as i32) >> 4;
        let mut list = Vec::new();
        let mut p = players.write();
        if let Some(s) = p.get_mut(&addr) {
            for dx in -DEFAULT_VIEW_DISTANCE..=DEFAULT_VIEW_DISTANCE {
                for dz in -DEFAULT_VIEW_DISTANCE..=DEFAULT_VIEW_DISTANCE {
                    let coord = (center_cx + dx, center_cz + dz);
                    if s.sent_chunks.insert(coord) {
                        list.push(coord);
                    }
                }
            }
        }
        list
    };

    let chunks: Vec<Vec<u8>> = {
        let target_world = match new_dim {
            1 => nether_world,
            2 => end_world,
            _ => world,
        };
        let mut w = target_world.write();
        wanted.iter()
            .map(|&(x, z)| {
                let tiles = tile_manager.map(|tm| tm.serialize_chunk_tiles_in_dim(new_dim, x, z)).unwrap_or_default();
                FullChunkDataPacket::new(x, z, w.serialize_chunk_with_tiles(x, z, &tiles)).encode()
            })
            .collect()
    };

    for pk in chunks {
        send_raw(rak, addr, BatchPacket::encode(&[pk])).await;
    }

    let mut mv = crate::protocol::packets::MovePlayerPacket::new(
        eid,
        target_x as f32,
        target_y as f32 + 1.62,
        target_z as f32,
        0.0,
        0.0,
    );
    mv.mode = crate::protocol::packets::move_player::MODE_RESET;
    send_packet(rak, addr, mv.encode()).await;

    if old_dim != new_dim {
        // Spawn existing players in new dimension to this player
        let peers: Vec<(i64, String, UUID, f64, f64, f64, f32, f32, crate::protocol::types::Item, [crate::protocol::types::Item; 4], EntityMetadata)> = {
            let guard = players.read();
            guard.iter()
                .filter(|(a, s)| **a != addr && s.logged_in && s.dimension == new_dim)
                .map(|(_, s)| (
                    s.entity_id, s.username.clone(), s.uuid, s.x, s.y, s.z, s.yaw, s.pitch,
                    s.inventory.held_item(),
                    [s.inventory.helmet(), s.inventory.chestplate(), s.inventory.leggings(), s.inventory.boots()],
                    s.metadata.clone(),
                ))
                .collect()
        };
        for (peid, pname, puuid, px, py, pz, pyaw, ppitch, pitem, parmor, pmeta) in peers {
            let mut add = AddPlayerPacket::new(puuid, pname, peid, px as f32, (py + 1.62) as f32, pz as f32);
            add.yaw = pyaw;
            add.pitch = ppitch;
            add.item = pitem;
            add.metadata = pmeta;
            send_packet(rak, addr, add.encode()).await;

            let mut arm = MobArmorEquipmentPacket::new(peid);
            arm.helmet = parmor[0].clone();
            arm.chestplate = parmor[1].clone();
            arm.leggings = parmor[2].clone();
            arm.boots = parmor[3].clone();
            send_packet(rak, addr, arm.encode()).await;
        }

        // Spawn this player to peers in new dimension
        let mut add_self = AddPlayerPacket::new(uuid, username, eid, target_x as f32, (target_y + 1.62) as f32, target_z as f32);
        add_self.item = held_item;
        add_self.metadata = metadata;
        broadcast_in_dim(rak, players, new_dim, Some(addr), add_self.encode()).await;

        let mut arm_self = MobArmorEquipmentPacket::new(eid);
        arm_self.helmet = armor[0].clone();
        arm_self.chestplate = armor[1].clone();
        arm_self.leggings = armor[2].clone();
        arm_self.boots = armor[3].clone();
        broadcast_in_dim(rak, players, new_dim, Some(addr), arm_self.encode()).await;

        if let Some(em) = entity_manager {
            em.send_entities_in_dim_to(addr, new_dim, rak).await;
        }
    }
}

fn try_ignite_nether_portal(world: &mut World, click_x: i32, click_y: i32, click_z: i32) -> Option<Vec<(i32, i32, i32)>> {
    let start_y = click_y + 1;

    if let Some(blocks) = check_portal_frame_x(world, click_x, start_y, click_z) {
        for &(bx, by, bz) in &blocks {
            world.set_block(bx, by, bz, 90, 1);
        }
        return Some(blocks);
    }

    if let Some(blocks) = check_portal_frame_z(world, click_x, start_y, click_z) {
        for &(bx, by, bz) in &blocks {
            world.set_block(bx, by, bz, 90, 2);
        }
        return Some(blocks);
    }

    None
}

fn check_portal_frame_x(world: &mut World, start_x: i32, start_y: i32, z: i32) -> Option<Vec<(i32, i32, i32)>> {
    let mut min_x = start_x;
    while min_x >= start_x - 10 && world.get_block_id(min_x, start_y - 1, z) == 49 {
        if world.get_block_id(min_x, start_y, z) == 49 {
            break;
        }
        min_x -= 1;
    }
    let left_pillar_x = min_x;
    if world.get_block_id(left_pillar_x, start_y, z) != 49 {
        return None;
    }

    let mut max_x = start_x;
    while max_x <= start_x + 10 && world.get_block_id(max_x, start_y - 1, z) == 49 {
        if world.get_block_id(max_x, start_y, z) == 49 {
            break;
        }
        max_x += 1;
    }
    let right_pillar_x = max_x;
    if world.get_block_id(right_pillar_x, start_y, z) != 49 {
        return None;
    }

    let inner_width = right_pillar_x - left_pillar_x - 1;
    if inner_width < 2 || inner_width > 21 {
        return None;
    }

    let mut top_y = start_y;
    while top_y <= start_y + 21 {
        let b = world.get_block_id(left_pillar_x + 1, top_y, z);
        if b == 49 {
            break;
        }
        top_y += 1;
    }
    let inner_height = top_y - start_y;
    if inner_height < 3 || inner_height > 21 {
        return None;
    }

    for x in (left_pillar_x + 1)..right_pillar_x {
        if world.get_block_id(x, start_y - 1, z) != 49 || world.get_block_id(x, top_y, z) != 49 {
            return None;
        }
    }

    for y in start_y..top_y {
        if world.get_block_id(left_pillar_x, y, z) != 49 || world.get_block_id(right_pillar_x, y, z) != 49 {
            return None;
        }
    }

    let mut portal_blocks = Vec::new();
    for x in (left_pillar_x + 1)..right_pillar_x {
        for y in start_y..top_y {
            let id = world.get_block_id(x, y, z);
            if id != 0 && id != 51 {
                return None;
            }
            portal_blocks.push((x, y, z));
        }
    }

    Some(portal_blocks)
}

fn check_portal_frame_z(world: &mut World, x: i32, start_y: i32, start_z: i32) -> Option<Vec<(i32, i32, i32)>> {
    let mut min_z = start_z;
    while min_z >= start_z - 10 && world.get_block_id(x, start_y - 1, min_z) == 49 {
        if world.get_block_id(x, start_y, min_z) == 49 {
            break;
        }
        min_z -= 1;
    }
    let left_pillar_z = min_z;
    if world.get_block_id(x, start_y, left_pillar_z) != 49 {
        return None;
    }

    let mut max_z = start_z;
    while max_z <= start_z + 10 && world.get_block_id(x, start_y - 1, max_z) == 49 {
        if world.get_block_id(x, start_y, max_z) == 49 {
            break;
        }
        max_z += 1;
    }
    let right_pillar_z = max_z;
    if world.get_block_id(x, start_y, right_pillar_z) != 49 {
        return None;
    }

    let inner_width = right_pillar_z - left_pillar_z - 1;
    if inner_width < 2 || inner_width > 21 {
        return None;
    }

    let mut top_y = start_y;
    while top_y <= start_y + 21 {
        let b = world.get_block_id(x, top_y, left_pillar_z + 1);
        if b == 49 {
            break;
        }
        top_y += 1;
    }
    let inner_height = top_y - start_y;
    if inner_height < 3 || inner_height > 21 {
        return None;
    }

    for z in (left_pillar_z + 1)..right_pillar_z {
        if world.get_block_id(x, start_y - 1, z) != 49 || world.get_block_id(x, top_y, z) != 49 {
            return None;
        }
    }

    for y in start_y..top_y {
        if world.get_block_id(x, y, left_pillar_z) != 49 || world.get_block_id(x, y, right_pillar_z) != 49 {
            return None;
        }
    }

    let mut portal_blocks = Vec::new();
    for z in (left_pillar_z + 1)..right_pillar_z {
        for y in start_y..top_y {
            let id = world.get_block_id(x, y, z);
            if id != 0 && id != 51 {
                return None;
            }
            portal_blocks.push((x, y, z));
        }
    }

    Some(portal_blocks)
}

fn build_destination_portal(world: &mut World, x: i32, y: i32, z: i32) {
    for dx in -1..=2 {
        world.set_block(x + dx, y, z, 49, 0);
        world.set_block(x + dx, y + 4, z, 49, 0);
    }
    for dy in 1..=3 {
        world.set_block(x - 1, y + dy, z, 49, 0);
        world.set_block(x + 2, y + dy, z, 49, 0);
        world.set_block(x, y + dy, z, 90, 1);
        world.set_block(x + 1, y + dy, z, 90, 1);
    }
    for dx in -1..=2 {
        for dy in 1..=4 {
            world.set_block(x + dx, y + dy, z - 1, 0, 0);
            world.set_block(x + dx, y + dy, z + 1, 0, 0);
        }
    }
}

struct TickHandles {
    rak: Arc<RakNetServer>,
    world: Arc<RwLock<World>>,
    nether_world: Arc<RwLock<World>>,
    end_world: Arc<RwLock<World>>,
    players: Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    scheduler: Arc<crate::scheduler::Scheduler>,
    projectile_manager: Arc<crate::entity::ProjectileManager>,
    entity_manager: Arc<crate::entity::EntityManager>,
    mob_spawner: Arc<crate::entity::MobSpawner>,
    tile_manager: Arc<crate::tile::TileManager>,
    crafting_manager: Arc<crate::inventory::CraftingManager>,
    liquid_sim: Arc<crate::level::LiquidSimulation>,
    weather: Arc<crate::level::WeatherSimulation>,
    next_entity_id: Arc<RwLock<i64>>,
    server_tick: Arc<std::sync::atomic::AtomicU64>,
    data_dir: std::path::PathBuf,
    redstone_sched: Arc<crate::level::redstone::RedstoneScheduler>,
}

async fn apply_redstone_result(
    rak: &RakNetServer,
    players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    entity_manager: &Arc<crate::entity::EntityManager>,
    next_eid: &Arc<RwLock<i64>>,
    dim: i32,
    res: crate::level::redstone::RedstoneUpdateResult,
) {
    for (bx, by, bz, bid, bmeta) in res.block_changes {
        let pk = UpdateBlockPacket::new(bx, by as u32, bz, bid as u32, bmeta as u32, update_block::FLAG_ALL);
        broadcast_in_dim(rak, players, dim, None, pk.encode()).await;
    }
    for (tx, ty, tz, fuse) in res.ignited_tnts {
        let tid = {
            let mut g = next_eid.write();
            let v = *g;
            *g += 1;
            v
        };
        entity_manager
            .spawn_tnt_in_dim(tid, dim, tx as f64 + 0.5, ty as f64, tz as f64 + 0.5, fuse, rak, players)
            .await;
    }
}

async fn tick_loop(h: TickHandles) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50));
    let mut tick_counter: u64 = 0;
    loop {
        interval.tick().await;
        tick_counter += 1;
        h.server_tick.store(tick_counter, std::sync::atomic::Ordering::Relaxed);

        h.scheduler.tick(tick_counter);

        {
            h.world.write().tick();
            h.nether_world.write().tick();
            h.end_world.write().tick();
        }

        let worlds = [&h.world, &h.nether_world, &h.end_world];

        // Ticking projectiles across all dimensions
        h.projectile_manager.tick_all(&worlds, &h.entity_manager, &h.players, &h.rak, &h.next_entity_id).await;

        // Ticking active entities across all dimensions
        h.entity_manager.tick_all(&worlds, &h.players, &h.rak, &h.next_entity_id).await;

        // Ticking player potion effects (every tick)
        tick_player_effects(&h.players, &h.rak).await;

        // Ticking mob spawner (every 100 ticks = 5 seconds)
        if tick_counter % 100 == 0 {
            h.mob_spawner.tick(0, &h.world, &h.players, &h.entity_manager, &h.rak, &h.next_entity_id).await;
            h.mob_spawner.tick(1, &h.nether_world, &h.players, &h.entity_manager, &h.rak, &h.next_entity_id).await;
            h.mob_spawner.tick(2, &h.end_world, &h.players, &h.entity_manager, &h.rak, &h.next_entity_id).await;
        }

        // Ticking weather simulation (every tick)
        h.weather.tick(&h.rak, &h.players).await;

        // Thunderstorm lightning strikes (every 300 ticks = 15 sec)
        if tick_counter % 300 == 0 && h.weather.current_state() == crate::level::WeatherState::Thunder {
            let target_pos = {
                let p = h.players.read();
                p.values().filter(|s| s.dimension == 0 && s.spawned).map(|s| (s.x, s.y, s.z)).next()
            };
            if let Some((px, py, pz)) = target_pos {
                let offset_x = (rand::random::<f64>() - 0.5) * 30.0;
                let offset_z = (rand::random::<f64>() - 0.5) * 30.0;
                let lx = px + offset_x;
                let lz = pz + offset_z;
                let eid = {
                    let mut id = h.next_entity_id.write();
                    let val = *id;
                    *id += 1;
                    val
                };
                h.entity_manager.spawn_lightning_in_dim(eid, 0, lx, py, lz, &h.world, &h.rak, &h.players).await;
            }
        }

        // Tick player damage cooldowns, fire burn ticks, and queued death drops
        let mut burning_players: Vec<(SocketAddr, String)> = Vec::new();
        let mut death_drop_queue: Vec<(i32, f64, f64, f64, Vec<crate::protocol::types::Item>, u32)> = Vec::new();
        {
            let mut p = h.players.write();
            for (addr, s) in p.iter_mut() {
                if s.no_damage_ticks > 0 {
                    s.no_damage_ticks -= 1;
                }
                if s.fire_ticks > 0 {
                    s.fire_ticks -= 1;
                    if tick_counter % 20 == 0 && s.gamemode == 0 && s.spawned {
                        burning_players.push((*addr, s.username.clone()));
                    }
                }
                if !s.pending_drops.is_empty() || s.pending_xp_drop > 0 {
                    let drops = std::mem::take(&mut s.pending_drops);
                    let xp = s.pending_xp_drop;
                    s.pending_xp_drop = 0;
                    death_drop_queue.push((s.dimension, s.x, s.y, s.z, drops, xp));
                }
            }
        }
        for (d_dim, dx, dy, dz, items, xp) in death_drop_queue {
            for it in items {
                let deid = {
                    let mut g = h.next_entity_id.write();
                    let v = *g;
                    *g += 1;
                    v
                };
                h.entity_manager.spawn_item_drop_in_dim(deid, d_dim, it, dx, dy + 0.5, dz, &h.rak, &h.players).await;
            }
            if xp > 0 {
                h.entity_manager.spawn_xp_orbs_split_in_dim(d_dim, dx, dy + 0.3, dz, xp, &h.next_entity_id, &h.rak, &h.players).await;
            }
        }
        for (addr, uname) in burning_players {
            let msg = format!("{} burned to death", uname);
            crate::entity::damage::hurt_player(
                &h.players,
                &h.rak,
                addr,
                1.0,
                crate::entity::damage::DamageCause::FireTick,
                false,
                None,
                None,
                Some(&msg),
            )
            .await;
        }

        // Timed redstone updates (button & pressure plate release)
        let due_redstone = h.redstone_sched.take_due(tick_counter);
        for (rdim, rx, ry, rz, expected_id) in due_redstone {
            let is_plate = matches!(expected_id, 70 | 72 | 147 | 148);
            let occupied = if is_plate {
                let p = h.players.read();
                p.values().any(|s| {
                    s.spawned
                        && s.dimension == rdim
                        && s.x.floor() as i32 == rx
                        && s.y.floor() as i32 == ry
                        && s.z.floor() as i32 == rz
                })
            } else {
                false
            };
            if occupied {
                h.redstone_sched.schedule_release(rdim, rx, ry, rz, tick_counter + 10, expected_id);
                continue;
            }
            let world = worlds[rdim.clamp(0, 2) as usize];
            let res_opt = {
                let mut w = world.write();
                let cur_id = w.get_block_id(rx, ry, rz);
                let cur_meta = w.get_block_data_opt(rx, ry, rz);
                if cur_id == expected_id && ((is_plate && cur_meta > 0) || (!is_plate && (cur_meta & 0x08) != 0)) {
                    let next_meta = if is_plate { 0 } else { cur_meta & !0x08 };
                    w.set_block(rx, ry, rz, cur_id, next_meta);
                    let mut rs = crate::level::redstone::update_redstone_around(&mut w, &[(rx, ry, rz)]);
                    rs.block_changes.insert(0, (rx, ry, rz, cur_id, next_meta));
                    Some(rs)
                } else {
                    None
                }
            };
            if let Some(rs) = res_opt {
                let click = LevelEventPacket::new(level_event::EVENT_SOUND_CLICK, rx as f32 + 0.5, ry as f32 + 0.5, rz as f32 + 0.5, 0);
                broadcast_in_dim(&h.rak, &h.players, rdim, None, click.encode()).await;
                apply_redstone_result(&h.rak, &h.players, &h.entity_manager, &h.next_entity_id, rdim, rs).await;
            }
        }

        // Ticking furnaces (every tick)
        let furnace_up = h.tile_manager.tick_furnaces_live(&h.crafting_manager);
        for (fdim, fx, fy, fz, is_lit) in furnace_up.block_toggles {
            let new_id = if is_lit { 62u8 } else { 61u8 };
            let fmeta = {
                let mut w = worlds[fdim.clamp(0, 2) as usize].write();
                let m = w.get_block_data_opt(fx, fy, fz);
                w.set_block(fx, fy, fz, new_id, m);
                m
            };
            let pk = UpdateBlockPacket::new(fx, fy as u32, fz, new_id as u32, fmeta as u32, update_block::FLAG_ALL);
            broadcast_in_dim(&h.rak, &h.players, fdim, None, pk.encode()).await;
        }
        for (viewer, prop, val) in furnace_up.data_updates {
            let dp = ContainerSetDataPacket { window_id: 1, property: prop, value: val };
            send_packet(&h.rak, viewer, dp.encode()).await;
        }
        for (viewer, slot, item) in furnace_up.slot_syncs {
            let sp = ContainerSetSlotPacket::new(1, slot as i32, item);
            send_packet(&h.rak, viewer, sp.encode()).await;
        }

        // Ticking brewing stands (every tick)
        let brew_up = h.tile_manager.tick_brewing_stands(&h.crafting_manager);
        for (viewer, prop, val) in brew_up.data_updates {
            let dp = ContainerSetDataPacket { window_id: 1, property: prop, value: val };
            send_packet(&h.rak, viewer, dp.encode()).await;
        }
        for (viewer, slot, item) in brew_up.slot_syncs {
            let sp = ContainerSetSlotPacket::new(1, slot as i32, item);
            send_packet(&h.rak, viewer, sp.encode()).await;
        }

        // Ticking liquid physics (every 2 ticks = 10 updates/sec)
        if tick_counter % 2 == 0 {
            h.liquid_sim.tick_all(&worlds, &h.rak, &h.players).await;
        }

        // Random block ticks (crops, saplings, grass spread, leaves decay, farmland hydration, fire, ice)
        let is_raining = h.weather.current_state() != crate::level::WeatherState::Clear;
        for (dim_idx, world_arc) in worlds.iter().enumerate() {
            let dim = dim_idx as i32;
            let player_chunks: Vec<(i32, i32)> = {
                let p = h.players.read();
                p.values()
                    .filter(|s| s.dimension == dim && s.spawned)
                    .map(|s| ((s.x.floor() as i32) >> 4, (s.z.floor() as i32) >> 4))
                    .collect()
            };
            if player_chunks.is_empty() {
                continue;
            }
            let changes = {
                let mut w = world_arc.write();
                crate::level::random_tick::tick_random_blocks(&mut w, dim, is_raining, &player_chunks)
            };
            if !changes.is_empty() {
                for (bx, by, bz, bid, bmeta) in changes {
                    let pk = UpdateBlockPacket::new(bx, by as u32, bz, bid as u32, bmeta as u32, update_block::FLAG_ALL);
                    broadcast_in_dim(&h.rak, &h.players, dim, None, pk.encode()).await;
                }
            }
        }

        // Environment hazards (void, lava, portals) every 4 ticks (0.2 sec)
        if tick_counter % 4 == 0 {
            check_environment_hazards(&h.world, &h.nether_world, &h.end_world, &h.players, &h.rak).await;
        }

        // Food regeneration & starvation every 80 ticks (4 seconds)
        if tick_counter % 80 == 0 {
            tick_player_food_and_health(&h.players, &h.rak).await;
        }

        if tick_counter % 40 == 0 {
            let time = { h.world.read().time };
            let pk = SetTimePacket::new(time);
            broadcast_except(&h.rak, &h.players, None, pk.encode()).await;
        }

        if tick_counter % 4 == 0 {
            broadcast_player_positions(&h.rak, &h.players).await;
        }

        // Periodic autosave every 6000 ticks (5 minutes)
        if tick_counter % 6000 == 0 {
            let mut saved = 0usize;
            saved += h.world.write().save_all();
            saved += h.nether_world.write().save_all();
            saved += h.end_world.write().save_all();
            let tiles_bytes = h.tile_manager.export_tiles_bytes();
            let _ = crate::level::storage::save_bytes_compressed(
                &h.data_dir.join("data").join("tiles.dat"),
                &tiles_bytes,
            );
            let snapshots: Vec<PlayerState> = {
                let p = h.players.read();
                p.values().filter(|s| s.logged_in).map(clone_state).collect()
            };
            for st in &snapshots {
                let _ = crate::level::storage::save_player_dat(&h.data_dir, st);
            }
            debug!("Autosave finished ({} chunks, {} players)", saved, snapshots.len());
        }
    }
}

async fn tick_player_effects(
    players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    rak: &RakNetServer,
) {
    let mut effect_updates: Vec<(SocketAddr, i64, u32, Vec<u8>)> = Vec::new();

    {
        let mut p = players.write();
        for (addr, s) in p.iter_mut() {
            if !s.logged_in || s.gamemode != 0 {
                continue;
            }
            let (expired, hp_delta) = s.effects.tick();
            let mut health_changed = false;
            if hp_delta > 0.0 {
                s.health = (s.health + hp_delta as u32).min(20);
                health_changed = true;
            } else if hp_delta < 0.0 {
                let dmg = (-hp_delta) as u32;
                s.health = s.health.saturating_sub(dmg).max(1); // Poison cannot kill
                health_changed = true;
            }

            if health_changed || !expired.is_empty() {
                effect_updates.push((*addr, s.entity_id, s.health, expired));
            }
        }
    }

    for (addr, eid, hp, expired) in effect_updates {
        let hp_pk = SetHealthPacket::new(hp as i32);
        let mut pk_batch = vec![hp_pk.encode()];

        for eff_id in expired {
            let remove_eff = MobEffectPacket {
                entity_runtime_id: eid,
                event_id: 3, // EVENT_REMOVE
                effect_id: eff_id as i32,
                amplifier: 0,
                particles: false,
                duration: 0,
            };
            pk_batch.push(remove_eff.encode());
        }

        let batch = BatchPacket::encode(&pk_batch);
        rak.send_encapsulated(addr, batch, true).await;
    }
}

pub fn exhaust_player(state: &mut PlayerState, amount: f32) -> bool {
    if state.gamemode != 0 {
        return false;
    }
    let old_food = state.food_level;
    let old_sat = state.food_saturation;
    state.exhaustion += amount;
    while state.exhaustion >= 4.0 {
        state.exhaustion -= 4.0;
        if state.food_saturation > 0.0 {
            state.food_saturation = (state.food_saturation - 1.0).max(0.0);
        } else if state.food_level > 0 {
            state.food_level -= 1;
        }
    }
    if state.food_level <= 6 && state.sprinting {
        state.sprinting = false;
    }
    state.food_level != old_food || (state.food_saturation - old_sat).abs() > 1e-4
}

async fn tick_player_food_and_health(
    players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    rak: &RakNetServer,
) {
    let mut updates: Vec<(SocketAddr, i64, u32, u32, f32)> = Vec::new();

    {
        let mut p = players.write();
        for (addr, s) in p.iter_mut() {
            if !s.logged_in || s.gamemode != 0 || s.health == 0 {
                continue;
            }

            if s.food_level >= 18 && s.health < 20 {
                s.health = (s.health + 1).min(20);
                exhaust_player(s, 6.0);
                updates.push((*addr, s.entity_id, s.health, s.food_level, s.food_saturation));
            } else if s.food_level == 0 && s.health > 1 {
                s.health = s.health.saturating_sub(1);
                updates.push((*addr, s.entity_id, s.health, s.food_level, s.food_saturation));
            }
        }
    }

    for (addr, eid, health, food, sat) in updates {
        let hp_pk = SetHealthPacket::new(health as i32);
        let mut attrs_pk = UpdateAttributesPacket::new(eid);
        attrs_pk.attributes = vec![
            Attribute::health(health as f32),
            Attribute::hunger(food as f32),
            Attribute::food_saturation(sat),
        ];
        let batch = BatchPacket::encode(&[hp_pk.encode(), attrs_pk.encode()]);
        rak.send_encapsulated(addr, batch, true).await;
    }
}

async fn check_environment_hazards(
    world: &Arc<RwLock<World>>,
    nether_world: &Arc<RwLock<World>>,
    end_world: &Arc<RwLock<World>>,
    players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    rak: &RakNetServer,
) {
    let list: Vec<(SocketAddr, i64, f64, f64, f64, u32, i32, i32, u32, String)> = {
        let guard = players.read();
        guard.iter().map(|(a, s)| (*a, s.entity_id, s.x, s.y, s.z, s.health, s.gamemode, s.dimension, s.portal_ticks, s.username.clone())).collect()
    };

    for (addr, _eid, px, py, pz, _health, gamemode, dimension, portal_ticks, username) in list {
        let cur_world = match dimension {
            1 => nether_world,
            2 => end_world,
            _ => world,
        };

        let block_at_feet = {
            let mut w = cur_world.write();
            w.get_block_id(px.floor() as i32, py.floor() as i32, pz.floor() as i32)
        };
        let block_at_body = {
            let mut w = cur_world.write();
            w.get_block_id(px.floor() as i32, (py + 1.0).floor() as i32, pz.floor() as i32)
        };

        // 1. Nether Portal (block 90)
        if block_at_feet == 90 || block_at_body == 90 {
            if portal_ticks == 0 {
                if dimension == 0 {
                    let tx = px / 8.0;
                    let ty = 64.0;
                    let tz = pz / 8.0;
                    {
                        let mut nw = nether_world.write();
                        build_destination_portal(&mut nw, tx as i32, ty as i32, tz as i32);
                    }
                    change_player_dimension_raw(rak, players, world, nether_world, end_world, addr, 1, tx, ty + 1.0, tz).await;
                } else if dimension == 1 {
                    let tx = px * 8.0;
                    let ty = 64.0;
                    let tz = pz * 8.0;
                    {
                        let mut ow = world.write();
                        build_destination_portal(&mut ow, tx as i32, ty as i32, tz as i32);
                    }
                    change_player_dimension_raw(rak, players, world, nether_world, end_world, addr, 0, tx, ty + 1.0, tz).await;
                }
            }
            continue;
        }

        // 2. End Portal (block 119)
        if block_at_feet == 119 || block_at_body == 119 {
            if portal_ticks == 0 {
                if dimension != 2 {
                    change_player_dimension_raw(rak, players, world, nether_world, end_world, addr, 2, 0.0, 66.0, 0.0).await;
                } else {
                    let (sx, sy, sz) = {
                        let w = world.read();
                        (w.spawn_x as f64, w.spawn_y as f64, w.spawn_z as f64)
                    };
                    change_player_dimension_raw(rak, players, world, nether_world, end_world, addr, 0, sx, sy, sz).await;
                }
            }
            continue;
        }

        if portal_ticks > 0 {
            let mut guard = players.write();
            if let Some(s) = guard.get_mut(&addr) {
                s.portal_ticks = s.portal_ticks.saturating_sub(1);
            }
        }

        if gamemode != 0 {
            continue; // Survival only for damage hazards
        }

        // 3. Water extinguishes fire
        if block_at_feet == 8 || block_at_feet == 9 || block_at_body == 8 || block_at_body == 9 {
            let mut guard = players.write();
            if let Some(s) = guard.get_mut(&addr) {
                s.fire_ticks = 0;
            }
        }

        // 4. Void check
        if py < -30.0 {
            let msg = format!("{} fell out of the world", username);
            let outcome = crate::entity::damage::hurt_player(
                players,
                rak,
                addr,
                4.0,
                crate::entity::damage::DamageCause::Void,
                false,
                None,
                None,
                Some(&msg),
            )
            .await;

            if outcome.died {
                let (sx, sy, sz) = {
                    let w = world.read();
                    (w.spawn_x, w.spawn_y, w.spawn_z)
                };
                {
                    let mut guard = players.write();
                    if let Some(s) = guard.get_mut(&addr) {
                        s.health = 20;
                        s.fire_ticks = 0;
                        s.fall_distance = 0.0;
                        s.x = sx as f64;
                        s.y = sy as f64;
                        s.z = sz as f64;
                        s.dimension = 0;
                    }
                }
                let respawn_pk = RespawnPacket::new(sx as f32, sy as f32, sz as f32);
                send_packet(rak, addr, respawn_pk.encode()).await;

                let reset_hp = SetHealthPacket::new(20);
                send_packet(rak, addr, reset_hp.encode()).await;
            }
            continue;
        }

        // 5. Lava check
        if block_at_feet == 10 || block_at_feet == 11 || block_at_body == 10 || block_at_body == 11 {
            {
                let mut guard = players.write();
                if let Some(s) = guard.get_mut(&addr) {
                    s.fire_ticks = s.fire_ticks.max(300);
                }
            }
            let msg = format!("{} tried to swim in lava", username);
            crate::entity::damage::hurt_player(
                players,
                rak,
                addr,
                4.0,
                crate::entity::damage::DamageCause::Lava,
                false,
                None,
                None,
                Some(&msg),
            )
            .await;
            continue;
        }

        // 6. Fire block (51) or Cactus (81)
        if block_at_feet == 51 || block_at_body == 51 {
            {
                let mut guard = players.write();
                if let Some(s) = guard.get_mut(&addr) {
                    s.fire_ticks = s.fire_ticks.max(160);
                }
            }
            let msg = format!("{} went up in flames", username);
            crate::entity::damage::hurt_player(
                players,
                rak,
                addr,
                1.0,
                crate::entity::damage::DamageCause::Fire,
                false,
                None,
                None,
                Some(&msg),
            )
            .await;
        } else if block_at_feet == 81 {
            let msg = format!("{} was pricked to death", username);
            crate::entity::damage::hurt_player(
                players,
                rak,
                addr,
                1.0,
                crate::entity::damage::DamageCause::Contact,
                false,
                None,
                None,
                Some(&msg),
            )
            .await;
        }

        // 7. Suffocation (solid full block at eye level)
        let (head_in_water, head_suffocating) = {
            let w = cur_world.read();
            let head_id = w.get_block_id_opt(px.floor() as i32, (py + 1.62).floor() as i32, pz.floor() as i32);
            (
                head_id == 8 || head_id == 9,
                crate::entity::physics::is_head_suffocating(&w, px, py + 1.62, pz),
            )
        };
        if head_suffocating {
            let msg = format!("{} suffocated in a wall", username);
            crate::entity::damage::hurt_player(
                players,
                rak,
                addr,
                1.0,
                crate::entity::damage::DamageCause::Suffocation,
                false,
                None,
                None,
                Some(&msg),
            )
            .await;
        }

        // 8. Water Drowning & Air Ticks (with Helmet Respiration enchantment)
        let mut drown_now = false;
        let mut air_meta_sync: Option<(i64, EntityMetadata)> = None;
        {
            let mut guard = players.write();
            if let Some(s) = guard.get_mut(&addr) {
                let helm = s.inventory.helmet();
                let resp_lvl = crate::item::enchantment::parse_enchantments_from_item(&helm)
                    .iter()
                    .find(|e| e.id == crate::item::enchantment::TYPE_WATER_BREATHING)
                    .map(|e| e.level as i32)
                    .unwrap_or(0);
                let max_air = (400 + resp_lvl * 300) as i16;
                let has_wb = s.effects.has_effect(crate::entity::effect::effect_id::WATER_BREATHING);

                if head_in_water && !has_wb {
                    s.metadata.set_generic_flag(33, false); // DATA_FLAG_BREATHING = false
                    s.metadata.set_short(43, max_air);
                    s.air_ticks -= 4;
                    if s.air_ticks <= -80 {
                        s.air_ticks = 0;
                        drown_now = true;
                    }
                    s.metadata.set_short(7, s.air_ticks.clamp(-80, max_air));
                    air_meta_sync = Some((s.entity_id, s.metadata.clone()));
                } else if s.air_ticks != max_air {
                    s.air_ticks = max_air;
                    s.metadata.set_generic_flag(33, true);
                    s.metadata.set_short(43, max_air);
                    s.metadata.set_short(7, max_air);
                    air_meta_sync = Some((s.entity_id, s.metadata.clone()));
                }
            }
        }
        if let Some((eid, meta)) = air_meta_sync {
            let dp = SetEntityDataPacket { entity_runtime_id: eid, metadata: meta };
            send_packet(rak, addr, dp.encode()).await;
        }
        if drown_now {
            let msg = format!("{} drowned", username);
            crate::entity::damage::hurt_player(
                players,
                rak,
                addr,
                2.0,
                crate::entity::damage::DamageCause::Drowning,
                false,
                None,
                None,
                Some(&msg),
            )
            .await;
        }
    }
}

async fn broadcast_player_positions(
    rak: &RakNetServer,
    players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
) {
    let snapshot: Vec<(SocketAddr, PlayerState)> = {
        let guard = players.read();
        guard.iter().map(|(a, s)| (*a, clone_state(s))).collect()
    };
    for (addr, state) in &snapshot {
        for (other_addr, other) in &snapshot {
            if other_addr == addr { continue; }
            if !other.logged_in || other.dimension != state.dimension { continue; }
            let mut pk = MovePlayerPacket::new(
                state.entity_id,
                state.x as f32,
                (state.y + 1.62) as f32,
                state.z as f32,
                state.yaw,
                state.pitch,
            );
            pk.head_yaw = state.yaw;
            pk.on_ground = state.on_ground;
            pk.mode = crate::protocol::packets::move_player::MODE_NORMAL;
            let batch = BatchPacket::encode(&[pk.encode()]);
            rak.send_encapsulated(*other_addr, batch, false).await;
        }
    }
}

fn clone_state(s: &PlayerState) -> PlayerState {
    let inv = crate::inventory::inventories::PlayerInventory::new();
    inv.set_contents(s.inventory.get_contents());
    inv.set_held_index(s.inventory.held_index());
    inv.set_hotbar_array(s.inventory.hotbar_array());
    inv.set_armor_contents(s.inventory.armor_contents());
    inv.set_offhand(s.inventory.offhand());
    PlayerState {
        username: s.username.clone(),
        uuid: s.uuid,
        entity_id: s.entity_id,
        skin_id: s.skin_id.clone(),
        skin: s.skin.clone(),
        client_id: s.client_id,
        device_os: s.device_os,
        x: s.x, y: s.y, z: s.z,
        yaw: s.yaw, pitch: s.pitch,
        on_ground: s.on_ground,
        fall_distance: s.fall_distance,
        gamemode: s.gamemode,
        allow_flight: s.allow_flight,
        is_flying: s.is_flying,
        sprinting: s.sprinting,
        sneaking: s.sneaking,
        gliding: s.gliding,
        health: s.health,
        food_level: s.food_level,
        food_saturation: s.food_saturation,
        exhaustion: s.exhaustion,
        air_ticks: s.air_ticks,
        fire_ticks: s.fire_ticks,
        no_damage_ticks: s.no_damage_ticks,
        xp_points: s.xp_points,
        xp_level: s.xp_level,
        xp_progress: s.xp_progress,
        metadata: s.metadata.clone(),
        chunk_radius: s.chunk_radius,
        logged_in: s.logged_in,
        spawned: s.spawned,
        sent_chunks: s.sent_chunks.clone(),
        inventory: inv,
        ender_chest: s.ender_chest.clone(),
        open_container: s.open_container,
        open_window_type: s.open_window_type,
        dimension: s.dimension,
        portal_ticks: s.portal_ticks,
        spawn_pos: s.spawn_pos,
        effects: s.effects.clone(),
        food_tick_timer: s.food_tick_timer,
        start_action_tick: s.start_action_tick,
        break_start: s.break_start,
        item_cooldowns: s.item_cooldowns.clone(),
        fishing_hook_eid: s.fishing_hook_eid,
        pending_drops: s.pending_drops.clone(),
        pending_xp_drop: s.pending_xp_drop,
    }
}

async fn event_loop(self_data: OxideServer) -> anyhow::Result<()> {
    loop {
        let events = self_data.rak.drain_events().await;
        if events.is_empty() {
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
            continue;
        }
        for ev in events {
            match ev {
                RakNetEvent::OpenSession(addr, client_id) => {
                    info!("Session opened: {} (client id {})", addr.get_ip(), client_id);
                }
                RakNetEvent::CloseSession(addr, reason) => {
                    info!("Session closed: {} ({})", addr.get_ip(), reason);
                    let sock_addr = addr.to_socket_addr();
                    let _ = self_data.tile_manager.close_temp_window(sock_addr);
                    let closed_chests = self_data.tile_manager.close_all_for_player(sock_addr);
                    let removed = self_data.players.write().remove(&sock_addr);
                    if let Some(state) = &removed {
                        if state.logged_in {
                            let _ = crate::level::storage::save_player_dat(&self_data.data_dir, state);
                        }
                        for ((cx, cy, cz), was_last) in closed_chests {
                            if was_last {
                                let block_ev = BlockEventPacket { x: cx, y: cy as u32, z: cz, case_1: 1, case_2: 0 };
                                let sound = LevelSoundEventPacket::new(level_event::SOUND_CHEST_CLOSED, cx as f32 + 0.5, cy as f32 + 0.5, cz as f32 + 0.5);
                                broadcast_in_dim(&self_data.rak, &self_data.players, state.dimension, None, block_ev.encode()).await;
                                broadcast_in_dim(&self_data.rak, &self_data.players, state.dimension, None, sound.encode()).await;
                            }
                        }
                        self_data.event_bus.fire(crate::event::event::PlayerQuitEvent {
                            player_name: state.username.clone(),
                            reason: reason.clone(),
                        });
                        if state.logged_in {
                            let pk = RemoveEntityPacket::new(state.entity_id);
                            broadcast_in_dim(&self_data.rak, &self_data.players, state.dimension, Some(sock_addr), pk.encode()).await;
                            let list_rm = PlayerListPacket::remove(vec![state.uuid]);
                            broadcast_except(&self_data.rak, &self_data.players, Some(sock_addr), list_rm.encode()).await;
                            let quit_msg = format!("{} left the game", state.username);
                            let msg_pk = TextPacket::system(quit_msg);
                            broadcast_except(&self_data.rak, &self_data.players, None, msg_pk.encode()).await;
                        }
                    }
                }
                RakNetEvent::Encapsulated(addr, payload) => {
                    if payload.is_empty() { continue; }
                    tracing::info!("RAKNET ENC id=0x{:02x} {} bytes from {}", payload[0], payload.len(), addr.get_ip());
                    let sock = addr.to_socket_addr();
                    if payload[0] == ProtocolInfo::BATCH_PACKET {
                        match BatchPacket::decode(&payload) {
                            Ok(inner) => {
                                tracing::info!("BATCH OK: {} inner packets", inner.len());
                                for pkt in inner {
                                    handle_packet(&self_data, sock, pkt).await;
                                }
                            }
                            Err(e) => warn!("batch decode failed from {}: {}", addr.get_ip(), e),
                        }
                    } else {
                        handle_packet(&self_data, sock, payload).await;
                    }
                }
                RakNetEvent::Raw(addr, data) => {
                    let _ = (addr, data);
                }
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
}

async fn handle_packet(self_data: &OxideServer, addr: SocketAddr, payload: Vec<u8>) {
    if payload.is_empty() { return; }
    let id = payload[0];
    let incoming = PacketHandler::handle(payload);
    match incoming {
        IncomingPacket::Login(login) => {
            if !ProtocolInfo::ACCEPTED_PROTOCOLS.contains(&(login.protocol as u32)) {
                let status = if login.protocol < ProtocolInfo::CURRENT_PROTOCOL as i32 {
                    play_status::LOGIN_FAILED_CLIENT
                } else {
                    play_status::LOGIN_FAILED_SERVER
                };
                warn!("Login REJECTED: username='{}' protocol={} (need one of {:?}). If 0/garbage → packet decode broken. status={}",
                    login.username, login.protocol, ProtocolInfo::ACCEPTED_PROTOCOLS, status);
                let pk = PlayStatusPacket::new(status);
                send_packet(&self_data.rak, addr, pk.encode()).await;
                let _ = self_data.rak.close_session(addr, "incompatible protocol".to_string()).await;
                return;
            }

            // Check player ban
            let lower_name = login.username.to_lowercase();
            if let Some(reason) = self_data.bans.read().get(&lower_name) {
                let disconnect = DisconnectPacket::new(format!("You are banned: {}", reason));
                send_packet(&self_data.rak, addr, disconnect.encode()).await;
                let _ = self_data.rak.close_session(addr, "banned".to_string()).await;
                return;
            }

            // Check IP ban
            let ip_str = addr.ip().to_string();
            if let Some(reason) = self_data.banned_ips.read().get(&ip_str) {
                let disconnect = DisconnectPacket::new(format!("Your IP is banned: {}", reason));
                send_packet(&self_data.rak, addr, disconnect.encode()).await;
                let _ = self_data.rak.close_session(addr, "ip banned".to_string()).await;
                return;
            }

            // Check whitelist
            if self_data.whitelist_enabled.load(std::sync::atomic::Ordering::SeqCst) {
                if !self_data.whitelist.read().contains(&lower_name) {
                    let disconnect = DisconnectPacket::new("Server is whitelisted".to_string());
                    send_packet(&self_data.rak, addr, disconnect.encode()).await;
                    let _ = self_data.rak.close_session(addr, "whitelist".to_string()).await;
                    return;
                }
            }
            let entity_id = {
                let mut eid = self_data.next_entity_id.write();
                let id = *eid;
                *eid += 1;
                id
            };
            let username = if login.username.is_empty() { "Player".to_string() } else { login.username.clone() };
            let uuid = UUID::from_string(&login.client_uuid).unwrap_or_else(|| UUID::random());
            info!("Player logging in: {} ({})", username, uuid);

            let mut state = PlayerState::new();
            state.username = username.clone();
            state.uuid = uuid;
            state.entity_id = entity_id;
            state.skin_id = login.skin_id.clone().filter(|s| !s.is_empty()).unwrap_or_else(|| "Standard_Custom".to_string());
            if let Some(skin_bytes) = login.skin.clone() {
                if skin_bytes.len() == 8192 || skin_bytes.len() == 16384 {
                    state.skin = skin_bytes;
                }
            }
            state.client_id = login.client_id;
            state.device_os = login.device_os.unwrap_or(0);
            state.metadata = default_player_metadata();

            if let Some(ld) = crate::level::storage::load_player_dat(&self_data.data_dir, &username) {
                state.x = ld.x;
                state.y = ld.y;
                state.z = ld.z;
                state.yaw = ld.yaw;
                state.pitch = ld.pitch;
                state.dimension = ld.dimension;
                state.gamemode = ld.gamemode;
                state.allow_flight = ld.gamemode == 1 || ld.gamemode == 3;
                state.health = ld.health;
                state.food_level = ld.food_level;
                state.food_saturation = ld.food_saturation;
                state.exhaustion = ld.exhaustion;
                state.air_ticks = ld.air_ticks;
                state.fire_ticks = ld.fire_ticks;
                state.xp_level = ld.xp_level;
                state.xp_progress = ld.xp_progress;
                state.xp_points = ld.xp_points;
                state.spawn_pos = ld.spawn_pos;
                state.inventory.set_contents(ld.inventory);
                state.inventory.set_armor_contents(ld.armor);
                state.inventory.set_offhand(ld.offhand);
                state.inventory.set_hotbar_array(ld.hotbar);
                state.inventory.set_held_index(ld.held_index);
                state.ender_chest = ld.ender_chest;
            } else {
                let default_gm = self_data
                    .config
                    .read()
                    .get("gamemode")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32;
                let (sx, sy, sz) = {
                    let w = self_data.world.read();
                    (w.spawn_x, w.spawn_y, w.spawn_z)
                };
                state.x = sx as f64 + 0.5;
                state.y = sy as f64;
                state.z = sz as f64 + 0.5;
                state.gamemode = default_gm;
                state.allow_flight = default_gm == 1 || default_gm == 3;
            }
            self_data.players.write().insert(addr, state);

            let success = PlayStatusPacket::new(play_status::LOGIN_SUCCESS);
            send_packet(&self_data.rak, addr, success.encode()).await;

            let rp_info = ResourcePacksInfoPacket::default();
            send_packet(&self_data.rak, addr, rp_info.encode()).await;
        }
        IncomingPacket::ResourcePackClientResponse(resp) => {
            info!("ResourcePackClientResponse: status={}", resp.status);
            match resp.status {
                resource_pack_client_response::STATUS_REFUSED => {
                    // Client refused resource packs, disconnect session
                    let _ = self_data.rak
                        .close_session(addr, "refused resource packs".to_string())
                        .await;
                    return;
                }
                resource_pack_client_response::STATUS_SEND_PACKS => {
                    // Server has no packs to download, ignore request
                    warn!("Client requested packs {:?}, but server has none", resp.pack_ids);
                    return;
                }
                resource_pack_client_response::STATUS_HAVE_ALL_PACKS => {
                    // Client confirmed having packs; send resource pack stack to proceed
                    let stack = ResourcePackStackPacket::default();
                    send_packet(&self_data.rak, addr, stack.encode()).await;
                    return;
                }
                _ => {}
            }
            if resp.status == resource_pack_client_response::STATUS_COMPLETED {
                if self_data.players.read().get(&addr).map(|s| s.spawned).unwrap_or(false) {
                    debug!("Duplicate COMPLETED from {} - player already spawned, skipping", addr);
                    return;
                }
                let state_opt = self_data.players.read().get(&addr).map(|s| (
                    s.entity_id,
                    s.username.clone(),
                    s.gamemode,
                    s.metadata.clone(),
                    s.chunk_radius,
                    s.uuid,
                    s.x,
                    s.y,
                    s.z,
                    s.yaw,
                    s.pitch,
                    s.dimension,
                    s.health,
                    s.food_level,
                    s.food_saturation,
                    s.xp_level,
                    s.xp_progress,
                    s.allow_flight,
                ));
                if let Some((
                    entity_id,
                    username,
                    gamemode,
                    metadata,
                    chunk_radius,
                    uuid,
                    px,
                    py,
                    pz,
                    yaw,
                    pitch,
                    dim,
                    hp,
                    food,
                    sat,
                    xp_lvl,
                    xp_prog,
                    allow_flight,
                )) = state_opt
                {
                    let (w_sx, w_sy, w_sz) = {
                        let w_arc = self_data.get_world(dim);
                        let w = w_arc.read();
                        (w.spawn_x, w.spawn_y, w.spawn_z)
                    };
                    let center_bx = px.floor() as i32;
                    let center_bz = pz.floor() as i32;

                    let start = StartGamePacket {
                        entity_unique_id: entity_id,
                        entity_runtime_id: entity_id,
                        player_gamemode: gamemode,
                        x: px as f32,
                        y: (py as f32) + 1.62,
                        z: pz as f32,
                        pitch,
                        yaw,
                        seed: 0,
                        dimension: dim,
                        generator: 1,
                        world_gamemode: gamemode,
                        difficulty: 1,
                        spawn_x: w_sx,
                        spawn_y: w_sy as u32,
                        spawn_z: w_sz,
                        has_achievements_disabled: true,
                        day_cycle_stop_time: -1,
                        edu_mode: false,
                        rain_level: 0.0,
                        lightning_level: 0.0,
                        commands_enabled: true,
                        is_texture_packs_required: false,
                        game_rules: GameRules::default_rules(),
                        level_id: String::new(),
                        world_name: "Oxide".to_string(),
                        premium_world_template_id: String::new(),
                        unknown_bool: false,
                        current_tick: 0,
                    };
                    send_packet(&self_data.rak, addr, start.encode()).await;

                    let time_pk = SetTimePacket::new(self_data.world.read().time);
                    send_packet(&self_data.rak, addr, time_pk.encode()).await;

                    let mut attrs = UpdateAttributesPacket::new(entity_id);
                    attrs.attributes = vec![
                        Attribute::health(hp as f32),
                        Attribute::movement_speed(0.1),
                        Attribute::hunger(food as f32),
                        Attribute::food_saturation(sat),
                        Attribute::experience_level(xp_lvl as f32),
                        Attribute::experience_progress(xp_prog),
                    ];
                    send_packet(&self_data.rak, addr, attrs.encode()).await;

                    let data_pk = SetEntityDataPacket::new(entity_id, metadata);
                    send_packet(&self_data.rak, addr, data_pk.encode()).await;

                    let mut settings = AdventureSettingsPacket::new();
                    settings.world_builder = true;
                    settings.allow_flight = allow_flight;
                    settings.user_permission = if self_data.is_op(&username) {
                        adventure_settings::PERMISSION_OPERATOR
                    } else {
                        adventure_settings::PERMISSION_NORMAL
                    };
                    send_packet(&self_data.rak, addr, settings.encode()).await;

                    let radius_pk = ChunkRadiusUpdatedPacket::new(chunk_radius);
                    send_packet(&self_data.rak, addr, radius_pk.encode()).await;

                    // Send initial spawn chunks around saved/spawn position
                    send_spawn_chunks(self_data, addr, center_bx, center_bz, chunk_radius.min(4)).await;

                    if let Some(state) = self_data.players.write().get_mut(&addr) {
                        state.logged_in = true;
                        state.spawned = true;
                        state.on_ground = true;
                    }

                    let spawn_pos_pk = SetSpawnPositionPacket::new_player(w_sx, w_sy, w_sz);
                    send_packet(&self_data.rak, addr, spawn_pos_pk.encode()).await;

                    let spawn_status = PlayStatusPacket::new(play_status::PLAYER_SPAWN);
                    send_packet(&self_data.rak, addr, spawn_status.encode()).await;

                    let available_cmds = self_data.command_manager.generate_available_commands_packet();
                    send_packet(&self_data.rak, addr, available_cmds.encode()).await;

                    self_data.weather.send_to_player(addr, &self_data.rak).await;

                    let crafting_pk = CraftingDataPacket::with_default_recipes();
                    send_packet(&self_data.rak, addr, crafting_pk.encode()).await;

                    let mut creative_pk = ContainerSetContentPacket::new(crate::protocol::types::ContainerIds::CREATIVE as u32);
                    creative_pk.target_eid = entity_id;
                    creative_pk.items = crate::item::creative::get_creative_items().to_vec();
                    send_packet(&self_data.rak, addr, creative_pk.encode()).await;

                    send_player_inventory(self_data, addr).await;

                    spawn_existing_players_to(self_data, addr).await;
                    announce_new_player_to_others(self_data, addr, entity_id, username.clone(), uuid).await;
                    broadcast_player_list_add(self_data).await;
                    self_data.entity_manager.send_entities_in_dim_to(addr, dim, &self_data.rak).await;

                    let join_msg = TextPacket::system(format!("{} joined the game", username));
                    broadcast_except(&self_data.rak, &self_data.players, None, join_msg.encode()).await;

                    self_data.event_bus.fire(crate::event::event::PlayerJoinEvent {
                        player_name: username.clone(),
                    });

                    if chunk_radius > 4 {
                        let server_clone = self_data.clone();
                        tokio::spawn(async move {
                            send_spawn_chunks(&server_clone, addr, center_bx, center_bz, chunk_radius).await;
                        });
                    }
                }
            }
        }
        IncomingPacket::RequestChunkRadius(req) => {
            // Allowed view distance: clamp between 2 and configured server view distance
            let max_view = {
                let cfg = self_data.config.read();
                let v = cfg.get_int("view_distance");
                if v > 0 { v as i32 } else { DEFAULT_VIEW_DISTANCE }
            };
            let radius = req.radius.min(max_view).max(2);
            info!("Requested chunk radius {} -> granted {}", req.radius, radius);
            let center_xz = {
                let mut guard = self_data.players.write();
                if let Some(state) = guard.get_mut(&addr) {
                    state.chunk_radius = radius;
                    Some((state.x as i32, state.z as i32))
                } else {
                    None
                }
            };
            let pk = ChunkRadiusUpdatedPacket::new(radius);
            send_packet(&self_data.rak, addr, pk.encode()).await;
            let (sx, sz) = center_xz.unwrap_or_else(|| {
                let w = self_data.world.read();
                (w.spawn_x, w.spawn_z)
            });
            let server_clone = self_data.clone();
            tokio::spawn(async move {
                send_spawn_chunks(&server_clone, addr, sx, sz, radius).await;
            });
        }
        IncomingPacket::MovePlayer(mv) => {
            let mut chunk_to_load = None;
            let mut fall_dist_hit = 0.0f32;
            let mut fall_player_name = String::new();
            let mut fall_land_pos = (0.0f64, 0.0f64, 0.0f64);
            let mut eid = 0;
            let mut moved = false;
            let mut player_dim = 0;
            let mut hunger_attr_sync: Option<(i64, u32, f32)> = None;
            if let Some(state) = self_data.players.write().get_mut(&addr) {
                let old_cx = (state.x as i32) >> 4;
                let old_cz = (state.z as i32) >> 4;
                let old_x = state.x;
                let old_y = state.y;
                let old_z = state.z;
                let was_on_ground = state.on_ground;
                eid = state.entity_id;
                player_dim = state.dimension;

                let dx = (mv.x as f64 - state.x).abs();
                let dy = (mv.y as f64 - 1.62 - state.y).abs();
                let dz = (mv.z as f64 - state.z).abs();
                let dyaw = (mv.yaw - state.yaw).abs();
                let dpitch = (mv.pitch - state.pitch).abs();
                if dx > 0.0001 || dy > 0.0001 || dz > 0.0001 || dyaw > 1.0 || dpitch > 1.0 {
                    moved = true;
                }

                state.x = mv.x as f64;
                state.y = (mv.y - 1.62) as f64;
                state.z = mv.z as f64;
                state.yaw = mv.yaw;
                state.pitch = mv.pitch;

                let horiz_dist = ((state.x - old_x).powi(2) + (state.z - old_z).powi(2)).sqrt() as f32;
                let mut ex_cost = if horiz_dist > 0.001 && horiz_dist < 10.0 {
                    if state.sprinting { horiz_dist * 0.1 } else { horiz_dist * 0.01 }
                } else {
                    0.0
                };
                if was_on_ground && !mv.on_ground && state.y > old_y + 0.15 && !state.is_flying {
                    ex_cost += if state.sprinting { 0.2 } else { 0.05 };
                }
                if ex_cost > 0.0 && exhaust_player(state, ex_cost) {
                    hunger_attr_sync = Some((state.entity_id, state.food_level, state.food_saturation));
                }

                if mv.on_ground {
                    if state.fall_distance > 3.0 && state.gamemode == 0 {
                        fall_dist_hit = state.fall_distance;
                        fall_player_name = state.username.clone();
                        fall_land_pos = (state.x, state.y, state.z);
                    }
                    state.fall_distance = 0.0;
                } else if state.y < old_y {
                    state.fall_distance += (old_y - state.y) as f32;
                }

                state.on_ground = mv.on_ground;
                let new_cx = (state.x as i32) >> 4;
                let new_cz = (state.z as i32) >> 4;
                if old_cx != new_cx || old_cz != new_cz {
                    chunk_to_load = Some((state.x as i32, state.z as i32, state.chunk_radius));
                }
            }
            if let Some((heid, hfood, hsat)) = hunger_attr_sync {
                let mut ap = UpdateAttributesPacket::new(heid);
                ap.attributes = vec![Attribute::hunger(hfood as f32), Attribute::food_saturation(hsat)];
                send_packet(&self_data.rak, addr, ap.encode()).await;
            }

            if moved {
                let bx = mv.x.floor() as i32;
                let by = (mv.y - 1.62).floor() as i32;
                let bz = mv.z.floor() as i32;
                let plate_res = {
                    let w_arc = self_data.get_world(player_dim);
                    let mut w = w_arc.write();
                    let bid = w.get_block_id_opt(bx, by, bz);
                    if matches!(bid, 70 | 72 | 147 | 148) && w.get_block_data_opt(bx, by, bz) == 0 {
                        w.set_block(bx, by, bz, bid, 1);
                        let mut rs = crate::level::redstone::update_redstone_around(&mut w, &[(bx, by, bz)]);
                        rs.block_changes.insert(0, (bx, by, bz, bid, 1));
                        Some((bid, rs))
                    } else {
                        None
                    }
                };
                if let Some((pid, rs)) = plate_res {
                    let cur_tick = self_data.server_tick.load(std::sync::atomic::Ordering::Relaxed);
                    self_data.redstone_sched.schedule_release(player_dim, bx, by, bz, cur_tick + 10, pid);
                    let click = LevelEventPacket::new(level_event::EVENT_SOUND_CLICK, bx as f32 + 0.5, by as f32 + 0.5, bz as f32 + 0.5, 0);
                    broadcast_in_dim(&self_data.rak, &self_data.players, player_dim, None, click.encode()).await;
                    apply_redstone_result(&self_data.rak, &self_data.players, &self_data.entity_manager, &self_data.next_entity_id, player_dim, rs).await;
                }
            }

            if moved {
                let mut out_mv = MovePlayerPacket::new(
                    eid,
                    mv.x,
                    mv.y,
                    mv.z,
                    mv.yaw,
                    mv.pitch,
                );
                out_mv.head_yaw = mv.head_yaw;
                out_mv.on_ground = mv.on_ground;
                out_mv.mode = crate::protocol::packets::move_player::MODE_NORMAL;
                let encoded = BatchPacket::encode(&[out_mv.encode()]);

                let players_guard = self_data.players.read();
                for (other_addr, other_state) in players_guard.iter() {
                    if *other_addr != addr && other_state.logged_in && other_state.dimension == player_dim {
                        let r = self_data.rak.clone();
                        let target_addr = *other_addr;
                        let payload = encoded.clone();
                        tokio::spawn(async move {
                            r.send_encapsulated(target_addr, payload, false).await;
                        });
                    }
                }
            }

            if fall_dist_hit > 3.0 {
                let (feet_id, below_id) = {
                    let w = self_data.get_world(player_dim);
                    let wg = w.read();
                    let bx = fall_land_pos.0.floor() as i32;
                    let by = fall_land_pos.1.floor() as i32;
                    let bz = fall_land_pos.2.floor() as i32;
                    (wg.get_block_id_opt(bx, by, bz), wg.get_block_id_opt(bx, by - 1, bz))
                };
                let safe_landing = matches!(feet_id, 8 | 9 | 30 | 165) || matches!(below_id, 8 | 9 | 30 | 165);
                if !safe_landing {
                    let mut raw_dmg = fall_dist_hit - 3.0;
                    if below_id == 170 {
                        raw_dmg *= 0.2;
                    }
                    if raw_dmg >= 1.0 {
                        let snd_id = if raw_dmg > 4.0 { level_event::SOUND_FALL_BIG } else { level_event::SOUND_FALL_SMALL };
                        let msg = format!("{} fell from a high place", fall_player_name);
                        crate::entity::damage::hurt_player(
                            &self_data.players,
                            &self_data.rak,
                            addr,
                            raw_dmg.floor(),
                            crate::entity::damage::DamageCause::Fall,
                            false,
                            None,
                            Some(snd_id),
                            Some(&msg),
                        )
                        .await;
                    }
                }
            }

            if let Some((px, pz, radius)) = chunk_to_load {
                let server_clone = self_data.clone();
                tokio::spawn(async move {
                    send_spawn_chunks(&server_clone, addr, px, pz, radius).await;
                });
            }
        }
        IncomingPacket::Text(txt) => {
            if txt.text_type == text::TYPE_CHAT {
                let clean = txt.message.replace('\u{00a7}', "").trim().to_string();
                if clean.starts_with('/') {
                    let cmd_line = clean[1..].trim();
                    let name = self_data.players.read().get(&addr).map(|s| s.username.clone()).unwrap_or_default();
                    let mut sender = crate::command::sender::PlayerSender::new(name, addr, true, self_data.clone());
                    let handled = self_data.command_manager.dispatch(&mut sender, self_data, cmd_line);
                    if !handled {
                        self_data.send_message_to(addr, &format!("Unknown command: /{}. Type /help for help.", cmd_line));
                    }
                } else {
                    let name = self_data.players.read().get(&addr).map(|s| s.username.clone()).unwrap_or_default();
                    let ev = self_data.event_bus.fire(crate::event::event::PlayerChatEvent {
                        player_name: name.clone(),
                        message: clean.clone(),
                        cancelled: false,
                    });
                    if !ev.cancelled {
                        let msg = format!("<{}> {}", name, clean);
                        info!("chat: {}", msg);
                        let pk = TextPacket::system(msg);
                        broadcast_except(&self_data.rak, &self_data.players, None, pk.encode()).await;
                    }
                }
            }
        }
        IncomingPacket::RemoveBlock(rb) => {
            handle_break_block(self_data, addr, rb.x, rb.y as i32, rb.z).await;
        }
        IncomingPacket::PlayerAction(action) => {
            let player_dim = self_data.players.read().get(&addr).map(|s| s.dimension).unwrap_or(0);
            match action.action {
                player_action::ACTION_START_BREAK => {
                    let cur_tick = self_data.server_tick.load(std::sync::atomic::Ordering::Relaxed);
                    if let Some(s) = self_data.players.write().get_mut(&addr) {
                        s.break_start = Some((action.x, action.y as i32, action.z, cur_tick));
                    }
                    let punch_pk = LevelEventPacket::new(
                        level_event::EVENT_PARTICLE_PUNCH_BLOCK,
                        action.x as f32 + 0.5,
                        action.y as f32 + 0.5,
                        action.z as f32 + 0.5,
                        action.face,
                    );
                    broadcast_in_dim(&self_data.rak, &self_data.players, player_dim, Some(addr), punch_pk.encode()).await;
                }
                player_action::ACTION_CONTINUE_BREAK => {
                    let punch_pk = LevelEventPacket::new(
                        level_event::EVENT_PARTICLE_PUNCH_BLOCK,
                        action.x as f32 + 0.5,
                        action.y as f32 + 0.5,
                        action.z as f32 + 0.5,
                        action.face,
                    );
                    broadcast_in_dim(&self_data.rak, &self_data.players, player_dim, Some(addr), punch_pk.encode()).await;
                }
                player_action::ACTION_ABORT_BREAK => {
                    if let Some(s) = self_data.players.write().get_mut(&addr) {
                        s.break_start = None;
                    }
                    let stop_pk = LevelEventPacket::new(
                        level_event::EVENT_BLOCK_STOP_BREAK,
                        action.x as f32 + 0.5,
                        action.y as f32 + 0.5,
                        action.z as f32 + 0.5,
                        0,
                    );
                    broadcast_in_dim(&self_data.rak, &self_data.players, player_dim, Some(addr), stop_pk.encode()).await;
                }
                player_action::ACTION_STOP_BREAK => {
                    if let Some(s) = self_data.players.write().get_mut(&addr) {
                        s.break_start = None;
                    }
                    handle_break_block(self_data, addr, action.x, action.y as i32, action.z).await;
                }
                player_action::ACTION_START_SPRINT => {
                    let (eid, meta) = {
                        let mut p = self_data.players.write();
                        if let Some(s) = p.get_mut(&addr) {
                            s.sprinting = true;
                            s.metadata.set_generic_flag(crate::protocol::types::metadata::DATA_FLAG_SPRINTING, true);
                            (s.entity_id, s.metadata.clone())
                        } else {
                            return;
                        }
                    };
                    let pk = SetEntityDataPacket::new(eid, meta);
                    broadcast_in_dim(&self_data.rak, &self_data.players, player_dim, None, pk.encode()).await;
                }
                player_action::ACTION_STOP_SPRINT => {
                    let (eid, meta) = {
                        let mut p = self_data.players.write();
                        if let Some(s) = p.get_mut(&addr) {
                            s.sprinting = false;
                            s.metadata.set_generic_flag(crate::protocol::types::metadata::DATA_FLAG_SPRINTING, false);
                            (s.entity_id, s.metadata.clone())
                        } else {
                            return;
                        }
                    };
                    let pk = SetEntityDataPacket::new(eid, meta);
                    broadcast_in_dim(&self_data.rak, &self_data.players, player_dim, None, pk.encode()).await;
                }
                player_action::ACTION_START_SNEAK => {
                    let (eid, meta) = {
                        let mut p = self_data.players.write();
                        if let Some(s) = p.get_mut(&addr) {
                            s.sneaking = true;
                            s.metadata.set_generic_flag(crate::protocol::types::metadata::DATA_FLAG_SNEAKING, true);
                            (s.entity_id, s.metadata.clone())
                        } else {
                            return;
                        }
                    };
                    let pk = SetEntityDataPacket::new(eid, meta);
                    broadcast_in_dim(&self_data.rak, &self_data.players, player_dim, None, pk.encode()).await;
                }
                player_action::ACTION_STOP_SNEAK => {
                    let (eid, meta) = {
                        let mut p = self_data.players.write();
                        if let Some(s) = p.get_mut(&addr) {
                            s.sneaking = false;
                            s.metadata.set_generic_flag(crate::protocol::types::metadata::DATA_FLAG_SNEAKING, false);
                            (s.entity_id, s.metadata.clone())
                        } else {
                            return;
                        }
                    };
                    let pk = SetEntityDataPacket::new(eid, meta);
                    broadcast_in_dim(&self_data.rak, &self_data.players, player_dim, None, pk.encode()).await;
                }
                player_action::ACTION_RELEASE_ITEM => {
                    let cur_tick = self_data.server_tick.load(std::sync::atomic::Ordering::Relaxed);
                    let release_info = {
                        let mut players = self_data.players.write();
                        if let Some(s) = players.get_mut(&addr) {
                            let start = s.start_action_tick.take();
                            s.metadata.set_generic_flag(crate::protocol::types::metadata::DATA_FLAG_ACTION, false);
                            let held = s.inventory.held_item();
                            let held_slot = s.inventory.held_slot();
                            Some((s.x, s.y, s.z, s.yaw, s.pitch, s.entity_id, s.dimension, s.gamemode == 0, held, held_slot, start, s.metadata.clone()))
                        } else {
                            None
                        }
                    };
                    if let Some((px, py, pz, yaw, pitch, eid, dim, is_survival, mut held_item, held_slot, start_opt, meta)) = release_info {
                        let meta_pk = SetEntityDataPacket::new(eid, meta);
                        broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, meta_pk.encode()).await;

                        if held_item.id == 261 {
                            if let Some(start_tick) = start_opt {
                                let diff = cur_tick.saturating_sub(start_tick);
                                let p = diff as f64 / 20.0;
                                let mut f = (p * p + p * 2.0) / 3.0;
                                if f > 1.0 {
                                    f = 1.0;
                                }
                                if f >= 0.1 && diff >= 5 {
                                    let enchs = crate::item::enchantment::parse_enchantments_from_nbt(&held_item.nbt);
                                    let ench_lvl = |id: u16| -> u16 {
                                        enchs.iter().find(|e| e.id == id).map(|e| e.level).unwrap_or(0)
                                    };
                                    let power = ench_lvl(crate::item::enchantment::TYPE_BOW_POWER);
                                    let punch = ench_lvl(crate::item::enchantment::TYPE_BOW_KNOCKBACK);
                                    let flame = ench_lvl(crate::item::enchantment::TYPE_BOW_FLAME);
                                    let infinity = ench_lvl(crate::item::enchantment::TYPE_BOW_INFINITY);
                                    let unbreaking = ench_lvl(crate::item::enchantment::TYPE_MINING_DURABILITY);

                                    let has_arrow = if is_survival && infinity == 0 {
                                        let mut guard = self_data.players.write();
                                        guard.get_mut(&addr).map(|s| s.inventory.remove_item(262, 1)).unwrap_or(false)
                                    } else {
                                        true
                                    };

                                    if has_arrow {
                                        if is_survival {
                                            let ignore_dmg = unbreaking > 0 && rand::random::<f32>() >= (1.0 / (unbreaking as f32 + 1.0));
                                            if !ignore_dmg {
                                                held_item.meta += 1;
                                                let broke = held_item.meta >= 385;
                                                {
                                                    let mut guard = self_data.players.write();
                                                    if let Some(s) = guard.get_mut(&addr) {
                                                        if broke {
                                                            s.inventory.set_item(held_slot, crate::protocol::types::Item::air());
                                                        } else {
                                                            s.inventory.set_item(held_slot, held_item);
                                                        }
                                                    }
                                                }
                                                if broke {
                                                    let brk_snd = LevelSoundEventPacket::new(level_event::SOUND_BREAK, px as f32, py as f32, pz as f32);
                                                    broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, brk_snd.encode()).await;
                                                }
                                            }
                                            send_player_inventory(self_data, addr).await;
                                        }

                                        let proj_eid = {
                                            let mut guard = self_data.next_entity_id.write();
                                            let id = *guard;
                                            *guard += 1;
                                            id
                                        };
                                        let damage = 2.0 + if power > 0 { 0.5 * (power as f32 + 1.0) } else { 0.0 };
                                        let fire_secs = if flame > 0 { 5 } else { 0 };
                                        self_data.projectile_manager.launch_configured(
                                            proj_eid,
                                            dim,
                                            crate::entity::ProjectileKind::Arrow,
                                            addr,
                                            eid,
                                            px,
                                            py,
                                            pz,
                                            yaw,
                                            pitch,
                                            f * 3.0,
                                            damage,
                                            punch as f32,
                                            fire_secs,
                                            f >= 1.0,
                                            None,
                                            &self_data.rak,
                                            &self_data.players,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                player_action::ACTION_SPAWN_SAME_DIMENSION | player_action::ACTION_SPAWN_OVERWORLD => {
                    let (sx, sy, sz) = {
                        let w = self_data.world.read();
                        (w.spawn_x, w.spawn_y, w.spawn_z)
                    };
                    let (eid, prev_dim) = {
                        let mut p = self_data.players.write();
                        if let Some(s) = p.get_mut(&addr) {
                            let old_d = s.dimension;
                            s.health = 20;
                            s.food_level = 20;
                            s.food_saturation = 5.0;
                            s.fire_ticks = 0;
                            s.fall_distance = 0.0;
                            s.no_damage_ticks = 40;
                            s.air_ticks = 300;
                            s.effects = crate::entity::effect::EffectContainer::new();
                            s.x = sx as f64 + 0.5;
                            s.y = sy as f64 + 0.5;
                            s.z = sz as f64 + 0.5;
                            (s.entity_id, old_d)
                        } else {
                            (0, 0)
                        }
                    };
                    if prev_dim != 0 {
                        change_player_dimension_raw(
                            &self_data.rak,
                            &self_data.players,
                            &self_data.world,
                            &self_data.nether_world,
                            &self_data.end_world,
                            addr,
                            0,
                            sx as f64 + 0.5,
                            sy as f64 + 0.5,
                            sz as f64 + 0.5,
                        )
                        .await;
                    }
                    let respawn_pk = RespawnPacket {
                        x: sx as f32 + 0.5,
                        y: sy as f32 + 0.5,
                        z: sz as f32 + 0.5,
                    };
                    send_packet(&self_data.rak, addr, respawn_pk.encode()).await;

                    let hp_pk = SetHealthPacket::new(20);
                    send_packet(&self_data.rak, addr, hp_pk.encode()).await;

                    let mv = MovePlayerPacket::new(eid, sx as f32 + 0.5, sy as f32 + 1.62, sz as f32 + 0.5, 0.0, 0.0);
                    send_packet(&self_data.rak, addr, mv.encode()).await;
                }
                _ => {}
            }
        }
        IncomingPacket::UseItem(ui) => {
            handle_use_item(self_data, addr, ui).await;
        }
        IncomingPacket::Interact(interact) => {
            if interact.action == crate::protocol::packets::interact::ACTION_RIGHT_CLICK {
                let info = self_data.players.read().get(&addr).map(|s| {
                    (s.entity_id, s.dimension, s.gamemode == 0, s.inventory.held_slot(), s.inventory.held_item(), s.x, s.y, s.z)
                });
                if let Some((peid, pdim, is_surv, hslot, held, px, py, pz)) = info {
                    if let Some(rc) = self_data
                        .entity_manager
                        .right_click_entity(
                            interact.target,
                            addr,
                            peid,
                            pdim,
                            &held,
                            &self_data.rak,
                            &self_data.players,
                            &self_data.next_entity_id,
                        )
                        .await
                    {
                        if is_surv && (rc.consume_held || rc.replace_held.is_some() || rc.damage_held) {
                            let new_item = if let Some(rep) = rc.replace_held {
                                rep
                            } else if rc.consume_held {
                                let mut it = held.clone();
                                if it.count > 1 { it.count -= 1; it } else { crate::protocol::types::Item::air() }
                            } else if rc.damage_held {
                                let mut it = held.clone();
                                let broke = crate::block::physics::apply_tool_damage(&mut it);
                                if broke {
                                    let snd = LevelSoundEventPacket::new(level_event::SOUND_BREAK, px as f32, py as f32, pz as f32);
                                    broadcast_in_dim(&self_data.rak, &self_data.players, pdim, None, snd.encode()).await;
                                    crate::protocol::types::Item::air()
                                } else {
                                    it
                                }
                            } else {
                                held
                            };
                            if let Some(s) = self_data.players.write().get_mut(&addr) {
                                s.inventory.set_item(hslot, new_item.clone());
                            }
                            let spk = ContainerSetSlotPacket::new(0, hslot as i32, new_item);
                            send_packet(&self_data.rak, addr, spk.encode()).await;
                        }
                    }
                }
            } else if interact.action == crate::protocol::packets::interact::ACTION_LEFT_CLICK {
                let (
                    held_slot,
                    held_item,
                    attacker_eid,
                    attacker_dim,
                    attacker_pos,
                    attacker_name,
                    is_critical,
                    is_survival,
                    strength_amp,
                    weakness_amp,
                ) = {
                    let p = self_data.players.read();
                    if let Some(s) = p.get(&addr) {
                        let slot = s.inventory.held_slot();
                        let crit = !s.on_ground && s.fall_distance > 0.0;
                        let str_a = s.effects.get_effect(crate::entity::effect::effect_id::STRENGTH).map(|e| e.amplifier + 1).unwrap_or(0);
                        let weak_a = s.effects.get_effect(crate::entity::effect::effect_id::WEAKNESS).map(|e| e.amplifier + 1).unwrap_or(0);
                        (
                            slot,
                            s.inventory.held_item(),
                            s.entity_id,
                            s.dimension,
                            Some((s.x, s.y, s.z)),
                            s.username.clone(),
                            crit,
                            s.gamemode == 0,
                            str_a,
                            weak_a,
                        )
                    } else {
                        return;
                    }
                };

                let base_damage = match held_item.id {
                    276 => 7.0,        // Diamond sword
                    267 | 279 => 6.0,  // Iron sword / Diamond axe
                    272 | 258 => 5.0,  // Stone sword / Iron axe
                    268 | 283 | 275 | 278 => 4.0, // Wood/Gold sword / Stone axe / Diamond pick
                    271 | 286 | 257 | 277 => 3.0, // Wood/Gold axe / Iron pick / Diamond shovel
                    274 | 256 => 2.0,  // Stone pick / Iron shovel
                    _ => 1.0,
                };
                let enchs = crate::item::enchantment::parse_enchantments_from_item(&held_item);
                let ench_bonus = crate::item::enchantment::calculate_melee_damage_bonus(&enchs, false, false);
                let kb_bonus = crate::item::enchantment::get_knockback_bonus(&enchs);
                let fire_secs = crate::item::enchantment::get_fire_aspect_seconds(&enchs);
                let potion_mod = (strength_amp as f32) * 3.0 - (weakness_amp as f32) * 4.0;
                let total_damage = ((base_damage + potion_mod).max(0.5) + ench_bonus)
                    * if is_critical { 1.5 } else { 1.0 };

                let mob_hit = self_data
                    .entity_manager
                    .damage_entity_ext(
                        interact.target,
                        total_damage,
                        attacker_eid,
                        attacker_pos,
                        fire_secs,
                        kb_bonus,
                        &self_data.rak,
                        &self_data.players,
                    )
                    .await;

                // PvP Player damage
                let target_player = {
                    let p = self_data.players.read();
                    p.iter()
                        .find(|(_, s)| s.entity_id == interact.target && s.dimension == attacker_dim)
                        .map(|(a, s)| (*a, s.entity_id, s.gamemode, s.username.clone(), s.x, s.y, s.z))
                };
                let mut pvp_hit = false;
                if let Some((target_addr, target_eid, gm, target_name, tx, ty, tz)) = target_player {
                    if gm == 0 && target_addr != addr {
                        pvp_hit = true;
                        if fire_secs > 0 {
                            if let Some(ts) = self_data.players.write().get_mut(&target_addr) {
                                ts.fire_ticks = ts.fire_ticks.max((fire_secs * 20) as i32);
                            }
                        }
                        let msg = format!("{} was slain by {}", target_name, attacker_name);
                        let outcome = crate::entity::damage::hurt_player(
                            &self_data.players,
                            &self_data.rak,
                            target_addr,
                            total_damage / if is_critical { 1.5 } else { 1.0 },
                            crate::entity::damage::DamageCause::EntityAttack,
                            is_critical,
                            attacker_pos,
                            Some(level_event::SOUND_HIT),
                            Some(&msg),
                        )
                        .await;

                        if kb_bonus > 0.0 && !outcome.died && outcome.final_damage > 0.0 {
                            if let Some((ax, _ay, az)) = attacker_pos {
                                let dx = tx - ax;
                                let dz = tz - az;
                                let len = (dx * dx + dz * dz).sqrt().max(0.01);
                                let mot = SetEntityMotionPacket::new(
                                    target_eid,
                                    (dx / len) as f32 * (0.4 * kb_bonus),
                                    0.3,
                                    (dz / len) as f32 * (0.4 * kb_bonus),
                                );
                                broadcast_in_dim(&self_data.rak, &self_data.players, attacker_dim, None, mot.encode()).await;
                            }
                        }
                        if outcome.thorns_damage > 0.0 {
                            let tmsg = format!("{} was killed trying to hurt {}", attacker_name, target_name);
                            crate::entity::damage::hurt_player(
                                &self_data.players,
                                &self_data.rak,
                                addr,
                                outcome.thorns_damage,
                                crate::entity::damage::DamageCause::Magic,
                                false,
                                Some((tx, ty, tz)),
                                None,
                                Some(&tmsg),
                            )
                            .await;
                        }
                    }
                }

                // Wear melee weapon durability in survival on successful hit
                if is_survival && (mob_hit || pvp_hit) && crate::block::physics::tool_spec(held_item.id).max_durability > 0 {
                    let mut updated = held_item.clone();
                    let broke = crate::block::physics::apply_tool_damage(&mut updated);
                    let out_it = if broke { crate::protocol::types::Item::air() } else { updated };
                    if let Some(s) = self_data.players.write().get_mut(&addr) {
                        s.inventory.set_item(held_slot, out_it.clone());
                    }
                    let slot_pk = ContainerSetSlotPacket::new(0, held_slot as i32, out_it);
                    send_packet(&self_data.rak, addr, slot_pk.encode()).await;
                    if broke {
                        let snd = LevelSoundEventPacket::new(
                            level_event::SOUND_BREAK,
                            attacker_pos.map(|p| p.0 as f32).unwrap_or(0.0),
                            attacker_pos.map(|p| p.1 as f32).unwrap_or(64.0),
                            attacker_pos.map(|p| p.2 as f32).unwrap_or(0.0),
                        );
                        broadcast_in_dim(&self_data.rak, &self_data.players, attacker_dim, None, snd.encode()).await;
                    }
                }
                if is_survival && (mob_hit || pvp_hit) {
                    let h_sync = {
                        let mut p = self_data.players.write();
                        p.get_mut(&addr).and_then(|s| {
                            if exhaust_player(s, 0.1) {
                                Some((s.entity_id, s.food_level, s.food_saturation))
                            } else {
                                None
                            }
                        })
                    };
                    if let Some((heid, hf, hs)) = h_sync {
                        let mut ap = UpdateAttributesPacket::new(heid);
                        ap.attributes = vec![Attribute::hunger(hf as f32), Attribute::food_saturation(hs)];
                        send_packet(&self_data.rak, addr, ap.encode()).await;
                    }
                }
            }
        }
        IncomingPacket::MobEquipment(me) => {
            let (eid, dim, window_id) = {
                let mut p = self_data.players.write();
                if let Some(state) = p.get_mut(&addr) {
                    if me.window_id == crate::protocol::types::ContainerIds::INVENTORY as u8 {
                        state.inventory.set_held_index_with_mapping(me.hotbar_slot as usize, me.inventory_slot as i32);
                    } else if me.window_id == crate::protocol::types::ContainerIds::OFFHAND as u8 {
                        state.inventory.set_offhand(me.item.clone());
                    }
                    (state.entity_id, state.dimension, me.window_id)
                } else {
                    return;
                }
            };
            let pk = MobEquipmentPacket {
                entity_runtime_id: eid,
                item: me.item,
                inventory_slot: me.inventory_slot,
                hotbar_slot: me.hotbar_slot,
                window_id,
            };
            broadcast_in_dim(&self_data.rak, &self_data.players, dim, Some(addr), pk.encode()).await;
        }
        IncomingPacket::ContainerSetSlot(slot_pk) => {
            let (dim, open_pos, open_type) = {
                let p = self_data.players.read();
                p.get(&addr)
                    .map(|s| (s.dimension, s.open_container, s.open_window_type))
                    .unwrap_or((0, None, None))
            };
            if slot_pk.window_id == 0 {
                let slot = slot_pk.slot as usize;
                if let Some(state) = self_data.players.write().get_mut(&addr) {
                    if slot < 36 {
                        state.inventory.set_item(slot, slot_pk.item);
                    }
                }
            } else if slot_pk.window_id == crate::protocol::types::ContainerIds::ARMOR as u8 {
                let slot = slot_pk.slot as usize;
                let armor_sync = {
                    let mut players = self_data.players.write();
                    if let Some(state) = players.get_mut(&addr) {
                        if slot < 4 {
                            state.inventory.set_armor(slot, slot_pk.item);
                        }
                        let mut arm = MobArmorEquipmentPacket::new(state.entity_id);
                        arm.helmet = state.inventory.helmet();
                        arm.chestplate = state.inventory.chestplate();
                        arm.leggings = state.inventory.leggings();
                        arm.boots = state.inventory.boots();
                        Some(arm)
                    } else {
                        None
                    }
                };
                if let Some(arm) = armor_sync {
                    broadcast_in_dim(&self_data.rak, &self_data.players, dim, Some(addr), arm.encode()).await;
                }
            } else if slot_pk.window_id == crate::protocol::types::ContainerIds::OFFHAND as u8 {
                if let Some(state) = self_data.players.write().get_mut(&addr) {
                    state.inventory.set_offhand(slot_pk.item);
                }
            } else if slot_pk.window_id == 1 {
                let slot = slot_pk.slot as usize;
                if open_type == Some(crate::protocol::types::WindowTypes::CONTAINER) && open_pos.is_none() {
                    // Ender Chest
                    if let Some(state) = self_data.players.write().get_mut(&addr) {
                        if slot < state.ender_chest.len() {
                            state.ender_chest[slot] = slot_pk.item;
                        }
                    }
                } else if open_type == Some(crate::protocol::types::WindowTypes::ENCHANTMENT) {
                    if let Some(temp) = self_data.tile_manager.get_temp_window(addr) {
                        let new_enchs = crate::item::enchantment::parse_enchantments_from_item(&slot_pk.item);
                        if slot == 0 && !new_enchs.is_empty() {
                            let before = temp.slots.get(0).cloned().unwrap_or_else(crate::protocol::types::Item::air);
                            let mut lapis = temp.slots.get(1).cloned().unwrap_or_else(crate::protocol::types::Item::air);
                            let before_enchs = crate::item::enchantment::parse_enchantments_from_item(&before);
                            let valid_item = !before.is_air()
                                && before_enchs.is_empty()
                                && (slot_pk.item.id == before.id || (before.id == 340 && slot_pk.item.id == 403));
                            let cost_tier = (new_enchs.len() as u32).clamp(1, 3);
                            let has_lapis = lapis.id == 351 && lapis.meta == 4 && lapis.count >= cost_tier;
                            let _bookshelves = {
                                let w_arc = self_data.get_world(temp.dim);
                                let mut w = w_arc.write();
                                crate::item::enchantment::count_bookshelves_around(&mut w, temp.pos.0, temp.pos.1, temp.pos.2)
                            };
                            let xp_update = {
                                let mut p = self_data.players.write();
                                if let Some(s) = p.get_mut(&addr) {
                                    let can_pay = s.gamemode == 1 || (valid_item && has_lapis && s.xp_level >= cost_tier);
                                    if can_pay {
                                        if s.gamemode != 1 {
                                            s.xp_level = s.xp_level.saturating_sub(cost_tier);
                                        }
                                        Some((s.entity_id, s.xp_level as f32, s.xp_progress, s.gamemode == 1))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            };
                            if let Some((eid, lvl, prog, creative)) = xp_update {
                                if !creative {
                                    if lapis.count > cost_tier {
                                        lapis.count -= cost_tier;
                                    } else {
                                        lapis = crate::protocol::types::Item::air();
                                    }
                                    self_data.tile_manager.set_temp_slot(addr, 1, lapis.clone());
                                    let lapis_pk = ContainerSetSlotPacket::new(1, 1, lapis);
                                    send_packet(&self_data.rak, addr, lapis_pk.encode()).await;
                                }
                                let mut result_item = slot_pk.item.clone();
                                if before.id == 340 {
                                    result_item.id = 403;
                                }
                                self_data.tile_manager.set_temp_slot(addr, 0, result_item.clone());
                                let res_pk = ContainerSetSlotPacket::new(1, 0, result_item);
                                send_packet(&self_data.rak, addr, res_pk.encode()).await;

                                let mut attrs = UpdateAttributesPacket::new(eid);
                                attrs.attributes = vec![
                                    Attribute::experience_level(lvl),
                                    Attribute::experience_progress(prog),
                                ];
                                send_packet(&self_data.rak, addr, attrs.encode()).await;
                            } else {
                                let revert_pk = ContainerSetSlotPacket::new(1, 0, before);
                                send_packet(&self_data.rak, addr, revert_pk.encode()).await;
                            }
                        } else {
                            self_data.tile_manager.set_temp_slot(addr, slot, slot_pk.item);
                        }
                    }
                } else if open_type == Some(crate::protocol::types::WindowTypes::ANVIL) {
                    if slot < 2 {
                        self_data.tile_manager.set_temp_slot(addr, slot, slot_pk.item);
                    }
                } else if let Some((cx, cy, cz)) = open_pos {
                    let world_arc = self_data.get_world(dim);
                    let block_id = { world_arc.read().get_block_id_opt(cx, cy, cz) };
                    if block_id == 54 || block_id == 146 {
                        let viewers = self_data.tile_manager.set_chest_slot_in_dim(dim, cx, cy, cz, slot, slot_pk.item.clone());
                        let sync_pk = ContainerSetSlotPacket::new(1, slot_pk.slot, slot_pk.item);
                        for viewer in viewers {
                            if viewer != addr {
                                self_data.send_packet_to(viewer, sync_pk.encode());
                            }
                        }
                    } else if block_id == 61 || block_id == 62 {
                        if slot == 2 && slot_pk.item.is_air() {
                            let fxp = self_data.tile_manager.take_furnace_result_xp_in_dim(dim, cx, cy, cz);
                            if fxp > 0 {
                                award_player_xp(self_data, addr, fxp).await;
                            }
                        }
                        let viewers = self_data.tile_manager.set_furnace_slot_in_dim(dim, cx, cy, cz, slot, slot_pk.item.clone());
                        let sync_pk = ContainerSetSlotPacket::new(1, slot_pk.slot, slot_pk.item);
                        for viewer in viewers {
                            if viewer != addr {
                                self_data.send_packet_to(viewer, sync_pk.encode());
                            }
                        }
                    } else if block_id == 117 {
                        let viewers = self_data.tile_manager.set_brewing_slot_in_dim(dim, cx, cy, cz, slot, slot_pk.item.clone());
                        let sync_pk = ContainerSetSlotPacket::new(1, slot_pk.slot, slot_pk.item);
                        for viewer in viewers {
                            if viewer != addr {
                                self_data.send_packet_to(viewer, sync_pk.encode());
                            }
                        }
                    }
                }
            }
        }
        IncomingPacket::ContainerClose(close_pk) => {
            if close_pk.window_id == 1 {
                let returned_temp = self_data.tile_manager.close_temp_window(addr);
                let mut had_leftovers = false;
                let dim = {
                    let mut p = self_data.players.write();
                    if let Some(s) = p.get_mut(&addr) {
                        s.open_container = None;
                        s.open_window_type = None;
                        for it in returned_temp {
                            let _ = s.inventory.add_item(it);
                            had_leftovers = true;
                        }
                        s.dimension
                    } else {
                        0
                    }
                };
                if had_leftovers {
                    send_player_inventory(self_data, addr).await;
                }
                let closed_chests = self_data.tile_manager.close_all_for_player(addr);
                for ((cx, cy, cz), was_last) in closed_chests {
                    if was_last {
                        let block_ev = BlockEventPacket { x: cx, y: cy as u32, z: cz, case_1: 1, case_2: 0 };
                        let sound = LevelSoundEventPacket::new(level_event::SOUND_CHEST_CLOSED, cx as f32 + 0.5, cy as f32 + 0.5, cz as f32 + 0.5);
                        broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, block_ev.encode()).await;
                        broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, sound.encode()).await;
                    }
                }
            }
        }
        IncomingPacket::CraftingEvent(craft_pk) => {
            let open_type = self_data.players.read().get(&addr).and_then(|s| s.open_window_type);
            if open_type == Some(crate::protocol::types::WindowTypes::ANVIL) {
                if let Some(temp) = self_data.tile_manager.get_temp_window(addr) {
                    let target = temp.slots.get(0).cloned().unwrap_or_else(crate::protocol::types::Item::air);
                    let sacrifice = temp.slots.get(1).cloned().unwrap_or_else(crate::protocol::types::Item::air);
                    let custom_name = craft_pk
                        .result
                        .first()
                        .and_then(|r| crate::item::enchantment::parse_item_custom_name(&r.nbt));
                    let combined = crate::item::enchantment::anvil_combine_items(
                        &target,
                        &sacrifice,
                        custom_name.as_deref(),
                    );
                    if let Some((out_item, xp_cost)) = combined {
                        let ok = {
                            let mut p = self_data.players.write();
                            if let Some(s) = p.get_mut(&addr) {
                                if s.gamemode == 1 || s.xp_level >= xp_cost {
                                    if s.gamemode != 1 {
                                        s.xp_level -= xp_cost;
                                    }
                                    let _ = s.inventory.add_item(out_item);
                                    Some((s.entity_id, s.xp_level as f32, s.xp_progress))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        };
                        if let Some((eid, lvl, prog)) = ok {
                            self_data.tile_manager.clear_temp_slots(addr);
                            send_player_inventory(self_data, addr).await;
                            let mut attrs = UpdateAttributesPacket::new(eid);
                            attrs.attributes = vec![
                                Attribute::experience_level(lvl),
                                Attribute::experience_progress(prog),
                            ];
                            send_packet(&self_data.rak, addr, attrs.encode()).await;

                            let use_snd = LevelEventPacket::new(
                                level_event::EVENT_SOUND_ANVIL_USE,
                                temp.pos.0 as f32 + 0.5,
                                temp.pos.1 as f32 + 0.5,
                                temp.pos.2 as f32 + 0.5,
                                0,
                            );
                            broadcast_in_dim(&self_data.rak, &self_data.players, temp.dim, None, use_snd.encode()).await;

                            if rand::random::<f32>() < 0.12 {
                                let (ax, ay, az) = temp.pos;
                                let (nid, nmeta, broke) = {
                                    let w_arc = self_data.get_world(temp.dim);
                                    let mut w = w_arc.write();
                                    let bid = w.get_block_id(ax, ay, az);
                                    if bid == 145 {
                                        let m = w.get_block_data_opt(ax, ay, az);
                                        let next = m.saturating_add(4);
                                        if next >= 12 {
                                            w.set_block(ax, ay, az, 0, 0);
                                            (0u8, 0u8, true)
                                        } else {
                                            w.set_block(ax, ay, az, 145, next);
                                            (145u8, next, false)
                                        }
                                    } else {
                                        (bid, 0, false)
                                    }
                                };
                                let bpk = UpdateBlockPacket::new(ax, ay as u32, az, nid as u32, nmeta as u32, update_block::FLAG_ALL);
                                broadcast_in_dim(&self_data.rak, &self_data.players, temp.dim, None, bpk.encode()).await;
                                if broke {
                                    let brk_snd = LevelEventPacket::new(
                                        level_event::EVENT_SOUND_ANVIL_BREAK,
                                        ax as f32 + 0.5,
                                        ay as f32 + 0.5,
                                        az as f32 + 0.5,
                                        0,
                                    );
                                    broadcast_in_dim(&self_data.rak, &self_data.players, temp.dim, None, brk_snd.encode()).await;
                                }
                            }
                            return;
                        }
                    }
                    send_player_inventory(self_data, addr).await;
                    return;
                }
            }

            let mut can_craft = true;
            let mut inputs = Vec::new();
            for inp in &craft_pk.input {
                if !inp.is_air() && inp.count > 0 {
                    inputs.push(inp.clone());
                }
            }

            {
                let mut players = self_data.players.write();
                if let Some(state) = players.get_mut(&addr) {
                    if state.gamemode == 0 && !inputs.is_empty() {
                        for inp in &inputs {
                            if !state.inventory.contains_item_meta(inp.id, inp.meta, inp.count) {
                                can_craft = false;
                                break;
                            }
                        }
                        if can_craft {
                            for inp in &inputs {
                                let _ = state.inventory.remove_item_meta(inp.id, inp.meta, inp.count);
                            }
                        }
                    }
                    if can_craft {
                        for out in &craft_pk.result {
                            let _ = state.inventory.add_item(out.clone());
                        }
                    }
                }
            }
            send_player_inventory(self_data, addr).await;
        }
        IncomingPacket::Animate(anim) => {
            let (eid, dim) = self_data.players.read().get(&addr)
                .map(|s| (s.entity_id, s.dimension))
                .unwrap_or((0, 0));
            let mut pk = anim;
            pk.entity_runtime_id = eid;
            broadcast_in_dim(&self_data.rak, &self_data.players, dim, Some(addr), pk.encode()).await;
        }
        IncomingPacket::DropItem(drop_pk) => {
            if !drop_pk.item.is_air() {
                let eid = {
                    let mut id_guard = self_data.next_entity_id.write();
                    let id = *id_guard;
                    *id_guard += 1;
                    id
                };
                let player_info = self_data.players.read().get(&addr).map(|s| (s.dimension, s.x, s.y, s.z, s.yaw, s.pitch, s.gamemode == 0, s.inventory.held_slot()));
                if let Some((dim, px, py, pz, _yaw, _pitch, is_survival, held_slot)) = player_info {
                    if is_survival {
                        let updated_item = {
                            let mut p = self_data.players.write();
                            if let Some(s) = p.get_mut(&addr) {
                                let mut cur = s.inventory.get_item(held_slot);
                                if cur.id == drop_pk.item.id && cur.count >= drop_pk.item.count {
                                    cur.count -= drop_pk.item.count;
                                    if cur.count == 0 {
                                        cur = crate::protocol::types::Item::air();
                                    }
                                    s.inventory.set_item(held_slot, cur.clone());
                                    Some(cur)
                                } else {
                                    let _ = s.inventory.remove_item_meta(drop_pk.item.id, drop_pk.item.meta, drop_pk.item.count);
                                    None
                                }
                            } else {
                                None
                            }
                        };
                        if let Some(it) = updated_item {
                            let slot_pk = ContainerSetSlotPacket::new(0, held_slot as i32, it);
                            send_packet(&self_data.rak, addr, slot_pk.encode()).await;
                        } else {
                            send_player_inventory(self_data, addr).await;
                        }
                    }

                    self_data.entity_manager.spawn_item_drop_in_dim(
                        eid,
                        dim,
                        drop_pk.item.clone(),
                        px,
                        py + 1.3,
                        pz,
                        &self_data.rak,
                        &self_data.players,
                    ).await;
                }
            }
        }
        IncomingPacket::PlayerInput(pi) => {
            let (eid, dim, meta) = {
                let mut p = self_data.players.write();
                if let Some(state) = p.get_mut(&addr) {
                    state.sneaking = pi.sneaking;
                    state.metadata.set_generic_flag(EntityMetadata::DATA_FLAG_SNEAKING, pi.sneaking);
                    (state.entity_id, state.dimension, state.metadata.clone())
                } else {
                    return;
                }
            };
            let pk = SetEntityDataPacket::new(eid, meta);
            broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, pk.encode()).await;
        }
        IncomingPacket::CommandStep(cs) => {
            let mut full_cmd = cs.command.trim().to_string();
            if !cs.input_json.is_empty() {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&cs.input_json) {
                    fn extract_strings(v: &serde_json::Value, list: &mut Vec<String>) {
                        match v {
                            serde_json::Value::String(s) => {
                                let t = s.trim();
                                if !t.is_empty() { list.push(t.to_string()); }
                            }
                            serde_json::Value::Number(n) => {
                                list.push(n.to_string());
                            }
                            serde_json::Value::Bool(b) => {
                                list.push(b.to_string());
                            }
                            serde_json::Value::Array(arr) => {
                                for item in arr { extract_strings(item, list); }
                            }
                            serde_json::Value::Object(map) => {
                                if let Some(t) = map.get("text").and_then(|x| x.as_str()) {
                                    let trimmed = t.trim();
                                    if !trimmed.is_empty() { list.push(trimmed.to_string()); }
                                } else if let Some(args) = map.get("args") {
                                    extract_strings(args, list);
                                } else {
                                    for (_k, sub) in map { extract_strings(sub, list); }
                                }
                            }
                            _ => {}
                        }
                    }
                    let mut extracted = Vec::new();
                    extract_strings(&val, &mut extracted);
                    if !extracted.is_empty() {
                        full_cmd.push(' ');
                        full_cmd.push_str(&extracted.join(" "));
                    }
                }
            }
            let clean_cmd = full_cmd.strip_prefix('/').unwrap_or(&full_cmd).trim();
            debug!("CommandStep from {}: '{}' (raw: '{}', json: '{}')", addr, clean_cmd, cs.command, cs.input_json);
            let name = self_data.players.read().get(&addr).map(|s| s.username.clone()).unwrap_or_default();
            let mut sender = crate::command::sender::PlayerSender::new(name, addr, true, self_data.clone());
            let handled = self_data.command_manager.dispatch(&mut sender, self_data, clean_cmd);
            if !handled {
                self_data.send_message_to(addr, &format!("Unknown command: /{}. Type /help for help.", clean_cmd));
            }
        }
        IncomingPacket::BlockPickRequest(bp) => {
            let dim = self_data.players.read().get(&addr).map(|s| s.dimension).unwrap_or(0);
            let world_arc = self_data.get_world(dim);
            let (block_id, block_meta) = {
                let mut w = world_arc.write();
                let id = w.get_block_id(bp.x, bp.y as i32, bp.z);
                let chunk = w.get_chunk_or_generate(bp.x >> 4, bp.z >> 4);
                let meta = chunk.get_block_data((bp.x & 0x0f) as usize, bp.y as usize, (bp.z & 0x0f) as usize);
                (id, meta)
            };
            if block_id != 0 {
                let item = crate::protocol::types::Item::new(block_id as i32, block_meta as i32, 1);
                let held_slot = {
                    let mut players = self_data.players.write();
                    if let Some(s) = players.get_mut(&addr) {
                        let slot = s.inventory.held_slot();
                        s.inventory.set_item(slot, item.clone());
                        slot
                    } else {
                        0
                    }
                };
                let slot_pk = ContainerSetSlotPacket::new(0, held_slot as i32, item);
                send_packet(&self_data.rak, addr, slot_pk.encode()).await;
            }
        }
        IncomingPacket::BlockEntityData(bed) => {
            let (dim, uname) = self_data
                .players
                .read()
                .get(&addr)
                .map(|s| (s.dimension, s.username.clone()))
                .unwrap_or((0, String::new()));
            if let Some(lines) = crate::tile::manager::parse_sign_nbt_lines(&bed.namedtag) {
                self_data.tile_manager.set_sign_text_in_dim(dim, bed.x, bed.y as i32, bed.z, lines, uname);
            }
            broadcast_in_dim(&self_data.rak, &self_data.players, dim, Some(addr), bed.encode()).await;
        }
        IncomingPacket::AdventureSettings(adv) => {
            let mut needs_sync = false;
            let (gamemode, allow_flight, mut out_settings) = {
                let mut players = self_data.players.write();
                if let Some(s) = players.get_mut(&addr) {
                    if adv.is_flying && !s.allow_flight && s.gamemode != 1 && s.gamemode != 3 {
                        needs_sync = true;
                        s.is_flying = false;
                    } else {
                        s.is_flying = adv.is_flying;
                    }
                    (s.gamemode, s.allow_flight, adv.clone())
                } else {
                    return;
                }
            };
            if needs_sync {
                out_settings.is_flying = false;
                out_settings.allow_flight = allow_flight || gamemode == 1 || gamemode == 3;
                send_packet(&self_data.rak, addr, out_settings.encode()).await;
            }
        }
        IncomingPacket::EntityEvent(ee) => {
            let (eid, dim) = self_data.players.read().get(&addr)
                .map(|s| (s.entity_id, s.dimension))
                .unwrap_or((0, 0));
            let mut out = ee;
            out.entity_runtime_id = eid;
            broadcast_in_dim(&self_data.rak, &self_data.players, dim, Some(addr), out.encode()).await;
        }
        IncomingPacket::EntityFall(ef) => {
            if let Some(s) = self_data.players.write().get_mut(&addr) {
                if ef.fall_distance > s.fall_distance {
                    s.fall_distance = ef.fall_distance;
                }
            }
        }
        IncomingPacket::SetPlayerGameType(gt) => {
            let is_op = {
                let name = self_data.players.read().get(&addr).map(|s| s.username.clone()).unwrap_or_default();
                !name.is_empty() && self_data.permission_manager.get(&format!("op.{}", name.to_lowercase())).is_some()
            };
            if is_op {
                self_data.set_player_gamemode(addr, gt.gamemode);
            } else {
                let cur_gm = self_data.players.read().get(&addr).map(|s| s.gamemode).unwrap_or(0);
                let revert = SetPlayerGameTypePacket::new(cur_gm);
                send_packet(&self_data.rak, addr, revert.encode()).await;
            }
        }
        IncomingPacket::ItemFrameDropItem(ifd) => {
            debug!("ItemFrameDropItem at ({}, {}, {}) from {}", ifd.x, ifd.y, ifd.z, addr);
        }
        IncomingPacket::Unknown(_, _) => {
            let name = crate::protocol::pool::PacketPool::name(id);
            let ip = self_data.players.read().get(&addr).map(|s| s.username.clone()).unwrap_or_default();
            warn!("unhandled packet {} ({:#x}) from {}", name, id, ip);
        }
    }
}

async fn handle_break_block(self_data: &OxideServer, addr: SocketAddr, x: i32, y: i32, z: i32) {
    if y < 0 || y >= 256 { return; }

    let (px, py, pz, dim, player_name, is_survival, held_slot, held_item) = match self_data.players.read().get(&addr) {
        Some(s) => (
            s.x, s.y, s.z, s.dimension, s.username.clone(), s.gamemode == 0,
            s.inventory.held_slot(), s.inventory.held_item(),
        ),
        None => return,
    };

    let dx = px - (x as f64 + 0.5);
    let dy = py - (y as f64 + 0.5);
    let dz = pz - (z as f64 + 0.5);
    if dx * dx + dy * dy + dz * dz > 64.0 {
        return;
    }

    let world_arc = self_data.get_world(dim);

    let (old_id, old_meta, extra_changes, cascade_drops, rs_res) = {
        let mut world = world_arc.write();
        let id = world.get_block_id(x, y, z);
        if id == 0 || crate::block::physics::block_hardness(id) < 0.0 {
            return;
        }
        let chunk_x = x >> 4;
        let chunk_z = z >> 4;
        let cx = (x & 0x0f) as usize;
        let cz = (z & 0x0f) as usize;
        let chunk = world.get_chunk_or_generate(chunk_x, chunk_z);
        let meta = chunk.get_block_data(cx, y as usize, cz);

        let ev = self_data.event_bus.fire(crate::event::event::BlockBreakEvent {
            player_name: player_name.clone(),
            x,
            y: y as u32,
            z,
            block_id: id as u16,
            cancelled: false,
        });
        if ev.cancelled {
            let revert_pk = UpdateBlockPacket::new(x, y as u32, z, id as u32, meta as u32, update_block::FLAG_ALL);
            let rak = self_data.rak.clone();
            tokio::spawn(async move {
                send_packet(&rak, addr, revert_pk.encode()).await;
            });
            return;
        }
        world.set_block(x, y, z, 0, 0);
        let (cascaded, c_drops) = crate::block::physics::cascade_neighbor_breaks(&mut world, &[(x, y, z)]);
        crate::level::light::update_light_around(&mut world, x, y, z);
        let rs = crate::level::redstone::update_redstone_around(&mut world, &[(x, y, z)]);
        (id, meta, cascaded, c_drops, rs)
    };
    apply_redstone_result(&self_data.rak, &self_data.players, &self_data.entity_manager, &self_data.next_entity_id, dim, rs_res).await;

    let (tile_drops, unpaired_partner) = self_data.tile_manager.remove_tile_in_dim(dim, x, y, z);
    if let Some((px, py, pz)) = unpaired_partner {
        let nbt = self_data.tile_manager.build_chest_nbt_in_dim(dim, px, py, pz);
        let bed_pk = BlockEntityDataPacket { x: px, y: py as u32, z: pz, namedtag: nbt };
        broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, bed_pk.encode()).await;
    }
    for drop_it in tile_drops {
        let drop_eid = {
            let mut eid_guard = self_data.next_entity_id.write();
            let id = *eid_guard;
            *eid_guard += 1;
            id
        };
        self_data.entity_manager.spawn_item_drop_in_dim(
            drop_eid,
            dim,
            drop_it,
            x as f64 + 0.5,
            y as f64 + 0.5,
            z as f64 + 0.5,
            &self_data.rak,
            &self_data.players,
        ).await;
    }

    let pk = UpdateBlockPacket::new(x, y as u32, z, 0, 0, update_block::FLAG_ALL);
    broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, pk.encode()).await;

    let destroy_pk = LevelEventPacket::new(
        level_event::EVENT_PARTICLE_DESTROY,
        x as f32 + 0.5,
        y as f32 + 0.5,
        z as f32 + 0.5,
        (old_id as i32) | ((old_meta as i32) << 8),
    );
    broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, destroy_pk.encode()).await;

    for (cx, cy, cz, nid, nmeta) in extra_changes {
        let npk = UpdateBlockPacket::new(cx, cy as u32, cz, nid as u32, nmeta as u32, update_block::FLAG_ALL);
        broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, npk.encode()).await;
    }

    let neighbors = [(x - 1, y, z), (x + 1, y, z), (x, y - 1, z), (x, y + 1, z), (x, y, z - 1), (x, y, z + 1)];
    for (nx, ny, nz) in neighbors {
        if ny >= 0 && ny < 256 {
            let nid = {
                let mut world = world_arc.write();
                world.get_block_id(nx, ny, nz)
            };
            if nid == 8 || nid == 9 || nid == 10 || nid == 11 {
                self_data.liquid_sim.schedule_update_dim(dim, nx, ny, nz);
            }
        }
    }

    if is_survival {
        // Damage tool on block break and handle tool breaking
        if !held_item.is_air() && crate::block::physics::block_hardness(old_id) > 0.0 {
            let mut updated_tool = held_item.clone();
            let broke = crate::block::physics::apply_tool_damage(&mut updated_tool);
            if crate::block::physics::tool_spec(held_item.id).max_durability > 0 {
                let out_item = if broke { crate::protocol::types::Item::air() } else { updated_tool };
                if let Some(s) = self_data.players.write().get_mut(&addr) {
                    s.inventory.set_item(held_slot, out_item.clone());
                }
                let slot_pk = ContainerSetSlotPacket::new(0, held_slot as i32, out_item);
                send_packet(&self_data.rak, addr, slot_pk.encode()).await;
                if broke {
                    let snd = LevelSoundEventPacket::new(level_event::SOUND_BREAK, px as f32, py as f32, pz as f32);
                    broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, snd.encode()).await;
                }
            }
        }

        let suppress_primary = matches!(old_id, 64 | 71 | 193..=197 | 26) && (old_meta & 0x08) != 0;
        let drops = if suppress_primary {
            Vec::new()
        } else {
            crate::block::physics::get_block_drops_ext(old_id, old_meta, &held_item)
        };
        for drop_item in drops {
            let entity_id = {
                let mut eid = self_data.next_entity_id.write();
                let id = *eid;
                *eid += 1;
                id
            };
            self_data.entity_manager.spawn_item_drop_in_dim(
                entity_id,
                dim,
                drop_item,
                x as f64 + 0.5,
                y as f64 + 0.5,
                z as f64 + 0.5,
                &self_data.rak,
                &self_data.players,
            ).await;
        }
        for (dx, dy, dz, item) in cascade_drops {
            let entity_id = {
                let mut eid = self_data.next_entity_id.write();
                let id = *eid;
                *eid += 1;
                id
            };
            self_data.entity_manager.spawn_item_drop_in_dim(
                entity_id,
                dim,
                item,
                dx as f64 + 0.5,
                dy as f64 + 0.5,
                dz as f64 + 0.5,
                &self_data.rak,
                &self_data.players,
            ).await;
        }

        let xp_gain = crate::block::physics::ore_xp_reward(old_id, &held_item);
        if xp_gain > 0 {
            self_data.entity_manager.spawn_xp_orbs_split_in_dim(
                dim,
                x as f64 + 0.5,
                y as f64 + 0.5,
                z as f64 + 0.5,
                xp_gain,
                &self_data.next_entity_id,
                &self_data.rak,
                &self_data.players,
            ).await;
        }
        if is_survival {
            let h_sync = {
                let mut p = self_data.players.write();
                p.get_mut(&addr).and_then(|s| {
                    if exhaust_player(s, 0.005) {
                        Some((s.entity_id, s.food_level, s.food_saturation))
                    } else {
                        None
                    }
                })
            };
            if let Some((heid, hf, hs)) = h_sync {
                let mut ap = UpdateAttributesPacket::new(heid);
                ap.attributes = vec![Attribute::hunger(hf as f32), Attribute::food_saturation(hs)];
                send_packet(&self_data.rak, addr, ap.encode()).await;
            }
        }
    }
}

#[allow(dead_code)]
fn get_block_drops(block_id: u8, meta: u8) -> Vec<crate::protocol::types::Item> {
    crate::block::physics::get_block_drops_ext(block_id, meta, &crate::protocol::types::Item::air())
}

async fn handle_use_item(self_data: &OxideServer, addr: SocketAddr, ui: UseItemPacket) {
    let player_info = self_data.players.read().get(&addr).map(|s| {
        (s.dimension, s.x, s.y, s.z, s.yaw, s.pitch, s.entity_id, s.gamemode == 0, s.health, s.food_level, s.inventory.held_slot(), s.inventory.held_item())
    });
    let (dim, px, py, pz, yaw, pitch, eid, is_survival, _cur_health, cur_food, held_slot, server_held_item) = match player_info {
        Some(info) => info,
        None => return,
    };
    let world_arc = self_data.get_world(dim);

    let item_id = if !server_held_item.is_air() { server_held_item.id } else { ui.item.id };
    let item_meta = if !server_held_item.is_air() { server_held_item.meta } else { ui.item.meta };

    // Bow (261): start drawing on item use, do not shoot or consume bow immediately
    if item_id == 261 && ui.face == -1 {
        let infinity = crate::item::enchantment::parse_enchantments_from_nbt(&server_held_item.nbt)
            .iter()
            .any(|e| e.id == crate::item::enchantment::TYPE_BOW_INFINITY);
        let has_arrow = if is_survival && !infinity {
            self_data.players.read().get(&addr).map(|s| s.inventory.contains_item(262, 1)).unwrap_or(false)
        } else {
            true
        };
        if has_arrow {
            let cur_tick = self_data.server_tick.load(std::sync::atomic::Ordering::Relaxed);
            let meta = {
                let mut players = self_data.players.write();
                if let Some(s) = players.get_mut(&addr) {
                    s.start_action_tick = Some(cur_tick);
                    s.metadata.set_generic_flag(crate::protocol::types::metadata::DATA_FLAG_ACTION, true);
                    Some(s.metadata.clone())
                } else {
                    None
                }
            };
            if let Some(m) = meta {
                let pk = SetEntityDataPacket::new(eid, m);
                broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, pk.encode()).await;
            }
        }
        return;
    }

    // 1. Throwable Projectiles: Ender Pearl (368), Snowball (332), Egg (344), Exp Bottle (384), Splash Potion (438)
    let projectile_kind = match item_id {
        368 => Some((crate::entity::ProjectileKind::EnderPearl, 1.5, None)),
        332 => Some((crate::entity::ProjectileKind::Snowball, 1.5, None)),
        344 => Some((crate::entity::ProjectileKind::Egg, 1.5, None)),
        384 => Some((crate::entity::ProjectileKind::ExpBottle, 0.7, None)),
        438 => Some((crate::entity::ProjectileKind::SplashPotion, 0.5, Some(item_meta))),
        _ => None,
    };

    if let Some((kind, force, potion_id)) = projectile_kind {
        let proj_eid = {
            let mut guard = self_data.next_entity_id.write();
            let id = *guard;
            *guard += 1;
            id
        };

        self_data.projectile_manager.launch_configured(
            proj_eid,
            dim,
            kind,
            addr,
            eid,
            px,
            py,
            pz,
            yaw,
            pitch,
            force,
            0.0,
            0.0,
            0,
            false,
            potion_id,
            &self_data.rak,
            &self_data.players,
        );

        if is_survival {
            let updated = {
                let mut players = self_data.players.write();
                if let Some(s) = players.get_mut(&addr) {
                    let mut it = s.inventory.get_item(held_slot);
                    if it.count > 1 {
                        it.count -= 1;
                        s.inventory.set_item(held_slot, it.clone());
                        it
                    } else {
                        let air = crate::protocol::types::Item::air();
                        s.inventory.set_item(held_slot, air.clone());
                        air
                    }
                } else {
                    crate::protocol::types::Item::air()
                }
            };
            let slot_pk = ContainerSetSlotPacket::new(0, held_slot as i32, updated);
            send_packet(&self_data.rak, addr, slot_pk.encode()).await;
        }
        return;
    }

    // 2. Milk Bucket (clears all active effects)
    let is_milk = item_id == 335 || (item_id == 325 && item_meta == 1);
    if is_milk {
        {
            let mut players = self_data.players.write();
            if let Some(s) = players.get_mut(&addr) {
                s.effects = crate::entity::effect::EffectContainer::new();
                if is_survival {
                    s.inventory.set_item(held_slot, crate::protocol::types::Item::new(325, 0, 1));
                }
            }
        }

        let drink_snd = LevelSoundEventPacket::new(level_event::SOUND_DRINK, px as f32, py as f32, pz as f32);
        broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, drink_snd.encode()).await;

        if is_survival {
            let slot_pk = ContainerSetSlotPacket::new(0, held_slot as i32, crate::protocol::types::Item::new(325, 0, 1));
            send_packet(&self_data.rak, addr, slot_pk.encode()).await;
        }
        return;
    }

    // 2.5. Potion items (373)
    if item_id == 373 {
        let potion_effects = crate::entity::effect::get_potion_effects(item_meta);
        if !potion_effects.is_empty() {
            let mut effects_to_send = Vec::new();
            let new_hp = {
                let mut players = self_data.players.write();
                if let Some(s) = players.get_mut(&addr) {
                    for eff in potion_effects {
                        if eff.is_instant() {
                            if eff.id == crate::entity::effect::effect_id::INSTANT_HEALTH {
                                let heal = s.effects.apply_instant_health(eff.amplifier);
                                s.health = (s.health + heal as u32).min(20);
                            } else if eff.id == crate::entity::effect::effect_id::INSTANT_DAMAGE {
                                let dmg = s.effects.apply_instant_damage(eff.amplifier);
                                s.health = s.health.saturating_sub((-dmg) as u32);
                            }
                        } else {
                            s.effects.add_effect(eff.clone());
                            effects_to_send.push(eff);
                        }
                    }
                    if is_survival {
                        s.inventory.set_item(held_slot, crate::protocol::types::Item::new(374, 0, 1));
                    }
                    s.health
                } else {
                    20
                }
            };

            let hp_pk = SetHealthPacket::new(new_hp as i32);
            send_packet(&self_data.rak, addr, hp_pk.encode()).await;

            let drink_snd = LevelSoundEventPacket::new(level_event::SOUND_DRINK, px as f32, py as f32, pz as f32);
            broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, drink_snd.encode()).await;

            for eff in effects_to_send {
                let eff_pk = MobEffectPacket {
                    entity_runtime_id: eid,
                    event_id: 1,
                    effect_id: eff.id as i32,
                    amplifier: eff.amplifier as i32,
                    particles: eff.show_particles,
                    duration: eff.duration,
                };
                send_packet(&self_data.rak, addr, eff_pk.encode()).await;
            }

            if is_survival {
                let slot_pk = ContainerSetSlotPacket::new(0, held_slot as i32, crate::protocol::types::Item::new(374, 0, 1));
                send_packet(&self_data.rak, addr, slot_pk.encode()).await;
            }
            return;
        }
    }

    // 3. Food items (skip if planting carrot/potato directly on farmland)
    let planting_crop = (item_id == 391 || item_id == 392) && ui.face == 1 && {
        world_arc.write().get_block_id(ui.x, ui.y as i32, ui.z) == 60
    };
    let food_info = if planting_crop {
        None
    } else {
        match item_id {
            260 => Some((4, 2.4f32)),
            297 => Some((5, 6.0f32)),
            319 => Some((3, 1.8f32)),
            320 => Some((8, 12.8f32)),
            322 => Some((4, 9.6f32)),
            363 => Some((3, 1.8f32)),
            364 => Some((8, 12.8f32)),
            365 => Some((2, 1.2f32)),
            366 => Some((6, 7.2f32)),
            367 => Some((4, 0.8f32)),
            391 => Some((1, 0.6f32)),
            393 => Some((5, 6.0f32)),
            396 => Some((6, 14.4f32)),
            411 => Some((2, 1.2f32)),
            412 => Some((6, 9.6f32)),
            466 => Some((4, 9.6f32)),
            _ => None,
        }
    };

    if let Some((food_gain, sat_gain)) = food_info {
        if cur_food < 20 || item_id == 322 || item_id == 466 {
            let mut effects_to_send = Vec::new();
            let (new_food, new_sat, new_hp) = {
                let mut players = self_data.players.write();
                if let Some(s) = players.get_mut(&addr) {
                    s.food_level = (s.food_level + food_gain).min(20);
                    s.food_saturation = (s.food_saturation + sat_gain).min(s.food_level as f32);

                    if item_id == 322 {
                        let eff1 = crate::entity::effect::ActiveEffect::new(crate::entity::effect::effect_id::REGENERATION, 100, 1);
                        let eff2 = crate::entity::effect::ActiveEffect::new(crate::entity::effect::effect_id::ABSORPTION, 2400, 0);
                        s.effects.add_effect(eff1.clone());
                        s.effects.add_effect(eff2.clone());
                        effects_to_send.push(eff1);
                        effects_to_send.push(eff2);
                    } else if item_id == 466 {
                        let eff1 = crate::entity::effect::ActiveEffect::new(crate::entity::effect::effect_id::REGENERATION, 600, 4);
                        let eff2 = crate::entity::effect::ActiveEffect::new(crate::entity::effect::effect_id::RESISTANCE, 6000, 0);
                        let eff3 = crate::entity::effect::ActiveEffect::new(crate::entity::effect::effect_id::FIRE_RESISTANCE, 6000, 0);
                        let eff4 = crate::entity::effect::ActiveEffect::new(crate::entity::effect::effect_id::ABSORPTION, 2400, 3);
                        s.effects.add_effect(eff1.clone());
                        s.effects.add_effect(eff2.clone());
                        s.effects.add_effect(eff3.clone());
                        s.effects.add_effect(eff4.clone());
                        effects_to_send.push(eff1);
                        effects_to_send.push(eff2);
                        effects_to_send.push(eff3);
                        effects_to_send.push(eff4);
                    } else if item_id == 367 && rand::random::<f32>() < 0.8 {
                        let eff = crate::entity::effect::ActiveEffect::new(crate::entity::effect::effect_id::HUNGER, 600, 0);
                        s.effects.add_effect(eff.clone());
                        effects_to_send.push(eff);
                    } else if item_id == 365 && rand::random::<f32>() < 0.3 {
                        let eff = crate::entity::effect::ActiveEffect::new(crate::entity::effect::effect_id::HUNGER, 600, 0);
                        s.effects.add_effect(eff.clone());
                        effects_to_send.push(eff);
                    }

                    (s.food_level, s.food_saturation, s.health)
                } else {
                    (20, 5.0, 20)
                }
            };

            let mut attrs_pk = UpdateAttributesPacket::new(eid);
            attrs_pk.attributes = vec![
                Attribute::health(new_hp as f32),
                Attribute::hunger(new_food as f32),
                Attribute::food_saturation(new_sat),
            ];
            send_packet(&self_data.rak, addr, attrs_pk.encode()).await;

            for eff in effects_to_send {
                let eff_pk = MobEffectPacket {
                    entity_runtime_id: eid,
                    event_id: 1,
                    effect_id: eff.id as i32,
                    amplifier: eff.amplifier as i32,
                    particles: eff.show_particles,
                    duration: eff.duration,
                };
                send_packet(&self_data.rak, addr, eff_pk.encode()).await;
            }

            let eat_sound = LevelSoundEventPacket::new(level_event::SOUND_EAT, px as f32, py as f32, pz as f32);
            let burp_sound = LevelSoundEventPacket::new(level_event::SOUND_BURP, px as f32, py as f32, pz as f32);
            broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, eat_sound.encode()).await;
            broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, burp_sound.encode()).await;

            if is_survival {
                let updated = {
                    let mut players = self_data.players.write();
                    if let Some(s) = players.get_mut(&addr) {
                        let mut it = s.inventory.get_item(held_slot);
                        if it.count > 1 {
                            it.count -= 1;
                            s.inventory.set_item(held_slot, it.clone());
                            it
                        } else {
                            let air = crate::protocol::types::Item::air();
                            s.inventory.set_item(held_slot, air.clone());
                            air
                        }
                    } else {
                        crate::protocol::types::Item::air()
                    }
                };
                let slot_pk = ContainerSetSlotPacket::new(0, held_slot as i32, updated);
                send_packet(&self_data.rak, addr, slot_pk.encode()).await;
            }
            return;
        }
    }

    // 3. Bucket handling
    let is_water_bucket = item_id == 326 || (item_id == 325 && (item_meta == 8 || item_meta == 9));
    let is_lava_bucket = item_id == 327 || (item_id == 325 && (item_meta == 10 || item_meta == 11));
    let is_empty_bucket = item_id == 325 && !is_water_bucket && !is_lava_bucket;

    if is_water_bucket || is_lava_bucket || is_empty_bucket {
        if ui.face >= 0 && ui.face <= 5 {
            let (nx, ny, nz) = match ui.face {
                0 => (ui.x, ui.y as i32 - 1, ui.z),
                1 => (ui.x, ui.y as i32 + 1, ui.z),
                2 => (ui.x, ui.y as i32, ui.z - 1),
                3 => (ui.x, ui.y as i32, ui.z + 1),
                4 => (ui.x - 1, ui.y as i32, ui.z),
                5 => (ui.x + 1, ui.y as i32, ui.z),
                _ => (ui.x, ui.y as i32 + 1, ui.z),
            };

            if is_water_bucket {
                {
                    let mut w = world_arc.write();
                    w.set_block(nx, ny, nz, 8, 0);
                }
                let pk = UpdateBlockPacket::new(nx, ny as u32, nz, 8, 0, update_block::FLAG_ALL);
                broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, pk.encode()).await;
                self_data.liquid_sim.schedule_update_dim(dim, nx, ny, nz);

                let sound = LevelSoundEventPacket::new(level_event::SOUND_BUCKET_EMPTY_WATER, nx as f32 + 0.5, ny as f32 + 0.5, nz as f32 + 0.5);
                broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, sound.encode()).await;

                if is_survival {
                    let empty_bucket = crate::protocol::types::Item::new(325, 0, 1);
                    {
                        let mut players = self_data.players.write();
                        if let Some(s) = players.get_mut(&addr) {
                            s.inventory.set_item(held_slot, empty_bucket.clone());
                        }
                    }
                    let slot_pk = ContainerSetSlotPacket::new(0, held_slot as i32, empty_bucket);
                    send_packet(&self_data.rak, addr, slot_pk.encode()).await;
                }
                return;
            } else if is_lava_bucket {
                {
                    let mut w = world_arc.write();
                    w.set_block(nx, ny, nz, 10, 0);
                }
                let pk = UpdateBlockPacket::new(nx, ny as u32, nz, 10, 0, update_block::FLAG_ALL);
                broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, pk.encode()).await;
                self_data.liquid_sim.schedule_update_dim(dim, nx, ny, nz);

                let sound = LevelSoundEventPacket::new(level_event::SOUND_BUCKET_EMPTY_LAVA, nx as f32 + 0.5, ny as f32 + 0.5, nz as f32 + 0.5);
                broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, sound.encode()).await;

                if is_survival {
                    let empty_bucket = crate::protocol::types::Item::new(325, 0, 1);
                    {
                        let mut players = self_data.players.write();
                        if let Some(s) = players.get_mut(&addr) {
                            s.inventory.set_item(held_slot, empty_bucket.clone());
                        }
                    }
                    let slot_pk = ContainerSetSlotPacket::new(0, held_slot as i32, empty_bucket);
                    send_packet(&self_data.rak, addr, slot_pk.encode()).await;
                }
                return;
            } else if is_empty_bucket {
                let target_id = {
                    let mut w = world_arc.write();
                    w.get_block_id(ui.x, ui.y as i32, ui.z)
                };
                if target_id == 8 || target_id == 9 {
                    {
                        let mut w = world_arc.write();
                        w.set_block(ui.x, ui.y as i32, ui.z, 0, 0);
                    }
                    let pk = UpdateBlockPacket::new(ui.x, ui.y, ui.z, 0, 0, update_block::FLAG_ALL);
                    broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, pk.encode()).await;

                    let sound = LevelSoundEventPacket::new(level_event::SOUND_BUCKET_FILL_WATER, ui.x as f32 + 0.5, ui.y as f32 + 0.5, ui.z as f32 + 0.5);
                    broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, sound.encode()).await;

                    self_data.liquid_sim.schedule_update_dim(dim, ui.x - 1, ui.y as i32, ui.z);
                    self_data.liquid_sim.schedule_update_dim(dim, ui.x + 1, ui.y as i32, ui.z);
                    self_data.liquid_sim.schedule_update_dim(dim, ui.x, ui.y as i32, ui.z - 1);
                    self_data.liquid_sim.schedule_update_dim(dim, ui.x, ui.y as i32, ui.z + 1);

                    if is_survival {
                        let water_bucket = crate::protocol::types::Item::new(325, 8, 1);
                        {
                            let mut players = self_data.players.write();
                            if let Some(s) = players.get_mut(&addr) {
                                s.inventory.set_item(held_slot, water_bucket.clone());
                            }
                        }
                        let slot_pk = ContainerSetSlotPacket::new(0, held_slot as i32, water_bucket);
                        send_packet(&self_data.rak, addr, slot_pk.encode()).await;
                    }
                    return;
                } else if target_id == 10 || target_id == 11 {
                    {
                        let mut w = world_arc.write();
                        w.set_block(ui.x, ui.y as i32, ui.z, 0, 0);
                    }
                    let pk = UpdateBlockPacket::new(ui.x, ui.y, ui.z, 0, 0, update_block::FLAG_ALL);
                    broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, pk.encode()).await;

                    let sound = LevelSoundEventPacket::new(level_event::SOUND_BUCKET_FILL_LAVA, ui.x as f32 + 0.5, ui.y as f32 + 0.5, ui.z as f32 + 0.5);
                    broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, sound.encode()).await;

                    self_data.liquid_sim.schedule_update_dim(dim, ui.x - 1, ui.y as i32, ui.z);
                    self_data.liquid_sim.schedule_update_dim(dim, ui.x + 1, ui.y as i32, ui.z);
                    self_data.liquid_sim.schedule_update_dim(dim, ui.x, ui.y as i32, ui.z - 1);
                    self_data.liquid_sim.schedule_update_dim(dim, ui.x, ui.y as i32, ui.z + 1);

                    if is_survival {
                        let lava_bucket = crate::protocol::types::Item::new(325, 10, 1);
                        {
                            let mut players = self_data.players.write();
                            if let Some(s) = players.get_mut(&addr) {
                                s.inventory.set_item(held_slot, lava_bucket.clone());
                            }
                        }
                        let slot_pk = ContainerSetSlotPacket::new(0, held_slot as i32, lava_bucket);
                        send_packet(&self_data.rak, addr, slot_pk.encode()).await;
                    }
                    return;
                }
            }
        }
    }

    // 4. Block and Container Interactions
    if ui.face >= 0 && ui.face <= 5 {
        let (target_id, target_meta) = {
            let mut w = world_arc.write();
            let id = w.get_block_id(ui.x, ui.y as i32, ui.z);
            let chunk_x = ui.x >> 4;
            let chunk_z = ui.z >> 4;
            let cx = (ui.x & 0x0f) as usize;
            let cz = (ui.z & 0x0f) as usize;
            let chunk = w.get_chunk_or_generate(chunk_x, chunk_z);
            let meta = chunk.get_block_data(cx, ui.y as usize, cz);
            (id, meta)
        };

        // Flint and Steel (259) on TNT (46)
        if (item_id == 259 || item_id == 385) && target_id == 46 {
            {
                let mut w = world_arc.write();
                w.set_block(ui.x, ui.y as i32, ui.z, 0, 0);
            }
            let clear_pk = UpdateBlockPacket::new(ui.x, ui.y, ui.z, 0, 0, update_block::FLAG_ALL);
            broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, clear_pk.encode()).await;

            let tnt_eid = {
                let mut guard = self_data.next_entity_id.write();
                let id = *guard;
                *guard += 1;
                id
            };
            self_data.entity_manager.spawn_tnt_in_dim(
                tnt_eid,
                dim,
                ui.x as f64 + 0.5,
                ui.y as f64,
                ui.z as f64 + 0.5,
                80,
                &self_data.rak,
                &self_data.players,
            ).await;
            return;
        }

        // Flint and Steel (259) or Fire Charge (385) on TNT (46) -> Ignite TNT
        if (item_id == 259 || item_id == 385) && target_id == 46 {
            {
                let mut w = world_arc.write();
                w.set_block(ui.x, ui.y as i32, ui.z, 0, 0);
            }
            let pk = UpdateBlockPacket::new(ui.x, ui.y, ui.z, 0, 0, update_block::FLAG_ALL);
            broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, pk.encode()).await;
            let tid = {
                let mut g = self_data.next_entity_id.write();
                let v = *g;
                *g += 1;
                v
            };
            self_data.entity_manager.spawn_tnt_in_dim(tid, dim, ui.x as f64 + 0.5, ui.y as f64, ui.z as f64 + 0.5, 80, &self_data.rak, &self_data.players).await;
            if is_survival {
                let mut updated = server_held_item.clone();
                let broke = crate::block::physics::apply_tool_damage(&mut updated);
                let out = if broke { crate::protocol::types::Item::air() } else { updated };
                if let Some(s) = self_data.players.write().get_mut(&addr) {
                    s.inventory.set_item(held_slot, out.clone());
                }
                send_packet(&self_data.rak, addr, ContainerSetSlotPacket::new(0, held_slot as i32, out).encode()).await;
            }
            return;
        }

        // Flint and Steel (259) or Fire Charge (385) on Obsidian (49) -> Ignite Nether Portal
        if (item_id == 259 || item_id == 385) && target_id == 49 {
            let portal_blocks = {
                let mut w = world_arc.write();
                try_ignite_nether_portal(&mut w, ui.x, ui.y as i32, ui.z)
            };
            if let Some(blocks) = portal_blocks {
                for &(bx, by, bz) in &blocks {
                    let pk = UpdateBlockPacket::new(bx, by as u32, bz, 90, 1, update_block::FLAG_ALL_PRIORITY);
                    broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, pk.encode()).await;
                }
                let sound = LevelSoundEventPacket::new(level_event::SOUND_PORTAL, ui.x as f32 + 0.5, ui.y as f32 + 0.5, ui.z as f32 + 0.5);
                broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, sound.encode()).await;
                return;
            }
        }

        // Hoe on Dirt (3) or Grass (2) -> Farmland (60)
        let is_hoe = item_id == 290 || item_id == 291 || item_id == 292 || item_id == 293 || item_id == 294;
        if is_hoe && (target_id == 2 || target_id == 3) && ui.face == 1 {
            {
                let mut w = world_arc.write();
                w.set_block(ui.x, ui.y as i32, ui.z, 60, 0);
            }
            let pk = UpdateBlockPacket::new(ui.x, ui.y, ui.z, 60, 0, update_block::FLAG_ALL);
            broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, pk.encode()).await;
            let till_snd = LevelSoundEventPacket::new(level_event::SOUND_ITEM_USE_ON, ui.x as f32 + 0.5, ui.y as f32 + 0.5, ui.z as f32 + 0.5);
            broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, till_snd.encode()).await;
            return;
        }

        // Shovel on Grass (2) -> Grass Path (198)
        let is_shovel = item_id == 256 || item_id == 269 || item_id == 273 || item_id == 277 || item_id == 284;
        if is_shovel && target_id == 2 && ui.face == 1 {
            {
                let mut w = world_arc.write();
                w.set_block(ui.x, ui.y as i32, ui.z, 198, 0);
            }
            let pk = UpdateBlockPacket::new(ui.x, ui.y, ui.z, 198, 0, update_block::FLAG_ALL);
            broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, pk.encode()).await;
            let till_snd = LevelSoundEventPacket::new(level_event::SOUND_ITEM_USE_ON, ui.x as f32 + 0.5, ui.y as f32 + 0.5, ui.z as f32 + 0.5);
            broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, till_snd.encode()).await;
            return;
        }

        // Bone Meal (351:15) on crops (59, 141, 142, 244) or sapling (6)
        if item_id == 351 && item_meta == 15 && matches!(target_id, 59 | 141 | 142 | 244 | 6) {
            if target_id == 6 {
                let tree_changes = {
                    let mut w = world_arc.write();
                    let changes = crate::level::random_tick::grow_simple_tree(&mut w, ui.x, ui.y as i32, ui.z, target_meta & 0x03);
                    if !changes.is_empty() {
                        crate::level::light::update_light_around(&mut w, ui.x, ui.y as i32, ui.z);
                    }
                    changes
                };
                for (bx, by, bz, bid, bmeta) in tree_changes {
                    let pk = UpdateBlockPacket::new(bx, by as u32, bz, bid as u32, bmeta as u32, update_block::FLAG_ALL);
                    broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, pk.encode()).await;
                }
            } else {
                let max_stage = if target_id == 244 { 4 } else { 7 };
                let step = {
                    use rand::Rng;
                    rand::thread_rng().gen_range(2..=5)
                };
                let new_meta = (target_meta + step).min(max_stage);
                {
                    let mut w = world_arc.write();
                    w.set_block(ui.x, ui.y as i32, ui.z, target_id, new_meta);
                }
                let pk = UpdateBlockPacket::new(ui.x, ui.y, ui.z, target_id as u32, new_meta as u32, update_block::FLAG_ALL);
                broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, pk.encode()).await;
            }
            let grow_snd = LevelEventPacket::new(level_event::EVENT_PARTICLE_SPLASH, ui.x as f32 + 0.5, ui.y as f32 + 0.5, ui.z as f32 + 0.5, 0);
            broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, grow_snd.encode()).await;
            if is_survival {
                let updated = {
                    let mut players = self_data.players.write();
                    if let Some(s) = players.get_mut(&addr) {
                        let mut it = s.inventory.get_item(held_slot);
                        if it.count > 1 {
                            it.count -= 1;
                            s.inventory.set_item(held_slot, it.clone());
                            it
                        } else {
                            let air = crate::protocol::types::Item::air();
                            s.inventory.set_item(held_slot, air.clone());
                            air
                        }
                    } else {
                        crate::protocol::types::Item::air()
                    }
                };
                let slot_pk = ContainerSetSlotPacket::new(0, held_slot as i32, updated);
                send_packet(&self_data.rak, addr, slot_pk.encode()).await;
            }
            return;
        }

        // Doors (64, 193..197), Trapdoors (96), Gates (107, 183..187) toggle
        if matches!(target_id, 64 | 193..=197) {
            let bottom_y = if (target_meta & 0x08) != 0 && ui.y > 0 {
                ui.y as i32 - 1
            } else {
                ui.y as i32
            };
            let new_meta = {
                let mut w = world_arc.write();
                if w.get_block_id(ui.x, bottom_y, ui.z) == target_id {
                    let ch = w.get_chunk_or_generate(ui.x >> 4, ui.z >> 4);
                    let cur = ch.get_block_data((ui.x & 0x0f) as usize, bottom_y as usize, (ui.z & 0x0f) as usize);
                    let toggled = cur ^ 0x04;
                    w.set_block(ui.x, bottom_y, ui.z, target_id, toggled);
                    Some(toggled)
                } else {
                    None
                }
            };
            if let Some(m) = new_meta {
                let pk = UpdateBlockPacket::new(ui.x, bottom_y as u32, ui.z, target_id as u32, m as u32, update_block::FLAG_ALL);
                broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, pk.encode()).await;
                let door_sound = LevelEventPacket::new(
                    level_event::EVENT_SOUND_DOOR,
                    ui.x as f32 + 0.5,
                    bottom_y as f32 + 0.5,
                    ui.z as f32 + 0.5,
                    0,
                );
                broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, door_sound.encode()).await;
            }
            return;
        }

        if matches!(target_id, 96 | 107 | 183..=187) {
            let toggle_bit = if target_id == 96 { 0x08 } else { 0x04 };
            let new_meta = target_meta ^ toggle_bit;
            {
                let mut w = world_arc.write();
                w.set_block(ui.x, ui.y as i32, ui.z, target_id, new_meta);
            }
            let pk = UpdateBlockPacket::new(ui.x, ui.y, ui.z, target_id as u32, new_meta as u32, update_block::FLAG_ALL);
            broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, pk.encode()).await;

            let door_sound = LevelEventPacket::new(
                level_event::EVENT_SOUND_DOOR,
                ui.x as f32 + 0.5,
                ui.y as f32 + 0.5,
                ui.z as f32 + 0.5,
                0,
            );
            broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, door_sound.encode()).await;
            return;
        }

        // Lever (69) toggle
        if target_id == 69 {
            let new_meta = target_meta ^ 0x08;
            let rs = {
                let mut w = world_arc.write();
                w.set_block(ui.x, ui.y as i32, ui.z, 69, new_meta);
                let (ax, ay, az) = crate::level::redstone::attached_offset_for_lever(new_meta);
                let mut r = crate::level::redstone::update_redstone_around(
                    &mut w,
                    &[(ui.x, ui.y as i32, ui.z), (ui.x + ax, ui.y as i32 + ay, ui.z + az)],
                );
                r.block_changes.insert(0, (ui.x, ui.y as i32, ui.z, 69, new_meta));
                r
            };
            let click = LevelEventPacket::new(level_event::EVENT_SOUND_CLICK, ui.x as f32 + 0.5, ui.y as f32 + 0.5, ui.z as f32 + 0.5, 0);
            broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, click.encode()).await;
            apply_redstone_result(&self_data.rak, &self_data.players, &self_data.entity_manager, &self_data.next_entity_id, dim, rs).await;
            return;
        }

        // Buttons (Stone 77 = 20t, Wood 143 = 30t)
        if target_id == 77 || target_id == 143 {
            if (target_meta & 0x08) == 0 {
                let new_meta = target_meta | 0x08;
                let rs = {
                    let mut w = world_arc.write();
                    w.set_block(ui.x, ui.y as i32, ui.z, target_id, new_meta);
                    let (ax, ay, az) = crate::level::redstone::attached_offset_for_button(new_meta);
                    let mut r = crate::level::redstone::update_redstone_around(
                        &mut w,
                        &[(ui.x, ui.y as i32, ui.z), (ui.x + ax, ui.y as i32 + ay, ui.z + az)],
                    );
                    r.block_changes.insert(0, (ui.x, ui.y as i32, ui.z, target_id, new_meta));
                    r
                };
                let delay = if target_id == 77 { 20u64 } else { 30u64 };
                let cur_tick = self_data.server_tick.load(std::sync::atomic::Ordering::Relaxed);
                self_data.redstone_sched.schedule_release(dim, ui.x, ui.y as i32, ui.z, cur_tick + delay, target_id);
                let click = LevelEventPacket::new(level_event::EVENT_SOUND_CLICK, ui.x as f32 + 0.5, ui.y as f32 + 0.5, ui.z as f32 + 0.5, 0);
                broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, click.encode()).await;
                apply_redstone_result(&self_data.rak, &self_data.players, &self_data.entity_manager, &self_data.next_entity_id, dim, rs).await;
            }
            return;
        }

        // Repeater (93, 94) delay cycle
        if target_id == 93 || target_id == 94 {
            let dir = target_meta & 0x03;
            let delay_bits = ((target_meta >> 2) + 1) & 0x03;
            let new_meta = (delay_bits << 2) | dir;
            {
                let mut w = world_arc.write();
                w.set_block(ui.x, ui.y as i32, ui.z, target_id, new_meta);
            }
            let pk = UpdateBlockPacket::new(ui.x, ui.y, ui.z, target_id as u32, new_meta as u32, update_block::FLAG_ALL);
            broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, pk.encode()).await;
            return;
        }

        // Crafting Table (58)
        if target_id == 58 {
            if let Some(s) = self_data.players.write().get_mut(&addr) {
                s.open_window_type = Some(crate::protocol::types::WindowTypes::WORKBENCH);
            }
            let open_pk = ContainerOpenPacket::new(1, 1, ui.x, ui.y, ui.z);
            send_packet(&self_data.rak, addr, open_pk.encode()).await;
            return;
        }

        // Chest / Trapped Chest (54, 146)
        if target_id == 54 || target_id == 146 {
            let (slots, first) = self_data.tile_manager.open_chest_in_dim(addr, dim, ui.x, ui.y as i32, ui.z);
            {
                let mut p = self_data.players.write();
                if let Some(s) = p.get_mut(&addr) {
                    s.open_container = Some((ui.x, ui.y as i32, ui.z));
                    s.open_window_type = Some(crate::protocol::types::WindowTypes::CONTAINER);
                }
            }
            let open_pk = ContainerOpenPacket::new(1, 0, ui.x, ui.y, ui.z);
            send_packet(&self_data.rak, addr, open_pk.encode()).await;

            let mut content_pk = ContainerSetContentPacket::new(1);
            content_pk.items = slots;
            send_packet(&self_data.rak, addr, content_pk.encode()).await;

            if first {
                let block_ev = BlockEventPacket { x: ui.x, y: ui.y, z: ui.z, case_1: 1, case_2: 2 };
                let sound = LevelSoundEventPacket::new(level_event::SOUND_CHEST_OPEN, ui.x as f32 + 0.5, ui.y as f32 + 0.5, ui.z as f32 + 0.5);
                broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, block_ev.encode()).await;
                broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, sound.encode()).await;
            }
            return;
        }

        // Ender Chest (130)
        if target_id == 130 {
            let slots = {
                let mut p = self_data.players.write();
                if let Some(s) = p.get_mut(&addr) {
                    s.open_container = None;
                    s.open_window_type = Some(crate::protocol::types::WindowTypes::CONTAINER);
                    s.ender_chest.clone()
                } else {
                    vec![crate::protocol::types::Item::air(); 27]
                }
            };
            let open_pk = ContainerOpenPacket::new(1, 0, ui.x, ui.y, ui.z);
            send_packet(&self_data.rak, addr, open_pk.encode()).await;

            let mut content_pk = ContainerSetContentPacket::new(1);
            content_pk.items = slots;
            send_packet(&self_data.rak, addr, content_pk.encode()).await;

            let sound = LevelSoundEventPacket::new(level_event::SOUND_CHEST_OPEN, ui.x as f32 + 0.5, ui.y as f32 + 0.5, ui.z as f32 + 0.5);
            broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, sound.encode()).await;
            return;
        }

        // Furnace (61, 62)
        if target_id == 61 || target_id == 62 {
            let slots = self_data.tile_manager.open_furnace_in_dim(addr, dim, ui.x, ui.y as i32, ui.z);
            let (cook_t, burn_t) = self_data.tile_manager.get_furnace_progress_in_dim(dim, ui.x, ui.y as i32, ui.z);
            {
                let mut p = self_data.players.write();
                if let Some(s) = p.get_mut(&addr) {
                    s.open_container = Some((ui.x, ui.y as i32, ui.z));
                    s.open_window_type = Some(crate::protocol::types::WindowTypes::FURNACE);
                }
            }
            let open_pk = ContainerOpenPacket::new(1, 2, ui.x, ui.y, ui.z);
            send_packet(&self_data.rak, addr, open_pk.encode()).await;

            let mut content_pk = ContainerSetContentPacket::new(1);
            content_pk.items = slots;
            send_packet(&self_data.rak, addr, content_pk.encode()).await;

            let dp0 = ContainerSetDataPacket { window_id: 1, property: 0, value: cook_t };
            let dp1 = ContainerSetDataPacket { window_id: 1, property: 1, value: burn_t };
            send_packet(&self_data.rak, addr, dp0.encode()).await;
            send_packet(&self_data.rak, addr, dp1.encode()).await;
            return;
        }

        // Anvil (145)
        if target_id == 145 {
            self_data.tile_manager.open_temp_window(addr, crate::protocol::types::WindowTypes::ANVIL as u8, dim, ui.x, ui.y as i32, ui.z, 3);
            {
                let mut p = self_data.players.write();
                if let Some(s) = p.get_mut(&addr) {
                    s.open_container = Some((ui.x, ui.y as i32, ui.z));
                    s.open_window_type = Some(crate::protocol::types::WindowTypes::ANVIL);
                }
            }
            let open_pk = ContainerOpenPacket::new(1, 5, ui.x, ui.y, ui.z);
            send_packet(&self_data.rak, addr, open_pk.encode()).await;
            return;
        }

        // Enchanting Table (116)
        if target_id == 116 {
            self_data.tile_manager.open_temp_window(addr, crate::protocol::types::WindowTypes::ENCHANTMENT as u8, dim, ui.x, ui.y as i32, ui.z, 2);
            {
                let mut p = self_data.players.write();
                if let Some(s) = p.get_mut(&addr) {
                    s.open_container = Some((ui.x, ui.y as i32, ui.z));
                    s.open_window_type = Some(crate::protocol::types::WindowTypes::ENCHANTMENT);
                }
            }
            let open_pk = ContainerOpenPacket::new(1, 3, ui.x, ui.y, ui.z);
            send_packet(&self_data.rak, addr, open_pk.encode()).await;
            return;
        }

        // Brewing Stand (117)
        if target_id == 117 {
            let (slots, brew_t) = self_data.tile_manager.open_brewing_in_dim(addr, dim, ui.x, ui.y as i32, ui.z);
            {
                let mut p = self_data.players.write();
                if let Some(s) = p.get_mut(&addr) {
                    s.open_container = Some((ui.x, ui.y as i32, ui.z));
                    s.open_window_type = Some(crate::protocol::types::WindowTypes::BREWING_STAND);
                }
            }
            let open_pk = ContainerOpenPacket::new(1, 4, ui.x, ui.y, ui.z);
            send_packet(&self_data.rak, addr, open_pk.encode()).await;

            let mut content_pk = ContainerSetContentPacket::new(1);
            content_pk.items = slots;
            send_packet(&self_data.rak, addr, content_pk.encode()).await;

            let dp = ContainerSetDataPacket { window_id: 1, property: 0, value: if brew_t < 400 { brew_t as i32 } else { 0 } };
            send_packet(&self_data.rak, addr, dp.encode()).await;
            return;
        }

        // Bed (26)
        if target_id == 26 {
            if dim == 1 || dim == 2 {
                // Explode bed in Nether/End
                let ex = ui.x as f64 + 0.5;
                let ey = ui.y as f64 + 0.5;
                let ez = ui.z as f64 + 0.5;
                let res = {
                    let mut w = world_arc.write();
                    crate::level::explosion::compute_explosion(&mut w, ex, ey, ez, 5.0)
                };
                for (bx, by, bz, bid, bmeta) in res.destroyed_blocks {
                    let pk = UpdateBlockPacket::new(bx, by as u32, bz, bid as u32, bmeta as u32, update_block::FLAG_ALL);
                    broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, pk.encode()).await;
                }
                for (tx, ty, tz, fuse) in res.ignited_tnts {
                    let tid = {
                        let mut id_g = self_data.next_entity_id.write();
                        let val = *id_g;
                        *id_g += 1;
                        val
                    };
                    self_data.entity_manager.spawn_tnt_in_dim(tid, dim, tx, ty, tz, fuse, &self_data.rak, &self_data.players).await;
                }
                let mut hit_players = Vec::new();
                {
                    let guard = self_data.players.read();
                    for (paddr, s) in guard.iter() {
                        if s.dimension == dim && s.spawned && s.gamemode == 0 {
                            if let Some((dmg, _, _, _)) = crate::level::explosion::explosion_damage_and_knockback(s.x, s.y, s.z, ex, ey, ez, 5.0) {
                                hit_players.push((*paddr, dmg, s.username.clone()));
                            }
                        }
                    }
                }
                for (paddr, dmg, uname) in hit_players {
                    let msg = format!("{} was killed by [Intentional Game Design]", uname);
                    crate::entity::damage::hurt_player(
                        &self_data.players,
                        &self_data.rak,
                        paddr,
                        dmg,
                        crate::entity::damage::DamageCause::BlockExplosion,
                        false,
                        Some((ex, ey, ez)),
                        None,
                        Some(&msg),
                    )
                    .await;
                }
                let exp_pk = ExplodePacket::new(ex as f32, ey as f32, ez as f32, 5.0);
                broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, exp_pk.encode()).await;
                let snd = LevelSoundEventPacket::new(level_event::SOUND_EXPLODE, ex as f32, ey as f32, ez as f32);
                broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, snd.encode()).await;
                return;
            }
            if let Some(s) = self_data.players.write().get_mut(&addr) {
                s.spawn_pos = Some((ui.x, ui.y as i32, ui.z, dim));
            }
            self_data.send_message_to(addr, "Respawn point set");
            return;
        }

        // Normal block placement (including doors, beds, signs, seeds, redstone)
        if let Some(block_id) = crate::block::physics::item_to_block_id(item_id, target_id, ui.face) {
            handle_place_block(self_data, addr, ui.x, ui.y as i32, ui.z, ui.face, ui.fy, block_id, item_meta).await;
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn handle_place_block(
    self_data: &OxideServer,
    addr: SocketAddr,
    x: i32,
    y: i32,
    z: i32,
    face: i32,
    fy: f32,
    block_id: u8,
    item_meta: i32,
) {
    let (player_name, is_survival, held_slot, dim, px, py, pz, yaw) = match self_data.players.read().get(&addr) {
        Some(s) => (s.username.clone(), s.gamemode == 0, s.inventory.held_slot(), s.dimension, s.x, s.y, s.z, s.yaw),
        None => return,
    };
    let world_arc = self_data.get_world(dim);

    let placement = {
        let mut world = world_arc.write();
        crate::block::physics::resolve_placement(&mut world, x, y, z, face, fy, yaw, block_id, item_meta)
    };

    let placed_list: Vec<(i32, i32, i32, u8, u8)> = match placement {
        crate::block::physics::PlacementResult::Denied => return,
        crate::block::physics::PlacementResult::Single { x: px_b, y: py_b, z: pz_b, id, meta } => {
            vec![(px_b, py_b, pz_b, id, meta)]
        }
        crate::block::physics::PlacementResult::Double { x1, y1, z1, id1, meta1, x2, y2, z2, id2, meta2 } => {
            vec![(x1, y1, z1, id1, meta1), (x2, y2, z2, id2, meta2)]
        }
    };

    for &(bx, by, bz, bid, _) in &placed_list {
        let dx = px - (bx as f64 + 0.5);
        let dy = py - (by as f64 + 0.5);
        let dz = pz - (bz as f64 + 0.5);
        if dx * dx + dy * dy + dz * dz > 64.0 {
            return;
        }
        if crate::block::physics::is_solid(bid) {
            let intersects = (px - 0.3 < bx as f64 + 1.0 && px + 0.3 > bx as f64)
                && (py < by as f64 + 1.0 && py + 1.8 > by as f64)
                && (pz - 0.3 < bz as f64 + 1.0 && pz + 0.3 > bz as f64);
            if intersects {
                return;
            }
        }
    }

    let (primary_x, primary_y, primary_z, primary_id, _) = placed_list[0];
    let ev = self_data.event_bus.fire(crate::event::event::BlockPlaceEvent {
        player_name,
        x: primary_x,
        y: primary_y as u32,
        z: primary_z,
        block_id: primary_id as u16,
        cancelled: false,
    });
    if ev.cancelled {
        for &(bx, by, bz, _, _) in &placed_list {
            let revert_pk = UpdateBlockPacket::new(bx, by as u32, bz, 0, 0, update_block::FLAG_ALL);
            send_packet(&self_data.rak, addr, revert_pk.encode()).await;
        }
        return;
    }

    let (cascaded_changes, cascade_drops, rs_res) = {
        let mut world = world_arc.write();
        let mut origins = Vec::with_capacity(placed_list.len());
        for &(bx, by, bz, bid, bmeta) in &placed_list {
            world.set_block(bx, by, bz, bid, bmeta);
            crate::level::light::update_light_around(&mut world, bx, by, bz);
            origins.push((bx, by, bz));
        }
        let (cascaded, c_drops) = crate::block::physics::cascade_neighbor_breaks(&mut world, &origins);
        let rs = crate::level::redstone::update_redstone_around(&mut world, &origins);
        (cascaded, c_drops, rs)
    };
    apply_redstone_result(&self_data.rak, &self_data.players, &self_data.entity_manager, &self_data.next_entity_id, dim, rs_res).await;

    for &(bx, by, bz, bid, bmeta) in &placed_list {
        let pk = UpdateBlockPacket::new(bx, by as u32, bz, bid as u32, bmeta as u32, update_block::FLAG_ALL);
        broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, pk.encode()).await;

        match bid {
            54 | 146 => {
                let is_trapped = bid == 146;
                self_data.tile_manager.ensure_chest_in_dim(dim, bx, by, bz, is_trapped);
                let paired = self_data.tile_manager.try_pair_chest_in_dim(dim, bx, by, bz, is_trapped);
                let nbt = self_data.tile_manager.build_chest_nbt_in_dim(dim, bx, by, bz);
                let bed_pk = BlockEntityDataPacket { x: bx, y: by as u32, z: bz, namedtag: nbt };
                broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, bed_pk.encode()).await;
                if let Some((px, py, pz)) = paired {
                    let pnbt = self_data.tile_manager.build_chest_nbt_in_dim(dim, px, py, pz);
                    let pbed_pk = BlockEntityDataPacket { x: px, y: py as u32, z: pz, namedtag: pnbt };
                    broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, pbed_pk.encode()).await;
                }
            }
            61 | 62 => {
                self_data.tile_manager.ensure_furnace_in_dim(dim, bx, by, bz);
                let nbt = self_data.tile_manager.build_furnace_nbt_in_dim(dim, bx, by, bz);
                let bed_pk = BlockEntityDataPacket { x: bx, y: by as u32, z: bz, namedtag: nbt };
                broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, bed_pk.encode()).await;
            }
            116 => {
                self_data.tile_manager.ensure_enchant_in_dim(dim, bx, by, bz);
                let nbt = self_data.tile_manager.build_enchant_nbt_in_dim(dim, bx, by, bz);
                let bed_pk = BlockEntityDataPacket { x: bx, y: by as u32, z: bz, namedtag: nbt };
                broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, bed_pk.encode()).await;
            }
            117 => {
                self_data.tile_manager.ensure_brewing_in_dim(dim, bx, by, bz);
                let nbt = self_data.tile_manager.build_brewing_nbt_in_dim(dim, bx, by, bz);
                let bed_pk = BlockEntityDataPacket { x: bx, y: by as u32, z: bz, namedtag: nbt };
                broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, bed_pk.encode()).await;
            }
            63 | 68 => {
                self_data.tile_manager.ensure_sign_in_dim(dim, bx, by, bz);
                let nbt = self_data.tile_manager.build_sign_nbt_in_dim(dim, bx, by, bz);
                let bed_pk = BlockEntityDataPacket { x: bx, y: by as u32, z: bz, namedtag: nbt };
                broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, bed_pk.encode()).await;
            }
            26 => {
                let color = (item_meta & 0x0f) as u8;
                self_data.tile_manager.set_bed_color_in_dim(dim, bx, by, bz, color);
                let nbt = self_data.tile_manager.build_bed_nbt(bx, by, bz, color);
                let bed_pk = BlockEntityDataPacket { x: bx, y: by as u32, z: bz, namedtag: nbt };
                broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, bed_pk.encode()).await;
            }
            _ => {}
        }
    }

    for (cx, cy, cz, nid, nmeta) in cascaded_changes {
        let npk = UpdateBlockPacket::new(cx, cy as u32, cz, nid as u32, nmeta as u32, update_block::FLAG_ALL);
        broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, npk.encode()).await;
    }

    if is_survival {
        for (dx, dy, dz, item) in cascade_drops {
            let entity_id = {
                let mut eid = self_data.next_entity_id.write();
                let id = *eid;
                *eid += 1;
                id
            };
            self_data.entity_manager.spawn_item_drop_in_dim(
                entity_id,
                dim,
                item,
                dx as f64 + 0.5,
                dy as f64 + 0.5,
                dz as f64 + 0.5,
                &self_data.rak,
                &self_data.players,
            ).await;
        }
    }

    let place_sound = LevelSoundEventPacket::new_with_extra(
        level_event::SOUND_PLACE,
        primary_x as f32 + 0.5,
        primary_y as f32 + 0.5,
        primary_z as f32 + 0.5,
        primary_id as i32,
    );
    broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, place_sound.encode()).await;

    if matches!(primary_id, 8 | 9 | 10 | 11) {
        self_data.liquid_sim.schedule_update_dim(dim, primary_x, primary_y, primary_z);
    }

    // Golem pattern summoning on Pumpkin (86, 91) placement
    if (primary_id == 86 || primary_id == 91) && primary_y >= 2 {
        let mut cleared_blocks: Vec<(i32, i32, i32)> = Vec::new();
        let mut summon_type: Option<(i32, &str, f32)> = None;
        {
            let mut w = world_arc.write();
            if w.get_block_id(primary_x, primary_y - 1, primary_z) == 80
                && w.get_block_id(primary_x, primary_y - 2, primary_z) == 80
            {
                cleared_blocks = vec![
                    (primary_x, primary_y, primary_z),
                    (primary_x, primary_y - 1, primary_z),
                    (primary_x, primary_y - 2, primary_z),
                ];
                summon_type = Some((21, "Snow Golem", 4.0));
            } else if w.get_block_id(primary_x, primary_y - 1, primary_z) == 42
                && w.get_block_id(primary_x, primary_y - 2, primary_z) == 42
            {
                let arms_x = w.get_block_id(primary_x - 1, primary_y - 1, primary_z) == 42
                    && w.get_block_id(primary_x + 1, primary_y - 1, primary_z) == 42;
                let arms_z = w.get_block_id(primary_x, primary_y - 1, primary_z - 1) == 42
                    && w.get_block_id(primary_x, primary_y - 1, primary_z + 1) == 42;
                if arms_x || arms_z {
                    cleared_blocks = vec![
                        (primary_x, primary_y, primary_z),
                        (primary_x, primary_y - 1, primary_z),
                        (primary_x, primary_y - 2, primary_z),
                    ];
                    if arms_x {
                        cleared_blocks.push((primary_x - 1, primary_y - 1, primary_z));
                        cleared_blocks.push((primary_x + 1, primary_y - 1, primary_z));
                    } else {
                        cleared_blocks.push((primary_x, primary_y - 1, primary_z - 1));
                        cleared_blocks.push((primary_x, primary_y - 1, primary_z + 1));
                    }
                    summon_type = Some((20, "Iron Golem", 100.0));
                }
            }
            for &(cx, cy, cz) in &cleared_blocks {
                w.set_block(cx, cy, cz, 0, 0);
            }
        }
        for (cx, cy, cz) in cleared_blocks {
            let air_pk = UpdateBlockPacket::new(cx, cy as u32, cz, 0, 0, update_block::FLAG_ALL);
            broadcast_in_dim(&self_data.rak, &self_data.players, dim, None, air_pk.encode()).await;
        }
        if let Some((etype, ename, ehp)) = summon_type {
            let gid = {
                let mut eid = self_data.next_entity_id.write();
                let id = *eid;
                *eid += 1;
                id
            };
            self_data.entity_manager.spawn_mob_in_dim(
                gid,
                dim,
                etype,
                ename.to_string(),
                primary_x as f64 + 0.5,
                (primary_y - 2) as f64,
                primary_z as f64 + 0.5,
                ehp,
                &self_data.rak,
                &self_data.players,
            ).await;
        }
    }

    if is_survival {
        let updated_item = {
            let mut players = self_data.players.write();
            if let Some(state) = players.get_mut(&addr) {
                let mut it = state.inventory.get_item(held_slot);
                if it.count > 1 {
                    it.count -= 1;
                    state.inventory.set_item(held_slot, it.clone());
                    it
                } else {
                    let air = crate::protocol::types::Item::air();
                    state.inventory.set_item(held_slot, air.clone());
                    air
                }
            } else {
                crate::protocol::types::Item::air()
            }
        };
        let slot_pk = ContainerSetSlotPacket::new(0, held_slot as i32, updated_item);
        send_packet(&self_data.rak, addr, slot_pk.encode()).await;
    }
}

async fn send_player_inventory(self_data: &OxideServer, addr: SocketAddr) {
    let payloads = {
        let players = self_data.players.read();
        players.get(&addr).map(|s| {
            let mut pk = ContainerSetContentPacket::new(crate::protocol::types::ContainerIds::INVENTORY as u32);
            pk.target_eid = s.entity_id;
            pk.items = s.inventory.wire_contents();
            pk.hotbar = s.inventory.wire_hotbar_links();

            let mut arm_pk = ContainerSetContentPacket::new(crate::protocol::types::ContainerIds::ARMOR as u32);
            arm_pk.target_eid = s.entity_id;
            arm_pk.items = s.inventory.armor_contents().to_vec();

            let mut off_pk = ContainerSetContentPacket::new(crate::protocol::types::ContainerIds::OFFHAND as u32);
            off_pk.target_eid = s.entity_id;
            off_pk.items = vec![s.inventory.offhand()];

            vec![pk.encode(), arm_pk.encode(), off_pk.encode()]
        })
    };
    if let Some(pks) = payloads {
        let batch = BatchPacket::encode(&pks);
        send_raw(&self_data.rak, addr, batch).await;
    }
}

pub fn xp_needed_for_level(level: u32) -> u32 {
    if level <= 16 {
        2 * level + 7
    } else if level <= 31 {
        5 * level - 38
    } else {
        9 * level - 158
    }
}

async fn award_player_xp(self_data: &OxideServer, addr: SocketAddr, amount: u32) {
    if amount == 0 {
        return;
    }
    let update = {
        let mut players = self_data.players.write();
        if let Some(s) = players.get_mut(&addr) {
            s.xp_points += amount;
            let mut progress_xp = s.xp_progress * xp_needed_for_level(s.xp_level) as f32 + amount as f32;
            while progress_xp >= xp_needed_for_level(s.xp_level) as f32 {
                progress_xp -= xp_needed_for_level(s.xp_level) as f32;
                s.xp_level += 1;
            }
            s.xp_progress = (progress_xp / xp_needed_for_level(s.xp_level) as f32).clamp(0.0, 0.9999);
            Some((s.entity_id, s.xp_level as f32, s.xp_progress))
        } else {
            None
        }
    };
    if let Some((eid, lvl, prog)) = update {
        let mut attrs = UpdateAttributesPacket::new(eid);
        attrs.attributes = vec![
            Attribute::experience_level(lvl),
            Attribute::experience_progress(prog),
        ];
        send_packet(&self_data.rak, addr, attrs.encode()).await;
    }
}

async fn spawn_existing_players_to(self_data: &OxideServer, addr: SocketAddr) {
    let self_dim = self_data.players.read().get(&addr).map(|s| s.dimension).unwrap_or(0);
    let others: Vec<(i64, String, UUID, f64, f64, f64, f32, f32, crate::protocol::types::Item, [crate::protocol::types::Item; 4], EntityMetadata)> = {
        let guard = self_data.players.read();
        guard.iter()
            .filter(|(a, _)| **a != addr)
            .filter(|(_, s)| s.logged_in && s.dimension == self_dim)
            .map(|(_, s)| (
                s.entity_id, s.username.clone(), s.uuid, s.x, s.y, s.z, s.yaw, s.pitch,
                s.inventory.held_item(),
                [s.inventory.helmet(), s.inventory.chestplate(), s.inventory.leggings(), s.inventory.boots()],
                s.metadata.clone(),
            ))
            .collect()
    };
    for (eid, name, uuid, x, y, z, yaw, pitch, held_item, armor, meta) in others {
        let mut pk = AddPlayerPacket::new(uuid, name, eid, x as f32, (y + 1.62) as f32, z as f32);
        pk.yaw = yaw;
        pk.pitch = pitch;
        pk.item = held_item;
        pk.metadata = meta;
        send_packet(&self_data.rak, addr, pk.encode()).await;

        let mut arm = MobArmorEquipmentPacket::new(eid);
        arm.helmet = armor[0].clone();
        arm.chestplate = armor[1].clone();
        arm.leggings = armor[2].clone();
        arm.boots = armor[3].clone();
        send_packet(&self_data.rak, addr, arm.encode()).await;
    }
}

async fn announce_new_player_to_others(self_data: &OxideServer, addr: SocketAddr, entity_id: i64, username: String, uuid: UUID) {
    let (dim, x, y, z, yaw, pitch, held_item, armor, meta) = self_data.players.read().get(&addr)
        .map(|s| (
            s.dimension, s.x, s.y, s.z, s.yaw, s.pitch,
            s.inventory.held_item(),
            [s.inventory.helmet(), s.inventory.chestplate(), s.inventory.leggings(), s.inventory.boots()],
            s.metadata.clone(),
        ))
        .unwrap_or((0, 0.0, 64.0, 0.0, 0.0, 0.0, crate::protocol::types::Item::air(), Default::default(), EntityMetadata::new()));
    let mut pk = AddPlayerPacket::new(uuid, username, entity_id, x as f32, (y + 1.62) as f32, z as f32);
    pk.yaw = yaw;
    pk.pitch = pitch;
    pk.item = held_item;
    pk.metadata = meta;
    broadcast_in_dim(&self_data.rak, &self_data.players, dim, Some(addr), pk.encode()).await;

    let mut arm = MobArmorEquipmentPacket::new(entity_id);
    arm.helmet = armor[0].clone();
    arm.chestplate = armor[1].clone();
    arm.leggings = armor[2].clone();
    arm.boots = armor[3].clone();
    broadcast_in_dim(&self_data.rak, &self_data.players, dim, Some(addr), arm.encode()).await;
}

async fn broadcast_player_list_add(self_data: &OxideServer) {
    let entries: Vec<player_list::PlayerListEntry> = {
        let guard = self_data.players.read();
        guard.iter()
            .filter(|(_, s)| s.logged_in)
            .map(|(_, s)| player_list::PlayerListEntry {
                uuid: s.uuid,
                entity_id: s.entity_id,
                name: s.username.clone(),
                skin_id: s.skin_id.clone(),
                skin: s.skin.clone(),
            })
            .collect()
    };
    if entries.is_empty() { return; }
    let pk = PlayerListPacket::add(entries);
    broadcast_except(&self_data.rak, &self_data.players, None, pk.encode()).await;
}

/// Default chunk view distance when not specified in configuration.
/// Standard chunk view distance for mobile clients.
/// Keeps network throughput predictable.
const DEFAULT_VIEW_DISTANCE: i32 = 8;

async fn send_spawn_chunks(self_data: &OxideServer, addr: SocketAddr, cx: i32, cz: i32, radius: i32) {
    let center_cx = cx >> 4;
    let center_cz = cz >> 4;
    let mut wanted: Vec<(i32, i32)> = {
        let mut players = self_data.players.write();
        let Some(state) = players.get_mut(&addr) else { return };
        let mut list = Vec::new();
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let coord = (center_cx + dx, center_cz + dz);
                if state.sent_chunks.insert(coord) {
                    list.push(coord);
                }
            }
        }
        list
    };
    if wanted.is_empty() { return; }

    // Prioritize chunks closest to player spawn/position
    wanted.sort_by_key(|&(x, z)| {
        let dx = x - center_cx;
        let dz = z - center_cz;
        dx * dx + dz * dz
    });

    let dim = self_data.players.read().get(&addr).map(|s| s.dimension).unwrap_or(0);
    let target_world = self_data.get_world(dim);
    let total = wanted.len();

    for (i, (x, z)) in wanted.into_iter().enumerate() {
        let tiles = self_data.tile_manager.serialize_chunk_tiles_in_dim(dim, x, z);
        let chunk_data = {
            let mut world = target_world.write();
            world.serialize_chunk_with_tiles(x, z, &tiles)
        };
        let pk = FullChunkDataPacket::new(x, z, chunk_data).encode();
        send_raw(&self_data.rak, addr, BatchPacket::encode(&[pk])).await;
        if (i + 1) % 4 == 0 {
            tokio::task::yield_now().await;
        }
    }
    debug!("Sent chunks to player {}: {} (radius {})", addr, total, radius);
}

async fn send_packet(rak: &RakNetServer, addr: SocketAddr, data: Vec<u8>) {
    if let Some(&id) = data.first() {
        debug!("SEND -> 0x{:02x} ({} bytes) to {}", id, data.len(), addr);
    }
    let batch = BatchPacket::encode(&[data]);
    send_raw(rak, addr, batch).await;
}

async fn send_raw(rak: &RakNetServer, addr: SocketAddr, data: Vec<u8>) {
    rak.send_encapsulated(addr, data, true).await;
}

async fn broadcast_in_dim(
    rak: &RakNetServer,
    players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    dimension: i32,
    except: Option<SocketAddr>,
    data: Vec<u8>,
) {
    let addrs: Vec<SocketAddr> = players.read().iter()
        .filter(|(_, s)| s.logged_in && s.dimension == dimension)
        .filter(|(a, _)| Some(**a) != except)
        .map(|(a, _)| *a)
        .collect();
    if addrs.is_empty() {
        return;
    }
    let batch = BatchPacket::encode(&[data]);
    for addr in addrs {
        rak.send_encapsulated(addr, batch.clone(), true).await;
    }
}

async fn broadcast_except(
    rak: &RakNetServer,
    players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    except: Option<SocketAddr>,
    data: Vec<u8>,
) {
    let addrs: Vec<SocketAddr> = players.read().iter()
        .filter(|(_, s)| s.logged_in)
        .filter(|(a, _)| Some(**a) != except)
        .map(|(a, _)| *a)
        .collect();
    let batch = BatchPacket::encode(&[data]);
    for addr in addrs {
        rak.send_encapsulated(addr, batch.clone(), true).await;
    }
}

fn default_player_metadata() -> EntityMetadata {
    let mut m = EntityMetadata::new();
    m.set_long(0, 0);
    m.set_string(4, "".to_string());
    m.set_short(7, 400);
    m.set_short(43, 400);
    m.set_byte(27, 0);
    m.set_pos(29, 0, 0, 0);
    m.set_float(39, 1.0);
    m
}

fn load_or_create_config() -> Config {
    let path = std::env::current_dir().unwrap_or_default().join("oxide.json");
    if path.exists() {
        Config::load(&path).unwrap_or_default()
    } else {
        let mut c = Config::new();
        c.set("motd", serde_json::Value::String("Oxide Server".to_string()));
        c.set("max_players", serde_json::Value::Number(serde_json::Number::from(20)));
        c.set("server_id", serde_json::Value::String("Oxide".to_string()));
        c.set("online_count", serde_json::Value::Number(serde_json::Number::from(0)));
        c.set("view_distance", serde_json::Value::Number(serde_json::Number::from(DEFAULT_VIEW_DISTANCE)));
        c.set("gamemode", serde_json::Value::Number(serde_json::Number::from(0)));
        c.set("white_list", serde_json::Value::Bool(false));
        let _ = c.save(&path);
        c
    }
}

impl Clone for OxideServer {
    fn clone(&self) -> Self {
        Self {
            rak: self.rak.clone(),
            world: self.world.clone(),
            nether_world: self.nether_world.clone(),
            end_world: self.end_world.clone(),
            players: self.players.clone(),
            config: self.config.clone(),
            command_manager: self.command_manager.clone(),
            block_registry: self.block_registry.clone(),
            item_registry: self.item_registry.clone(),
            crafting_manager: self.crafting_manager.clone(),
            event_bus: self.event_bus.clone(),
            scheduler: self.scheduler.clone(),
            plugin_manager: self.plugin_manager.clone(),
            permission_manager: self.permission_manager.clone(),
            projectile_manager: self.projectile_manager.clone(),
            entity_manager: self.entity_manager.clone(),
            mob_spawner: self.mob_spawner.clone(),
            tile_manager: self.tile_manager.clone(),
            liquid_sim: self.liquid_sim.clone(),
            weather: self.weather.clone(),
            bans: self.bans.clone(),
            banned_ips: self.banned_ips.clone(),
            whitelist: self.whitelist.clone(),
            whitelist_enabled: self.whitelist_enabled.clone(),
            next_entity_id: self.next_entity_id.clone(),
            server_tick: self.server_tick.clone(),
            data_dir: self.data_dir.clone(),
            shutdown_notify: self.shutdown_notify.clone(),
            redstone_sched: self.redstone_sched.clone(),
        }
    }
}
