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
}

impl OxideServer {
    pub async fn new(addr: SocketAddr) -> anyhow::Result<Self> {
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
        let world = World::with_generator(&level_type, level_seed);
        let nether_world = World::with_generator("hell", level_seed.wrapping_add(100));
        let end_world = World::with_generator("ender", level_seed.wrapping_add(200));

        let block_registry = Arc::new(crate::block::BlockRegistry::default());
        let item_registry = Arc::new(crate::item::ItemRegistry::default());
        let crafting_manager = Arc::new(crate::inventory::CraftingManager::default());
        let event_bus = Arc::new(crate::event::EventBus::new());
        let scheduler = Arc::new(crate::scheduler::Scheduler::new());
        let plugin_manager = Arc::new(crate::plugin::PluginManager::new());
        plugin_manager.load_directory("plugins");
        let permission_manager = Arc::new(crate::permission::PermissionManager::new());

        Ok(Self {
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
            whitelist_enabled: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            next_entity_id: Arc::new(RwLock::new(1)),
        })
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
        };
        tokio::spawn(async move {
            tick_loop(tick_handles).await;
        });

        let event_self = self.clone();
        event_loop(event_self).await
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

        {
            let mut guard = self.players.write();
            if let Some(state) = guard.get_mut(&addr) {
                state.gamemode = mode;
            } else {
                return;
            }
        }

        let gt = SetPlayerGameTypePacket::new(mode);
        self.send_packet_to(addr, gt.encode());

        let mut settings = AdventureSettingsPacket::new();
        settings.world_builder = true;
        settings.allow_flight = mode == 1;
        settings.is_flying = false;
        settings.user_permission = adventure_settings::PERMISSION_NORMAL;
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
        let (entity_id, yaw, pitch) = {
            let mut guard = self.players.write();
            if let Some(state) = guard.get_mut(&addr) {
                state.x = x;
                state.y = y;
                state.z = z;
                (state.entity_id, state.yaw, state.pitch)
            } else {
                return;
            }
        };

        let mut mv = MovePlayerPacket::new(entity_id, x as f32, y as f32, z as f32, yaw, pitch);
        mv.mode = 1; // teleport/reset
        self.send_packet_to(addr, mv.encode());

        let pk = MoveEntityPacket::new(entity_id, x as f32, y as f32, z as f32, yaw, pitch);
        let rak = self.rak.clone();
        let players = self.players.clone();
        tokio::spawn(async move {
            broadcast_except(&rak, &players, Some(addr), pk.encode()).await;
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
        change_player_dimension_raw(
            &self.rak,
            &self.players,
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
    let eid = {
        let mut p = players.write();
        let Some(s) = p.get_mut(&addr) else { return; };
        s.dimension = new_dim;
        s.sent_chunks.clear();
        s.x = target_x;
        s.y = target_y;
        s.z = target_z;
        s.portal_ticks = 100;
        s.entity_id
    };

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
            .map(|&(x, z)| FullChunkDataPacket::new(x, z, w.serialize_chunk(x, z)).encode())
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
}

async fn tick_loop(h: TickHandles) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50));
    let mut tick_counter: u64 = 0;
    loop {
        interval.tick().await;
        tick_counter += 1;

        h.scheduler.tick(tick_counter);

        {
            h.world.write().tick();
            h.nether_world.write().tick();
            h.end_world.write().tick();
        }

        // Ticking projectiles (every tick)
        h.projectile_manager.tick(&h.world, &h.players, &h.rak, &h.next_entity_id).await;

        // Ticking active entities (every tick)
        h.entity_manager.tick(&h.world, &h.players, &h.rak, &h.next_entity_id).await;

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
                h.entity_manager.spawn_lightning(eid, lx, py, lz, &h.world, &h.rak, &h.players).await;
            }
        }

        // Ticking furnaces (every tick)
        h.tile_manager.tick_furnaces(&h.crafting_manager);

        // Ticking liquid physics (every 2 ticks = 10 updates/sec)
        if tick_counter % 2 == 0 {
            h.liquid_sim.tick(&h.world, &h.rak, &h.players).await;
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

async fn tick_player_food_and_health(
    players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    rak: &RakNetServer,
) {
    let mut updates: Vec<(SocketAddr, i64, u32, u32, f32)> = Vec::new();

    {
        let mut p = players.write();
        for (addr, s) in p.iter_mut() {
            if !s.logged_in || s.gamemode != 0 {
                continue;
            }

            if s.food_level >= 18 && s.health < 20 {
                s.health = (s.health + 1).min(20);
                if s.food_saturation > 0.0 {
                    s.food_saturation = (s.food_saturation - 1.0).max(0.0);
                } else {
                    s.food_level = s.food_level.saturating_sub(1);
                }
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

    for (addr, eid, px, py, pz, health, gamemode, dimension, portal_ticks, username) in list {
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

        // 3. Void check
        if py < -30.0 {
            let new_hp = health.saturating_sub(4);
            {
                let mut guard = players.write();
                if let Some(s) = guard.get_mut(&addr) {
                    s.health = new_hp;
                }
            }
            let hp_pk = SetHealthPacket::new(new_hp as i32);
            send_packet(rak, addr, hp_pk.encode()).await;

            if new_hp == 0 {
                let (sx, sy, sz) = {
                    let w = world.read();
                    (w.spawn_x, w.spawn_y, w.spawn_z)
                };
                {
                    let mut guard = players.write();
                    if let Some(s) = guard.get_mut(&addr) {
                        s.health = 20;
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

                let text = TextPacket::system(format!("{} fell out of the world", username));
                broadcast_except(rak, players, None, text.encode()).await;
            }
            continue;
        }

        // 4. Lava check
        if block_at_feet == 10 || block_at_feet == 11 || block_at_body == 10 || block_at_body == 11 {
            let new_hp = health.saturating_sub(4);
            {
                let mut guard = players.write();
                if let Some(s) = guard.get_mut(&addr) {
                    s.health = new_hp;
                }
            }
            let hp_pk = SetHealthPacket::new(new_hp as i32);
            send_packet(rak, addr, hp_pk.encode()).await;

            let hurt_sound = LevelSoundEventPacket::new(level_event::SOUND_HURT, px as f32, py as f32, pz as f32);
            broadcast_except(rak, players, None, hurt_sound.encode()).await;

            let anim = AnimatePacket::new(1, eid);
            broadcast_except(rak, players, None, anim.encode()).await;

            if new_hp == 0 {
                let (sx, sy, sz) = {
                    let w = world.read();
                    (w.spawn_x, w.spawn_y, w.spawn_z)
                };
                {
                    let mut guard = players.write();
                    if let Some(s) = guard.get_mut(&addr) {
                        s.health = 20;
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

                let text = TextPacket::system(format!("{} tried to swim in lava", username));
                broadcast_except(rak, players, None, text.encode()).await;
            }
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
            if !other.logged_in { continue; }
            let pk = MoveEntityPacket::new(
                state.entity_id,
                state.x as f32,
                state.y as f32,
                state.z as f32,
                state.yaw,
                state.pitch,
            );
            send_packet(rak, *other_addr, pk.encode()).await;
        }
    }
}

fn clone_state(s: &PlayerState) -> PlayerState {
    let inv = crate::inventory::inventories::PlayerInventory::new();
    inv.set_contents(s.inventory.get_contents());
    inv.set_held_index(s.inventory.held_index());
    for i in 0..4 {
        inv.set_armor(i, s.inventory.get_armor(i));
    }
    PlayerState {
        username: s.username.clone(),
        uuid: s.uuid,
        entity_id: s.entity_id,
        x: s.x, y: s.y, z: s.z,
        yaw: s.yaw, pitch: s.pitch,
        on_ground: s.on_ground,
        fall_distance: s.fall_distance,
        gamemode: s.gamemode,
        health: s.health,
        food_level: s.food_level,
        food_saturation: s.food_saturation,
        xp_points: s.xp_points,
        xp_level: s.xp_level,
        metadata: s.metadata.clone(),
        chunk_radius: s.chunk_radius,
        logged_in: s.logged_in,
        spawned: s.spawned,
        sent_chunks: s.sent_chunks.clone(),
        inventory: inv,
        open_container: s.open_container,
        dimension: s.dimension,
        portal_ticks: s.portal_ticks,
        effects: s.effects.clone(),
        food_tick_timer: s.food_tick_timer,
    }
}

async fn event_loop(self_data: OxideServer) -> anyhow::Result<()> {
    loop {
        let events = self_data.rak.drain_events().await;
        for ev in events {
            match ev {
                RakNetEvent::OpenSession(addr, client_id) => {
                    info!("Session opened: {} (client id {})", addr.get_ip(), client_id);
                }
                RakNetEvent::CloseSession(addr, reason) => {
                    info!("Session closed: {} ({})", addr.get_ip(), reason);
                    let removed = self_data.players.write().remove(&addr.to_socket_addr());
                    if let Some(state) = &removed {
                        self_data.event_bus.fire(crate::event::event::PlayerQuitEvent {
                            player_name: state.username.clone(),
                            reason: reason.clone(),
                        });
                        if state.logged_in {
                            let pk = RemoveEntityPacket::new(state.entity_id);
                            broadcast_except(&self_data.rak, &self_data.players, Some(addr.to_socket_addr()), pk.encode()).await;
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
            state.gamemode = 1;
            state.metadata = default_player_metadata();
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
                // Client may repeat COMPLETED; avoid duplicate spawn sequence
                // if player is already marked as spawned
                if self_data.players.read().get(&addr).map(|s| s.spawned).unwrap_or(false) {
                    debug!("Duplicate COMPLETED from {} - player already spawned, skipping", addr);
                    return;
                }
                let state_opt = self_data.players.read().get(&addr).map(|s| (
                    s.entity_id, s.username.clone(), s.gamemode, s.metadata.clone(), s.chunk_radius, s.uuid
                ));
                if let Some((entity_id, username, gamemode, metadata, chunk_radius, uuid)) = state_opt {
                    let (spawn_x, spawn_y, spawn_z) = {
                        let w = self_data.world.read();
                        (w.spawn_x, w.spawn_y, w.spawn_z)
                    };

                    let start = StartGamePacket {
                        entity_unique_id: entity_id,
                        entity_runtime_id: entity_id,
                        player_gamemode: gamemode,
                        x: spawn_x as f32 + 0.5,
                        y: (spawn_y as f32) + 1.62,
                        z: spawn_z as f32 + 0.5,
                        pitch: 0.0, yaw: 0.0,
                        seed: 0,
                        dimension: 0,
                        generator: 1,
                        world_gamemode: gamemode,
                        difficulty: 1,
                        spawn_x, spawn_y: spawn_y as u32, spawn_z,
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

                    // Complete login sequence: send game state and attributes
                    let time_pk = SetTimePacket::new(self_data.world.read().time);
                    send_packet(&self_data.rak, addr, time_pk.encode()).await;

                    let mut attrs = UpdateAttributesPacket::new(entity_id);
                    attrs.attributes = vec![
                        Attribute::health(20.0),
                        Attribute::movement_speed(0.1),
                        Attribute::hunger(20.0),
                        Attribute::food_saturation(20.0),
                        Attribute::experience_level(0.0),
                        Attribute::experience_progress(0.0),
                    ];
                    send_packet(&self_data.rak, addr, attrs.encode()).await;

                    // Send entity metadata
                    let data_pk = SetEntityDataPacket::new(entity_id, metadata);
                    send_packet(&self_data.rak, addr, data_pk.encode()).await;

                    // Send adventure settings (flight, world builder permissions)
                    // required so client does not hang in air and can place blocks
                    let mut settings = AdventureSettingsPacket::new();
                    settings.world_builder = true;
                    settings.allow_flight = gamemode == 1;
                    settings.user_permission = adventure_settings::PERMISSION_NORMAL;
                    send_packet(&self_data.rak, addr, settings.encode()).await;

                    let radius_pk = ChunkRadiusUpdatedPacket::new(chunk_radius);
                    send_packet(&self_data.rak, addr, radius_pk.encode()).await;

                    // Send initial spawn chunks (closest to spawn first)
                    send_spawn_chunks(self_data, addr, spawn_x, spawn_z, chunk_radius.min(4)).await;

                    if let Some(state) = self_data.players.write().get_mut(&addr) {
                        state.x = spawn_x as f64 + 0.5;
                        state.y = spawn_y as f64;
                        state.z = spawn_z as f64 + 0.5;
                        state.logged_in = true;
                        state.spawned = true;
                        state.on_ground = true;
                    }

                    let spawn_pos_pk = SetSpawnPositionPacket::new_player(spawn_x, spawn_y, spawn_z);
                    send_packet(&self_data.rak, addr, spawn_pos_pk.encode()).await;

                    let spawn_status = PlayStatusPacket::new(play_status::PLAYER_SPAWN);
                    send_packet(&self_data.rak, addr, spawn_status.encode()).await;

                    let available_cmds = self_data.command_manager.generate_available_commands_packet();
                    send_packet(&self_data.rak, addr, available_cmds.encode()).await;

                    // Synchronize weather state with player
                    self_data.weather.send_to_player(addr, &self_data.rak).await;

                    let crafting_pk = CraftingDataPacket::with_default_recipes();
                    send_packet(&self_data.rak, addr, crafting_pk.encode()).await;

                    let mut creative_pk = ContainerSetContentPacket::new(crate::protocol::types::ContainerIds::CREATIVE as u32);
                    creative_pk.target_eid = entity_id;
                    creative_pk.items = crate::item::creative::get_creative_items().to_vec();
                    send_packet(&self_data.rak, addr, creative_pk.encode()).await;

                    let mut content_pk = ContainerSetContentPacket::new(crate::protocol::types::ContainerIds::INVENTORY as u32);
                    content_pk.target_eid = entity_id;
                    content_pk.items = self_data.players.read().get(&addr)
                        .map(|s| s.inventory.get_contents())
                        .unwrap_or_default();
                    send_packet(&self_data.rak, addr, content_pk.encode()).await;

                    spawn_existing_players_to(self_data, addr).await;
                    announce_new_player_to_others(self_data, addr, entity_id, username.clone(), uuid).await;
                    broadcast_player_list_add(self_data).await;

                    let join_msg = TextPacket::system(format!("{} joined the game", username));
                    broadcast_except(&self_data.rak, &self_data.players, None, join_msg.encode()).await;

                    self_data.event_bus.fire(crate::event::event::PlayerJoinEvent {
                        player_name: username.clone(),
                    });

                    // Stream remaining chunks up to full radius in background
                    if chunk_radius > 4 {
                        let server_clone = self_data.clone();
                        tokio::spawn(async move {
                            send_spawn_chunks(&server_clone, addr, spawn_x, spawn_z, chunk_radius).await;
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
            if let Some(state) = self_data.players.write().get_mut(&addr) {
                state.chunk_radius = radius;
            }
            let pk = ChunkRadiusUpdatedPacket::new(radius);
            send_packet(&self_data.rak, addr, pk.encode()).await;
            let (sx, sz) = {
                let w = self_data.world.read();
                (w.spawn_x, w.spawn_z)
            };
            let server_clone = self_data.clone();
            tokio::spawn(async move {
                send_spawn_chunks(&server_clone, addr, sx, sz, radius).await;
            });
        }
        IncomingPacket::MovePlayer(mv) => {
            let mut chunk_to_load = None;
            let mut fall_damage = 0;
            let mut eid = 0;
            let mut cur_hp = 20;
            if let Some(state) = self_data.players.write().get_mut(&addr) {
                let old_cx = (state.x as i32) >> 4;
                let old_cz = (state.z as i32) >> 4;
                let old_y = state.y;
                eid = state.entity_id;

                state.x = mv.x as f64;
                state.y = (mv.y - 1.62) as f64;
                state.z = mv.z as f64;
                state.yaw = mv.yaw;
                state.pitch = mv.pitch;

                if mv.on_ground {
                    if state.fall_distance > 3.0 && state.gamemode == 0 {
                        fall_damage = (state.fall_distance - 3.0).floor() as u32;
                        state.health = state.health.saturating_sub(fall_damage);
                        cur_hp = state.health;
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

            if fall_damage > 0 {
                let hp_pk = SetHealthPacket::new(cur_hp as i32);
                send_packet(&self_data.rak, addr, hp_pk.encode()).await;

                let sound_id = if fall_damage > 4 { level_event::SOUND_FALL_BIG } else { level_event::SOUND_FALL_SMALL };
                let fall_sound = LevelSoundEventPacket::new(sound_id, mv.x, mv.y, mv.z);
                broadcast_except(&self_data.rak, &self_data.players, None, fall_sound.encode()).await;

                let anim = AnimatePacket::new(1, eid);
                broadcast_except(&self_data.rak, &self_data.players, None, anim.encode()).await;
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
            match action.action {
                player_action::ACTION_START_BREAK => {
                    let punch_pk = LevelEventPacket::new(
                        level_event::EVENT_PARTICLE_PUNCH_BLOCK,
                        action.x as f32 + 0.5,
                        action.y as f32 + 0.5,
                        action.z as f32 + 0.5,
                        action.face,
                    );
                    broadcast_except(&self_data.rak, &self_data.players, Some(addr), punch_pk.encode()).await;
                }
                player_action::ACTION_ABORT_BREAK => {
                    let stop_pk = LevelEventPacket::new(
                        level_event::EVENT_BLOCK_STOP_BREAK,
                        action.x as f32 + 0.5,
                        action.y as f32 + 0.5,
                        action.z as f32 + 0.5,
                        0,
                    );
                    broadcast_except(&self_data.rak, &self_data.players, Some(addr), stop_pk.encode()).await;
                }
                player_action::ACTION_STOP_BREAK => {
                    handle_break_block(self_data, addr, action.x, action.y as i32, action.z).await;
                }
                player_action::ACTION_RELEASE_ITEM => {
                    let held_item = self_data.players.read().get(&addr).map(|s| s.inventory.held_item()).unwrap_or_else(crate::protocol::types::Item::air);
                    if held_item.id == 261 { // Bow
                        let proj_eid = {
                            let mut guard = self_data.next_entity_id.write();
                            let id = *guard;
                            *guard += 1;
                            id
                        };
                        let p_info = self_data.players.read().get(&addr).map(|s| (s.x, s.y, s.z, s.yaw, s.pitch, s.entity_id));
                        if let Some((px, py, pz, yaw, pitch, eid)) = p_info {
                            let yaw_rad = (yaw as f64).to_radians();
                            let pitch_rad = (pitch as f64).to_radians();
                            let speed = 2.2f32;
                            let vx = (-yaw_rad.sin() * pitch_rad.cos() * speed as f64) as f32;
                            let vy = (-pitch_rad.sin() * speed as f64) as f32;
                            let vz = (yaw_rad.cos() * pitch_rad.cos() * speed as f64) as f32;

                            self_data.entity_manager.spawn_projectile(
                                proj_eid,
                                80, // Arrow
                                px,
                                py + 1.5,
                                pz,
                                vx, vy, vz,
                                yaw, pitch,
                                eid,
                                &self_data.rak,
                                &self_data.players,
                            ).await;

                            let bow_snd = LevelSoundEventPacket::new(level_event::SOUND_BOW, px as f32, py as f32, pz as f32);
                            broadcast_except(&self_data.rak, &self_data.players, None, bow_snd.encode()).await;
                        }
                    }
                }
                player_action::ACTION_SPAWN_SAME_DIMENSION | player_action::ACTION_SPAWN_OVERWORLD => {
                    let (sx, sy, sz) = {
                        let w = self_data.world.read();
                        (w.spawn_x, w.spawn_y, w.spawn_z)
                    };
                    let eid = {
                        let mut p = self_data.players.write();
                        if let Some(s) = p.get_mut(&addr) {
                            s.health = 20;
                            s.food_level = 20;
                            s.food_saturation = 5.0;
                            s.x = sx as f64 + 0.5;
                            s.y = sy as f64 + 0.5;
                            s.z = sz as f64 + 0.5;
                            s.entity_id
                        } else {
                            0
                        }
                    };
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
            if interact.action == crate::protocol::packets::interact::ACTION_LEFT_CLICK {
                let held_item = self_data.players.read().get(&addr).map(|s| s.inventory.held_item()).unwrap_or_else(crate::protocol::types::Item::air);
                let base_damage = match held_item.id {
                    276 => 7.0, // Diamond sword
                    267 => 6.0, // Iron sword
                    272 => 5.0, // Stone sword
                    268 | 283 => 4.0, // Wood / Gold sword
                    279 => 6.0, // Diamond axe
                    258 => 5.0, // Iron axe
                    _ => 1.0,
                };
                let enchs = crate::item::enchantment::parse_enchantments_from_item(&held_item);
                let bonus = crate::item::enchantment::calculate_melee_damage_bonus(&enchs, false, false);
                let total_damage = base_damage + bonus;
                let attacker_eid = self_data.players.read().get(&addr).map(|s| s.entity_id).unwrap_or(0);

                let attacker_pos = self_data.players.read().get(&addr).map(|s| (s.x, s.y, s.z));
                let _killed = self_data.entity_manager.damage_entity(interact.target, total_damage, attacker_eid, attacker_pos, &self_data.rak, &self_data.players).await;

                // PvP Player damage
                let target_player = {
                    let p = self_data.players.read();
                    p.iter().find(|(_, s)| s.entity_id == interact.target).map(|(a, s)| (*a, s.health, s.gamemode, s.username.clone()))
                };
                if let Some((target_addr, target_hp, gm, target_name)) = target_player {
                    if gm == 0 && target_addr != addr {
                        let new_hp = (target_hp as f32 - total_damage).max(0.0) as u32;
                        {
                            let mut p = self_data.players.write();
                            if let Some(s) = p.get_mut(&target_addr) {
                                s.health = new_hp;
                            }
                        }
                        let hp_pk = SetHealthPacket::new(new_hp as i32);
                        self_data.send_packet_to(target_addr, hp_pk.encode());

                        let anim = AnimatePacket::new(1, interact.target);
                        let hurt_snd = LevelSoundEventPacket::new(level_event::SOUND_HIT, 0.0, 0.0, 0.0);
                        broadcast_except(&self_data.rak, &self_data.players, None, anim.encode()).await;
                        broadcast_except(&self_data.rak, &self_data.players, None, hurt_snd.encode()).await;

                        if new_hp == 0 {
                            let attacker_name = self_data.players.read().get(&addr).map(|s| s.username.clone()).unwrap_or_default();
                            self_data.broadcast_message(&format!("{} was slain by {}", target_name, attacker_name));
                        }
                    }
                }
            }
        }
        IncomingPacket::MobEquipment(me) => {
            if let Some(state) = self_data.players.write().get_mut(&addr) {
                state.inventory.set_held_index(me.hotbar_slot as usize);
            }
            let pk = MobEquipmentPacket {
                entity_runtime_id: self_data.players.read().get(&addr).map(|s| s.entity_id).unwrap_or(0),
                item: me.item,
                inventory_slot: me.inventory_slot,
                hotbar_slot: me.hotbar_slot,
                window_id: me.window_id,
            };
            broadcast_except(&self_data.rak, &self_data.players, Some(addr), pk.encode()).await;
        }
        IncomingPacket::ContainerSetSlot(slot_pk) => {
            if slot_pk.window_id == 0 {
                let slot = slot_pk.slot as usize;
                if let Some(state) = self_data.players.write().get_mut(&addr) {
                    if slot < 36 {
                        state.inventory.set_item(slot, slot_pk.item);
                    }
                }
            } else if slot_pk.window_id == 1 {
                let open_pos = self_data.players.read().get(&addr).and_then(|s| s.open_container);
                if let Some((cx, cy, cz)) = open_pos {
                    let block_id = { self_data.world.read().get_block_id_opt(cx, cy, cz) };
                    if block_id == 54 {
                        let viewers = self_data.tile_manager.set_chest_slot(cx, cy, cz, slot_pk.slot as usize, slot_pk.item.clone());
                        let sync_pk = ContainerSetSlotPacket::new(1, slot_pk.slot, slot_pk.item);
                        for viewer in viewers {
                            if viewer != addr {
                                self_data.send_packet_to(viewer, sync_pk.encode());
                            }
                        }
                    } else if block_id == 61 || block_id == 62 {
                        self_data.tile_manager.set_furnace_slot(cx, cy, cz, slot_pk.slot as usize, slot_pk.item);
                    }
                }
            }
        }
        IncomingPacket::ContainerClose(close_pk) => {
            if close_pk.window_id == 1 {
                {
                    let mut p = self_data.players.write();
                    if let Some(s) = p.get_mut(&addr) {
                        s.open_container = None;
                    }
                }
                let closed_chests = self_data.tile_manager.close_all_for_player(addr);
                for ((cx, cy, cz), was_last) in closed_chests {
                    if was_last {
                        let block_ev = BlockEventPacket { x: cx, y: cy as u32, z: cz, case_1: 1, case_2: 0 };
                        let sound = LevelSoundEventPacket::new(level_event::SOUND_CHEST_CLOSED, cx as f32 + 0.5, cy as f32 + 0.5, cz as f32 + 0.5);
                        broadcast_except(&self_data.rak, &self_data.players, None, block_ev.encode()).await;
                        broadcast_except(&self_data.rak, &self_data.players, None, sound.encode()).await;
                    }
                }
            }
        }
        IncomingPacket::CraftingEvent(craft_pk) => {
            let recipe_opt = self_data.crafting_manager.match_shapeless(&craft_pk.input)
                .or_else(|| self_data.crafting_manager.match_shaped(&craft_pk.input, 3, 3));
            if let Some(_recipe) = recipe_opt {
                if let Some(state) = self_data.players.write().get_mut(&addr) {
                    for out in &craft_pk.result {
                        let _ = state.inventory.add_item(out.clone());
                    }
                }
            } else if !craft_pk.result.is_empty() {
                if let Some(state) = self_data.players.write().get_mut(&addr) {
                    for out in &craft_pk.result {
                        let _ = state.inventory.add_item(out.clone());
                    }
                }
            }
            let inv_items = self_data.players.read().get(&addr).map(|s| s.inventory.get_contents());
            if let Some(items) = inv_items {
                let mut content_pk = ContainerSetContentPacket::new(crate::protocol::types::ContainerIds::INVENTORY as u32);
                content_pk.target_eid = self_data.players.read().get(&addr).map(|s| s.entity_id).unwrap_or(0);
                content_pk.items = items;
                send_packet(&self_data.rak, addr, content_pk.encode()).await;
            }
        }
        IncomingPacket::Animate(anim) => {
            let eid = self_data.players.read().get(&addr).map(|s| s.entity_id).unwrap_or(0);
            let mut pk = anim;
            pk.entity_runtime_id = eid;
            broadcast_except(&self_data.rak, &self_data.players, Some(addr), pk.encode()).await;
        }
        IncomingPacket::DropItem(drop_pk) => {
            if !drop_pk.item.is_air() {
                let eid = {
                    let mut id_guard = self_data.next_entity_id.write();
                    let id = *id_guard;
                    *id_guard += 1;
                    id
                };
                let player_info = self_data.players.read().get(&addr).map(|s| (s.x, s.y, s.z, s.yaw, s.pitch));
                if let Some((px, py, pz, yaw, pitch)) = player_info {
                    let yaw_rad = (yaw as f64).to_radians();
                    let pitch_rad = (pitch as f64).to_radians();
                    let speed_x = (-yaw_rad.sin() * pitch_rad.cos() * 0.4) as f32;
                    let speed_y = (-pitch_rad.sin() * 0.4 + 0.1) as f32;
                    let speed_z = (yaw_rad.cos() * pitch_rad.cos() * 0.4) as f32;

                    let mut add_item = AddItemEntityPacket::new(
                        eid,
                        drop_pk.item.clone(),
                        px as f32,
                        (py + 1.3) as f32,
                        pz as f32,
                    );
                    add_item.speed_x = speed_x;
                    add_item.speed_y = speed_y;
                    add_item.speed_z = speed_z;
                    broadcast_except(&self_data.rak, &self_data.players, None, add_item.encode()).await;
                }
            }
        }
        IncomingPacket::PlayerInput(pi) => {
            if let Some(state) = self_data.players.write().get_mut(&addr) {
                state.metadata.set_generic_flag(EntityMetadata::DATA_FLAG_SNEAKING, pi.sneaking);
            }
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
            let block_id = {
                let mut w = self_data.world.write();
                w.get_block_id(bp.x, bp.y as i32, bp.z)
            };
            if block_id != 0 {
                let item = crate::protocol::types::Item::new(block_id as i32, 0, 1);
                let held_slot = {
                    let mut players = self_data.players.write();
                    if let Some(s) = players.get_mut(&addr) {
                        let slot = s.inventory.held_index();
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
            // Sign text updates and block entity sync to other players
            broadcast_except(&self_data.rak, &self_data.players, Some(addr), bed.encode()).await;
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

    // Reach distance check
    let player_pos = self_data.players.read().get(&addr).map(|s| (s.x, s.y, s.z));
    if let Some((px, py, pz)) = player_pos {
        let dx = px - (x as f64 + 0.5);
        let dy = py - (y as f64 + 0.5);
        let dz = pz - (z as f64 + 0.5);
        if dx*dx + dy*dy + dz*dz > 64.0 {
            return;
        }
    }

    let (player_name, old_id, old_meta) = {
        let mut world = self_data.world.write();
        let id = world.get_block_id(x, y, z);
        if id == 0 || id == 7 { // cannot break air or bedrock
            return;
        }
        let chunk_x = x >> 4;
        let chunk_z = z >> 4;
        let cx = (x & 0x0f) as usize;
        let cz = (z & 0x0f) as usize;
        let chunk = world.get_chunk_or_generate(chunk_x, chunk_z);
        let meta = chunk.get_block_data(cx, y as usize, cz);

        let name = self_data.players.read().get(&addr).map(|s| s.username.clone()).unwrap_or_default();
        let ev = self_data.event_bus.fire(crate::event::event::BlockBreakEvent {
            player_name: name.clone(),
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
        (name, id, meta)
    };
    let _ = player_name;

    // Remove tile entity data and drop its stored items
    let tile_drops = self_data.tile_manager.remove_tile(x, y, z);
    for drop_it in tile_drops {
        let drop_eid = {
            let mut eid_guard = self_data.next_entity_id.write();
            let id = *eid_guard;
            *eid_guard += 1;
            id
        };
        self_data.entity_manager.spawn_item_drop(
            drop_eid,
            drop_it,
            x as f64 + 0.5,
            y as f64 + 0.5,
            z as f64 + 0.5,
            &self_data.rak,
            &self_data.players,
        ).await;
    }

    let pk = UpdateBlockPacket::new(x, y as u32, z, 0, 0, update_block::FLAG_ALL);
    broadcast_except(&self_data.rak, &self_data.players, None, pk.encode()).await;

    // Block break particle and sound effect
    let destroy_pk = LevelEventPacket::new(
        level_event::EVENT_PARTICLE_DESTROY,
        x as f32 + 0.5,
        y as f32 + 0.5,
        z as f32 + 0.5,
        (old_id as i32) | ((old_meta as i32) << 8),
    );
    broadcast_except(&self_data.rak, &self_data.players, None, destroy_pk.encode()).await;

    let break_sound = LevelSoundEventPacket::new_with_extra(
        level_event::SOUND_BREAK,
        x as f32 + 0.5,
        y as f32 + 0.5,
        z as f32 + 0.5,
        old_id as i32,
    );
    broadcast_except(&self_data.rak, &self_data.players, None, break_sound.encode()).await;

    // Gravity check for sand (12) and gravel (13) directly above
    if y + 1 < 256 {
        let above_id = {
            let mut world = self_data.world.write();
            world.get_block_id(x, y + 1, z)
        };
        if above_id == 12 || above_id == 13 {
            let mut fall_y = y;
            {
                let mut world = self_data.world.write();
                while fall_y > 0 && world.get_block_id(x, fall_y - 1, z) == 0 {
                    fall_y -= 1;
                }
                world.set_block(x, y + 1, z, 0, 0);
                world.set_block(x, fall_y, z, above_id, 0);
            }
            let clear_pk = UpdateBlockPacket::new(x, (y + 1) as u32, z, 0, 0, update_block::FLAG_ALL);
            broadcast_except(&self_data.rak, &self_data.players, None, clear_pk.encode()).await;

            let place_pk = UpdateBlockPacket::new(x, fall_y as u32, z, above_id as u32, 0, update_block::FLAG_ALL);
            broadcast_except(&self_data.rak, &self_data.players, None, place_pk.encode()).await;
        }
    }

    // Schedule liquid updates for neighboring liquids
    let neighbors = [(x - 1, y, z), (x + 1, y, z), (x, y - 1, z), (x, y + 1, z), (x, y, z - 1), (x, y, z + 1)];
    for (nx, ny, nz) in neighbors {
        if ny >= 0 && ny < 256 {
            let nid = {
                let mut world = self_data.world.write();
                world.get_block_id(nx, ny, nz)
            };
            if nid == 8 || nid == 9 || nid == 10 || nid == 11 {
                self_data.liquid_sim.schedule_update(nx, ny, nz);
            }
        }
    }

    // Spawn block drop item entity if survival
    let is_survival = self_data.players.read().get(&addr).map(|s| s.gamemode == 0).unwrap_or(true);
    if is_survival {
        let drops = get_block_drops(old_id, old_meta);
        for drop_item in drops {
            let entity_id = {
                let mut eid = self_data.next_entity_id.write();
                let id = *eid;
                *eid += 1;
                id
            };
            self_data.entity_manager.spawn_item_drop(
                entity_id,
                drop_item,
                x as f64 + 0.5,
                y as f64 + 0.5,
                z as f64 + 0.5,
                &self_data.rak,
                &self_data.players,
            ).await;
        }
    }
}

fn get_block_drops(block_id: u8, meta: u8) -> Vec<crate::protocol::types::Item> {
    use crate::protocol::types::Item;
    match block_id {
        0 | 7 | 8 | 9 | 10 | 11 | 90 | 119 => vec![],
        20 | 79 | 95 | 102 | 160 | 52 => vec![],
        1 => vec![Item::new(4, 0, 1)], // Stone -> Cobblestone
        2 => vec![Item::new(3, 0, 1)], // Grass -> Dirt
        16 => vec![Item::new(263, 0, 1)], // Coal Ore -> Coal
        56 => vec![Item::new(264, 0, 1)], // Diamond Ore -> Diamond
        129 => vec![Item::new(388, 0, 1)], // Emerald Ore -> Emerald
        153 => vec![Item::new(406, 0, 1)], // Quartz Ore -> Quartz
        21 => vec![Item::new(351, 4, 5)], // Lapis Ore -> 5 Lapis
        73 | 74 => vec![Item::new(331, 0, 4)], // Redstone Ore -> 4 Redstone
        89 => vec![Item::new(348, 0, 3)], // Glowstone -> 3 Dust
        82 => vec![Item::new(337, 0, 4)], // Clay Block -> 4 Clay Balls
        47 => vec![Item::new(340, 0, 3)], // Bookshelf -> 3 Books
        103 => vec![Item::new(360, 0, 5)], // Melon Block -> 5 Slices
        59 => if meta >= 7 { vec![Item::new(296, 0, 1), Item::new(295, 0, 2)] } else { vec![Item::new(295, 0, 1)] },
        141 => if meta >= 7 { vec![Item::new(391, 0, 3)] } else { vec![Item::new(391, 0, 1)] },
        142 => if meta >= 7 { vec![Item::new(392, 0, 3)] } else { vec![Item::new(392, 0, 1)] },
        244 => if meta >= 7 { vec![Item::new(434, 0, 1), Item::new(435, 0, 2)] } else { vec![Item::new(435, 0, 1)] },
        13 => if rand::random::<f32>() < 0.15 { vec![Item::new(318, 0, 1)] } else { vec![Item::new(13, 0, 1)] },
        18 | 161 => {
            let r = rand::random::<f32>();
            if r < 0.05 { vec![Item::new(6, (meta & 0x03) as i32, 1)] }
            else if r < 0.06 { vec![Item::new(260, 0, 1)] }
            else { vec![] }
        }
        _ => vec![Item::new(block_id as i32, meta as i32, 1)],
    }
}

async fn handle_use_item(self_data: &OxideServer, addr: SocketAddr, ui: UseItemPacket) {
    let player_info = self_data.players.read().get(&addr).map(|s| {
        (s.x, s.y, s.z, s.yaw, s.pitch, s.entity_id, s.gamemode == 0, s.health, s.food_level, s.inventory.held_index())
    });
    let (px, py, pz, yaw, pitch, eid, is_survival, _cur_health, cur_food, held_slot) = match player_info {
        Some(info) => info,
        None => return,
    };

    let item_id = ui.item.id;
    let item_meta = ui.item.meta;

    // 1. Projectiles: Ender Pearl (368), Snowball (332), Egg (344), Bow (261), Exp Bottle (384)
    let projectile_kind = match item_id {
        368 => Some((crate::entity::ProjectileKind::EnderPearl, 1.5)),
        332 => Some((crate::entity::ProjectileKind::Snowball, 1.5)),
        344 => Some((crate::entity::ProjectileKind::Egg, 1.5)),
        261 => Some((crate::entity::ProjectileKind::Arrow, 2.0)),
        384 => Some((crate::entity::ProjectileKind::ExpBottle, 0.7)),
        _ => None,
    };

    if let Some((kind, force)) = projectile_kind {
        let proj_eid = {
            let mut guard = self_data.next_entity_id.write();
            let id = *guard;
            *guard += 1;
            id
        };

        self_data.projectile_manager.launch(
            proj_eid,
            kind,
            addr,
            eid,
            px,
            py,
            pz,
            yaw,
            pitch,
            force,
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
        broadcast_except(&self_data.rak, &self_data.players, None, drink_snd.encode()).await;

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
                        s.inventory.set_item(held_slot, crate::protocol::types::Item::new(374, 0, 1)); // Glass bottle
                    }
                    s.health
                } else {
                    20
                }
            };

            let hp_pk = SetHealthPacket::new(new_hp as i32);
            send_packet(&self_data.rak, addr, hp_pk.encode()).await;

            let drink_snd = LevelSoundEventPacket::new(level_event::SOUND_DRINK, px as f32, py as f32, pz as f32);
            broadcast_except(&self_data.rak, &self_data.players, None, drink_snd.encode()).await;

            for eff in effects_to_send {
                let eff_pk = MobEffectPacket {
                    entity_runtime_id: eid,
                    event_id: 1, // EVENT_ADD
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

    // 3. Food items
    let food_info = match item_id {
        260 => Some((4, 2.4f32)),  // Apple
        297 => Some((5, 6.0f32)),  // Bread
        319 => Some((3, 1.8f32)),  // Raw Porkchop
        320 => Some((8, 12.8f32)), // Cooked Porkchop
        322 => Some((4, 9.6f32)),  // Golden Apple
        363 => Some((3, 1.8f32)),  // Raw Beef
        364 => Some((8, 12.8f32)), // Steak
        365 => Some((2, 1.2f32)),  // Raw Chicken
        366 => Some((6, 7.2f32)),  // Cooked Chicken
        367 => Some((4, 0.8f32)),  // Rotten Flesh
        391 => Some((1, 0.6f32)),  // Potato
        393 => Some((5, 6.0f32)),  // Baked Potato
        396 => Some((6, 14.4f32)), // Golden Carrot
        411 => Some((2, 1.2f32)),  // Raw Mutton
        412 => Some((6, 9.6f32)),  // Cooked Mutton
        466 => Some((4, 9.6f32)),  // Enchanted Golden Apple
        _ => None,
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
                        // Golden Apple: Regen II (100 ticks), Absorption I (2400 ticks)
                        let eff1 = crate::entity::effect::ActiveEffect::new(crate::entity::effect::effect_id::REGENERATION, 100, 1);
                        let eff2 = crate::entity::effect::ActiveEffect::new(crate::entity::effect::effect_id::ABSORPTION, 2400, 0);
                        s.effects.add_effect(eff1.clone());
                        s.effects.add_effect(eff2.clone());
                        effects_to_send.push(eff1);
                        effects_to_send.push(eff2);
                    } else if item_id == 466 {
                        // Enchanted Golden Apple: Regen V (600 ticks), Resistance I (6000 ticks), Fire Resistance I (6000 ticks), Absorption IV (2400 ticks)
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
                        // Rotten Flesh: 80% Hunger
                        let eff = crate::entity::effect::ActiveEffect::new(crate::entity::effect::effect_id::HUNGER, 600, 0);
                        s.effects.add_effect(eff.clone());
                        effects_to_send.push(eff);
                    } else if item_id == 365 && rand::random::<f32>() < 0.3 {
                        // Raw Chicken: 30% Hunger
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

            // Send effect packets to client
            for eff in effects_to_send {
                let eff_pk = MobEffectPacket {
                    entity_runtime_id: eid,
                    event_id: 1, // EVENT_ADD
                    effect_id: eff.id as i32,
                    amplifier: eff.amplifier as i32,
                    particles: eff.show_particles,
                    duration: eff.duration,
                };
                send_packet(&self_data.rak, addr, eff_pk.encode()).await;
            }

            let eat_sound = LevelSoundEventPacket::new(level_event::SOUND_EAT, px as f32, py as f32, pz as f32);
            let burp_sound = LevelSoundEventPacket::new(level_event::SOUND_BURP, px as f32, py as f32, pz as f32);
            broadcast_except(&self_data.rak, &self_data.players, None, eat_sound.encode()).await;
            broadcast_except(&self_data.rak, &self_data.players, None, burp_sound.encode()).await;

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

    // 3. Bucket handling: Water Bucket (326 or 325:8/9), Lava Bucket (327 or 325:10/11), Empty Bucket (325:0)
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
                    let mut w = self_data.world.write();
                    w.set_block(nx, ny, nz, 8, 0);
                }
                let pk = UpdateBlockPacket::new(nx, ny as u32, nz, 8, 0, update_block::FLAG_ALL);
                broadcast_except(&self_data.rak, &self_data.players, None, pk.encode()).await;
                self_data.liquid_sim.schedule_update(nx, ny, nz);

                let sound = LevelSoundEventPacket::new(level_event::SOUND_BUCKET_EMPTY_WATER, nx as f32 + 0.5, ny as f32 + 0.5, nz as f32 + 0.5);
                broadcast_except(&self_data.rak, &self_data.players, None, sound.encode()).await;

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
                    let mut w = self_data.world.write();
                    w.set_block(nx, ny, nz, 10, 0);
                }
                let pk = UpdateBlockPacket::new(nx, ny as u32, nz, 10, 0, update_block::FLAG_ALL);
                broadcast_except(&self_data.rak, &self_data.players, None, pk.encode()).await;
                self_data.liquid_sim.schedule_update(nx, ny, nz);

                let sound = LevelSoundEventPacket::new(level_event::SOUND_BUCKET_EMPTY_LAVA, nx as f32 + 0.5, ny as f32 + 0.5, nz as f32 + 0.5);
                broadcast_except(&self_data.rak, &self_data.players, None, sound.encode()).await;

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
                    let mut w = self_data.world.write();
                    w.get_block_id(ui.x, ui.y as i32, ui.z)
                };
                if target_id == 8 || target_id == 9 {
                    {
                        let mut w = self_data.world.write();
                        w.set_block(ui.x, ui.y as i32, ui.z, 0, 0);
                    }
                    let pk = UpdateBlockPacket::new(ui.x, ui.y, ui.z, 0, 0, update_block::FLAG_ALL);
                    broadcast_except(&self_data.rak, &self_data.players, None, pk.encode()).await;

                    let sound = LevelSoundEventPacket::new(level_event::SOUND_BUCKET_FILL_WATER, ui.x as f32 + 0.5, ui.y as f32 + 0.5, ui.z as f32 + 0.5);
                    broadcast_except(&self_data.rak, &self_data.players, None, sound.encode()).await;

                    // Wake up surrounding liquid
                    self_data.liquid_sim.schedule_update(ui.x - 1, ui.y as i32, ui.z);
                    self_data.liquid_sim.schedule_update(ui.x + 1, ui.y as i32, ui.z);
                    self_data.liquid_sim.schedule_update(ui.x, ui.y as i32, ui.z - 1);
                    self_data.liquid_sim.schedule_update(ui.x, ui.y as i32, ui.z + 1);

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
                        let mut w = self_data.world.write();
                        w.set_block(ui.x, ui.y as i32, ui.z, 0, 0);
                    }
                    let pk = UpdateBlockPacket::new(ui.x, ui.y, ui.z, 0, 0, update_block::FLAG_ALL);
                    broadcast_except(&self_data.rak, &self_data.players, None, pk.encode()).await;

                    let sound = LevelSoundEventPacket::new(level_event::SOUND_BUCKET_FILL_LAVA, ui.x as f32 + 0.5, ui.y as f32 + 0.5, ui.z as f32 + 0.5);
                    broadcast_except(&self_data.rak, &self_data.players, None, sound.encode()).await;

                    // Wake up surrounding liquid
                    self_data.liquid_sim.schedule_update(ui.x - 1, ui.y as i32, ui.z);
                    self_data.liquid_sim.schedule_update(ui.x + 1, ui.y as i32, ui.z);
                    self_data.liquid_sim.schedule_update(ui.x, ui.y as i32, ui.z - 1);
                    self_data.liquid_sim.schedule_update(ui.x, ui.y as i32, ui.z + 1);

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
            let mut w = self_data.world.write();
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
                let mut w = self_data.world.write();
                w.set_block(ui.x, ui.y as i32, ui.z, 0, 0);
            }
            let clear_pk = UpdateBlockPacket::new(ui.x, ui.y, ui.z, 0, 0, update_block::FLAG_ALL);
            broadcast_except(&self_data.rak, &self_data.players, None, clear_pk.encode()).await;

            let tnt_eid = {
                let mut guard = self_data.next_entity_id.write();
                let id = *guard;
                *guard += 1;
                id
            };
            self_data.entity_manager.spawn_tnt(
                tnt_eid,
                ui.x as f64 + 0.5,
                ui.y as f64,
                ui.z as f64 + 0.5,
                80,
                &self_data.rak,
                &self_data.players,
            ).await;
            return;
        }

        // Flint and Steel (259) or Fire Charge (385) on Obsidian (49) -> Ignite Nether Portal
        if (item_id == 259 || item_id == 385) && target_id == 49 {
            let dim = self_data.players.read().get(&addr).map(|s| s.dimension).unwrap_or(0);
            let world_arc = self_data.get_world(dim);
            let portal_blocks = {
                let mut w = world_arc.write();
                try_ignite_nether_portal(&mut w, ui.x, ui.y as i32, ui.z)
            };
            if let Some(blocks) = portal_blocks {
                for &(bx, by, bz) in &blocks {
                    let pk = UpdateBlockPacket::new(bx, by as u32, bz, 90, 1, update_block::FLAG_ALL_PRIORITY);
                    broadcast_except(&self_data.rak, &self_data.players, None, pk.encode()).await;
                }
                let sound = LevelSoundEventPacket::new(level_event::SOUND_PORTAL, ui.x as f32 + 0.5, ui.y as f32 + 0.5, ui.z as f32 + 0.5);
                broadcast_except(&self_data.rak, &self_data.players, None, sound.encode()).await;
                return;
            }
        }

        // Hoe on Dirt (3) or Grass (2) -> Farmland (60)
        let is_hoe = item_id == 290 || item_id == 291 || item_id == 292 || item_id == 293 || item_id == 294;
        if is_hoe && (target_id == 2 || target_id == 3) && ui.face == 1 {
            {
                let mut w = self_data.world.write();
                w.set_block(ui.x, ui.y as i32, ui.z, 60, 0);
            }
            let pk = UpdateBlockPacket::new(ui.x, ui.y, ui.z, 60, 0, update_block::FLAG_ALL);
            broadcast_except(&self_data.rak, &self_data.players, None, pk.encode()).await;
            let till_snd = LevelSoundEventPacket::new(level_event::SOUND_ITEM_USE_ON, ui.x as f32 + 0.5, ui.y as f32 + 0.5, ui.z as f32 + 0.5);
            broadcast_except(&self_data.rak, &self_data.players, None, till_snd.encode()).await;
            return;
        }

        // Shovel on Grass (2) -> Grass Path (198)
        let is_shovel = item_id == 256 || item_id == 269 || item_id == 273 || item_id == 277 || item_id == 284;
        if is_shovel && target_id == 2 && ui.face == 1 {
            {
                let mut w = self_data.world.write();
                w.set_block(ui.x, ui.y as i32, ui.z, 198, 0);
            }
            let pk = UpdateBlockPacket::new(ui.x, ui.y, ui.z, 198, 0, update_block::FLAG_ALL);
            broadcast_except(&self_data.rak, &self_data.players, None, pk.encode()).await;
            let till_snd = LevelSoundEventPacket::new(level_event::SOUND_ITEM_USE_ON, ui.x as f32 + 0.5, ui.y as f32 + 0.5, ui.z as f32 + 0.5);
            broadcast_except(&self_data.rak, &self_data.players, None, till_snd.encode()).await;
            return;
        }

        // Bone Meal (351:15) on crops (59, 141, 142, 244) or sapling (6)
        if item_id == 351 && item_meta == 15 && (target_id == 59 || target_id == 141 || target_id == 142 || target_id == 244 || target_id == 6) {
            let mature_meta = if target_id == 244 { 4 } else { 7 };
            {
                let mut w = self_data.world.write();
                w.set_block(ui.x, ui.y as i32, ui.z, target_id, mature_meta);
            }
            let pk = UpdateBlockPacket::new(ui.x, ui.y, ui.z, target_id as u32, mature_meta as u32, update_block::FLAG_ALL);
            broadcast_except(&self_data.rak, &self_data.players, None, pk.encode()).await;
            let grow_snd = LevelEventPacket::new(level_event::EVENT_PARTICLE_SPLASH, ui.x as f32 + 0.5, ui.y as f32 + 0.5, ui.z as f32 + 0.5, 0);
            broadcast_except(&self_data.rak, &self_data.players, None, grow_snd.encode()).await;
            return;
        }

        // Doors (64, 71), Trapdoors (96), Gates (107) toggle
        if target_id == 64 || target_id == 71 || target_id == 96 || target_id == 107 {
            let new_meta = target_meta ^ 0x04;
            {
                let mut w = self_data.world.write();
                w.set_block(ui.x, ui.y as i32, ui.z, target_id, new_meta);
            }
            let pk = UpdateBlockPacket::new(ui.x, ui.y, ui.z, target_id as u32, new_meta as u32, update_block::FLAG_ALL);
            broadcast_except(&self_data.rak, &self_data.players, None, pk.encode()).await;

            let door_sound = LevelEventPacket::new(
                level_event::EVENT_SOUND_DOOR,
                ui.x as f32 + 0.5,
                ui.y as f32 + 0.5,
                ui.z as f32 + 0.5,
                0,
            );
            broadcast_except(&self_data.rak, &self_data.players, None, door_sound.encode()).await;
            return;
        }

        // Crafting Table (58)
        if target_id == 58 {
            let open_pk = ContainerOpenPacket::new(1, 1, ui.x, ui.y, ui.z);
            send_packet(&self_data.rak, addr, open_pk.encode()).await;
            return;
        }

        // Chest (54)
        if target_id == 54 {
            let (slots, first) = self_data.tile_manager.open_chest(addr, ui.x, ui.y as i32, ui.z);
            {
                let mut p = self_data.players.write();
                if let Some(s) = p.get_mut(&addr) {
                    s.open_container = Some((ui.x, ui.y as i32, ui.z));
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
                broadcast_except(&self_data.rak, &self_data.players, None, block_ev.encode()).await;
                broadcast_except(&self_data.rak, &self_data.players, None, sound.encode()).await;
            }
            return;
        }

        // Furnace (61, 62)
        if target_id == 61 || target_id == 62 {
            let slots = self_data.tile_manager.open_furnace(addr, ui.x, ui.y as i32, ui.z);
            {
                let mut p = self_data.players.write();
                if let Some(s) = p.get_mut(&addr) {
                    s.open_container = Some((ui.x, ui.y as i32, ui.z));
                }
            }
            let open_pk = ContainerOpenPacket::new(1, 2, ui.x, ui.y, ui.z);
            send_packet(&self_data.rak, addr, open_pk.encode()).await;

            let mut content_pk = ContainerSetContentPacket::new(1);
            content_pk.items = slots;
            send_packet(&self_data.rak, addr, content_pk.encode()).await;
            return;
        }

        // Anvil (145)
        if target_id == 145 {
            let open_pk = ContainerOpenPacket::new(1, 5, ui.x, ui.y, ui.z);
            send_packet(&self_data.rak, addr, open_pk.encode()).await;
            return;
        }

        // Enchanting Table (116)
        if target_id == 116 {
            let open_pk = ContainerOpenPacket::new(1, 3, ui.x, ui.y, ui.z);
            send_packet(&self_data.rak, addr, open_pk.encode()).await;
            return;
        }

        // Brewing Stand (117)
        if target_id == 117 {
            let open_pk = ContainerOpenPacket::new(1, 4, ui.x, ui.y, ui.z);
            send_packet(&self_data.rak, addr, open_pk.encode()).await;
            return;
        }

        // Bed (26)
        if target_id == 26 {
            self_data.send_message_to(addr, "Respawn point set");
            return;
        }
    }

    // 5. Normal block placement
    if ui.face >= 0 && ui.face <= 5 {
        handle_place_block(self_data, addr, ui.x, ui.y as i32, ui.z, ui.face, ui.item.id).await;
    }
}

async fn handle_place_block(self_data: &OxideServer, addr: SocketAddr, x: i32, y: i32, z: i32, face: i32, item_id: i32) {
    // Only valid placeable block IDs (1..255)
    if item_id <= 0 || item_id >= 256 { return; }
    let (nx, ny, nz) = match face {
        0 => (x, y - 1, z),
        1 => (x, y + 1, z),
        2 => (x, y, z - 1),
        3 => (x, y, z + 1),
        4 => (x - 1, y, z),
        5 => (x + 1, y, z),
        _ => (x, y + 1, z),
    };
    if ny < 0 || ny >= 256 { return; }

    // Reach and AABB collision check
    let player_info = self_data.players.read().get(&addr).map(|s| (s.x, s.y, s.z));
    if let Some((px, py, pz)) = player_info {
        let dx = px - (nx as f64 + 0.5);
        let dy = py - (ny as f64 + 0.5);
        let dz = pz - (nz as f64 + 0.5);
        if dx*dx + dy*dy + dz*dz > 64.0 {
            return;
        }

        // Check bounding box intersection with player
        let min_x = nx as f64;
        let max_x = min_x + 1.0;
        let min_y = ny as f64;
        let max_y = min_y + 1.0;
        let min_z = nz as f64;
        let max_z = min_z + 1.0;

        let p_min_x = px - 0.3;
        let p_max_x = px + 0.3;
        let p_min_y = py;
        let p_max_y = py + 1.8;
        let p_min_z = pz - 0.3;
        let p_max_z = pz + 0.3;

        let intersects = (p_min_x < max_x && p_max_x > min_x) &&
                         (p_min_y < max_y && p_max_y > min_y) &&
                         (p_min_z < max_z && p_max_z > min_z);
        if intersects {
            return;
        }
    }

    let (player_name, is_survival, held_slot) = self_data.players.read().get(&addr)
        .map(|s| (s.username.clone(), s.gamemode == 0, s.inventory.held_index()))
        .unwrap_or_default();

    let ev = self_data.event_bus.fire(crate::event::event::BlockPlaceEvent {
        player_name,
        x: nx,
        y: ny as u32,
        z: nz,
        block_id: item_id as u16,
        cancelled: false,
    });
    if ev.cancelled {
        let revert_pk = UpdateBlockPacket::new(nx, ny as u32, nz, 0, 0, update_block::FLAG_ALL);
        send_packet(&self_data.rak, addr, revert_pk.encode()).await;
        return;
    }

    {
        let mut world = self_data.world.write();
        let existing = world.get_block_id(nx, ny, nz);
        if existing != 0 { return; }
        world.set_block(nx, ny, nz, item_id as u8, 0);
    }
    let pk = UpdateBlockPacket::new(nx, ny as u32, nz, item_id as u32, 0, update_block::FLAG_ALL);
    broadcast_except(&self_data.rak, &self_data.players, None, pk.encode()).await;

    // Block place sound with block ID
    let place_sound = LevelSoundEventPacket::new_with_extra(
        level_event::SOUND_PLACE,
        nx as f32 + 0.5,
        ny as f32 + 0.5,
        nz as f32 + 0.5,
        item_id as i32,
    );
    broadcast_except(&self_data.rak, &self_data.players, None, place_sound.encode()).await;

    // Gravity for sand and gravel
    if item_id == 12 || item_id == 13 {
        let below_id = {
            let mut world = self_data.world.write();
            if ny > 0 { world.get_block_id(nx, ny - 1, nz) } else { 7 }
        };
        if below_id == 0 {
            let mut fall_y = ny - 1;
            {
                let mut world = self_data.world.write();
                while fall_y > 0 && world.get_block_id(nx, fall_y - 1, nz) == 0 {
                    fall_y -= 1;
                }
                world.set_block(nx, ny, nz, 0, 0);
                world.set_block(nx, fall_y, nz, item_id as u8, 0);
            }
            let clear_pk = UpdateBlockPacket::new(nx, ny as u32, nz, 0, 0, update_block::FLAG_ALL);
            broadcast_except(&self_data.rak, &self_data.players, None, clear_pk.encode()).await;

            let place_pk = UpdateBlockPacket::new(nx, fall_y as u32, nz, item_id as u32, 0, update_block::FLAG_ALL);
            broadcast_except(&self_data.rak, &self_data.players, None, place_pk.encode()).await;
        }
    }

    // Schedule liquid update if water or lava placed
    if item_id == 8 || item_id == 9 || item_id == 10 || item_id == 11 {
        self_data.liquid_sim.schedule_update(nx, ny, nz);
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

async fn spawn_existing_players_to(self_data: &OxideServer, addr: SocketAddr) {
    let others: Vec<(i64, String, UUID, f64, f64, f64, f32, f32)> = {
        let guard = self_data.players.read();
        guard.iter()
            .filter(|(a, _)| **a != addr)
            .filter(|(_, s)| s.logged_in)
            .map(|(_, s)| (s.entity_id, s.username.clone(), s.uuid, s.x, s.y, s.z, s.yaw, s.pitch))
            .collect()
    };
    for (eid, name, uuid, x, y, z, yaw, pitch) in others {
        let mut pk = AddPlayerPacket::new(uuid, name.clone(), eid, x as f32, (y + 1.62) as f32, z as f32);
        pk.yaw = yaw;
        pk.pitch = pitch;
        send_packet(&self_data.rak, addr, pk.encode()).await;
    }
}

async fn announce_new_player_to_others(self_data: &OxideServer, addr: SocketAddr, entity_id: i64, username: String, uuid: UUID) {
    let (x, y, z, yaw, pitch) = self_data.players.read().get(&addr)
        .map(|s| (s.x, s.y, s.z, s.yaw, s.pitch))
        .unwrap_or((0.0, 64.0, 0.0, 0.0, 0.0));
    let mut pk = AddPlayerPacket::new(uuid, username, entity_id, x as f32, (y + 1.62) as f32, z as f32);
    pk.yaw = yaw;
    pk.pitch = pitch;
    broadcast_except(&self_data.rak, &self_data.players, Some(addr), pk.encode()).await;
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
                skin_id: "Standard".to_string(),
                skin: Vec::new(),
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

    let chunks: Vec<Vec<u8>> = {
        let dim = self_data.players.read().get(&addr).map(|s| s.dimension).unwrap_or(0);
        let target_world = self_data.get_world(dim);
        let mut world = target_world.write();
        wanted.iter()
            .map(|&(x, z)| FullChunkDataPacket::new(x, z, world.serialize_chunk(x, z)).encode())
            .collect()
    };

    let total = chunks.len();
    for (i, pk) in chunks.into_iter().enumerate() {
        send_raw(&self_data.rak, addr, BatchPacket::encode(&[pk])).await;
        if (i + 1) % 16 == 0 {
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
        }
    }
}
