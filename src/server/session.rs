use std::net::SocketAddr;
use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::RwLock;
use tokio::sync::Mutex;
use tracing::{info, warn, error};

use crate::level::World;
use crate::protocol::info::ProtocolInfo;
use crate::protocol::packet::DataPacket;
use crate::protocol::batch::BatchPacket;
use crate::protocol::handler::{PacketHandler, IncomingPacket, PlayerState};
use crate::protocol::packets::*;
use crate::protocol::types::{EntityMetadata, GameRules};
use crate::raknet::{RakNetServer, RakNetEvent};
use crate::raknet::protocol::PacketReliability;
use crate::utils::{Config, UUID};

pub struct OxideServer {
    rak: Arc<RakNetServer>,
    world: Arc<RwLock<World>>,
    players: Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    config: Arc<RwLock<Config>>,
    next_entity_id: Arc<RwLock<i64>>,
}

impl OxideServer {
    pub async fn new(addr: SocketAddr) -> anyhow::Result<Self> {
        let config = load_or_create_config();
        let server_name = format!(
            "MCPE;;{};{};{};{};{};{};",
            config.get_string("motd"),
            ProtocolInfo::MINECRAFT_VERSION_NETWORK,
            ProtocolInfo::CURRENT_PROTOCOL,
            config.get_int("max_players"),
            config.get_int("online_count"),
            config.get_string("server_id"),
        );
        let rak = Arc::new(RakNetServer::new(addr, server_name).await?);
        info!("Oxide server starting on {}", addr);
        Ok(Self {
            rak,
            world: Arc::new(RwLock::new(World::new())),
            players: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(config)),
            next_entity_id: Arc::new(RwLock::new(1)),
        })
    }

    pub async fn start(self) -> anyhow::Result<()> {
        let rak = self.rak.clone();
        rak.clone().start().await?;

        let tick_self = self_for_tick(&self);
        tokio::spawn(async move {
            tick_loop(tick_self).await;
        });

        let event_self = self.clone();
        event_loop(event_self).await
    }
}

fn self_for_tick(s: &OxideServer) -> TickHandles {
    TickHandles {
        rak: s.rak.clone(),
        world: s.world.clone(),
        players: s.players.clone(),
    }
}

struct TickHandles {
    rak: Arc<RakNetServer>,
    world: Arc<RwLock<World>>,
    players: Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
}

async fn tick_loop(h: TickHandles) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50));
    let mut tick_counter: u64 = 0;
    loop {
        interval.tick().await;
        tick_counter += 1;

        {
            let mut world = h.world.write();
            world.tick();
        }

        if tick_counter % 40 == 0 {
            let time = { h.world.read().time };
            let pk = SetTimePacket::new(time);
            broadcast(&h.rak, &h.players, pk.encode()).await;
        }
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
                    self_data.players.write().remove(&addr.to_socket_addr());
                }
                RakNetEvent::Encapsulated(addr, payload) => {
                    if payload.is_empty() { continue; }
                    if payload[0] == ProtocolInfo::BATCH_PACKET {
                        match BatchPacket::decode(&payload) {
                            Ok(inner) => {
                                for pkt in inner {
                                    handle_packet(&self_data, addr.to_socket_addr(), pkt).await;
                                }
                            }
                            Err(e) => warn!("batch decode failed from {}: {}", addr.get_ip(), e),
                        }
                    } else {
                        handle_packet(&self_data, addr.to_socket_addr(), payload).await;
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
            if login.protocol as u32 != ProtocolInfo::CURRENT_PROTOCOL {
                let status = if ProtocolInfo::ACCEPTED_PROTOCOLS.contains(&(login.protocol as u32)) {
                    play_status::LOGIN_FAILED_SERVER
                } else {
                    play_status::LOGIN_FAILED_CLIENT
                };
                let pk = PlayStatusPacket::new(status);
                send_packet(&self_data.rak, addr, pk.encode()).await;
                let _ = self_data.rak.close_session(addr, "incompatible protocol").await;
                return;
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
            state.username = username;
            state.uuid = uuid;
            state.entity_id = entity_id;
            state.gamemode = 0;
            state.metadata = default_player_metadata();
            self_data.players.write().insert(addr, state);

            let success = PlayStatusPacket::new(play_status::LOGIN_SUCCESS);
            send_packet(&self_data.rak, addr, success.encode()).await;

            let rp_info = ResourcePacksInfoPacket::default();
            send_packet(&self_data.rak, addr, rp_info.encode()).await;
        }
        IncomingPacket::ResourcePackClientResponse(resp) => {
            if resp.status == resource_pack_client_response::STATUS_COMPLETED {
                let state_opt = self_data.players.read().get(&addr).map(|s| (s.entity_id, s.username.clone(), s.gamemode, s.metadata.clone(), s.chunk_radius));
                if let Some((entity_id, _username, gamemode, metadata, chunk_radius)) = state_opt {
                    let (spawn_x, spawn_y, spawn_z) = {
                        let w = self_data.world.read();
                        (w.spawn_x, w.spawn_y, w.spawn_z)
                    };
                    let start = StartGamePacket {
                        entity_unique_id: entity_id,
                        entity_runtime_id: entity_id,
                        player_gamemode: gamemode,
                        x: spawn_x as f32 + 0.5,
                        y: spawn_y as f32,
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
                        is_texture_packs_required: true,
                        game_rules: GameRules::default_rules(),
                        level_id: String::new(),
                        world_name: "Oxide".to_string(),
                        premium_world_template_id: String::new(),
                        unknown_bool: false,
                        current_tick: 0,
                    };
                    send_packet(&self_data.rak, addr, start.encode()).await;

                    let radius_pk = ChunkRadiusUpdatedPacket::new(chunk_radius);
                    send_packet(&self_data.rak, addr, radius_pk.encode()).await;

                    let _ = metadata;

                    send_spawn_chunks(self_data, addr, spawn_x, spawn_z, chunk_radius).await;

                    if let Some(state) = self_data.players.write().get_mut(&addr) {
                        state.logged_in = true;
                    }

                    let spawn = PlayStatusPacket::new(play_status::PLAYER_SPAWN);
                    send_packet(&self_data.rak, addr, spawn.encode()).await;
                }
            }
        }
        IncomingPacket::RequestChunkRadius(req) => {
            let radius = req.radius.clamp(2, 32);
            if let Some(state) = self_data.players.write().get_mut(&addr) {
                state.chunk_radius = radius;
            }
            let pk = ChunkRadiusUpdatedPacket::new(radius);
            send_packet(&self_data.rak, addr, pk.encode()).await;
            let (sx, sz) = {
                let w = self_data.world.read();
                (w.spawn_x, w.spawn_z)
            };
            send_spawn_chunks(self_data, addr, sx, sz, radius).await;
        }
        IncomingPacket::MovePlayer(mv) => {
            if let Some(state) = self_data.players.write().get_mut(&addr) {
                state.x = mv.x as f64;
                state.y = mv.y as f64;
                state.z = mv.z as f64;
                state.yaw = mv.yaw;
                state.pitch = mv.pitch;
                state.on_ground = mv.on_ground;
            }
        }
        IncomingPacket::Text(txt) => {
            if txt.text_type == text::TYPE_CHAT {
                let name = self_data.players.read().get(&addr).map(|s| s.username.clone()).unwrap_or_default();
                let msg = format!("<{}> {}", name, txt.message);
                info!("chat: {}", msg);
                let pk = TextPacket::system(msg);
                broadcast(&self_data.rak, &self_data.players, pk.encode()).await;
            }
        }
        IncomingPacket::PlayerAction(_) | IncomingPacket::RemoveBlock(_) => {}
        IncomingPacket::Unknown(_, _) => {
            let name = crate::protocol::pool::PacketPool::name(id);
            let ip = self_data.players.read().get(&addr).map(|s| s.username.clone()).unwrap_or_default();
            warn!("unhandled packet {} ({:#x}) from {}", name, id, ip);
        }
    }
}

async fn send_spawn_chunks(self_data: &OxideServer, addr: SocketAddr, cx: i32, cz: i32, radius: i32) {
    let mut chunks_data = Vec::new();
    {
        let mut world = self_data.world.write();
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let x = (cx >> 4) + dx;
                let z = (cz >> 4) + dz;
                let data = world.serialize_chunk(x, z);
                let pk = FullChunkDataPacket::new(x, z, data);
                chunks_data.push(pk.encode());
            }
        }
    }
    if chunks_data.is_empty() { return; }
    let batch = BatchPacket::encode(&chunks_data);
    send_raw(&self_data.rak, addr, batch).await;
}

async fn send_packet(rak: &RakNetServer, addr: SocketAddr, data: Vec<u8>) {
    let batch = BatchPacket::encode(&[data]);
    send_raw(rak, addr, batch).await;
}

async fn send_raw(rak: &RakNetServer, addr: SocketAddr, data: Vec<u8>) {
    rak.send_encapsulated(addr, data, true).await;
}

async fn broadcast(rak: &RakNetServer, players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>, data: Vec<u8>) {
    let addrs: Vec<SocketAddr> = players.read().keys().copied().collect();
    let batch = BatchPacket::encode(&[data]);
    for addr in addrs {
        rak.send_encapsulated(addr, batch.clone(), true).await;
    }
}

fn default_player_metadata() -> EntityMetadata {
    let mut m = EntityMetadata::new();
    m.set_int(0, 0);
    m.set_int(1, 300);
    m.set_int(2, 0);
    m.set_byte(3, 0);
    m.set_string(4, "".to_string());
    m.set_byte(5, 0);
    m.set_byte(6, 0);
    m.set_byte(7, 0);
    m.set_float(8, 0.0);
    m.set_int(9, 0);
    m.set_int(10, 0);
    m.set_int(11, 100);
    m.set_int(12, 0);
    m.set_long(13, -1);
    m.set_byte(14, 0);
    m.set_vector3f(15, 0.0, 0.0, 0.0);
    m.set_vector3f(16, 0.0, 0.0, 0.0);
    m.set_vector3f(17, 0.0, 0.0, 0.0);
    m.set_float(18, 0.0);
    m.set_float(19, 0.0);
    m.set_float(20, 0.0);
    m.set_int(21, 0);
    m.set_int(22, 0);
    m.set_int(23, 0);
    m.set_byte(24, 0);
    m.set_long(25, 0);
    m.set_byte(26, 0);
    m.set_byte(27, 0);
    m.set_int(28, 0);
    m.set_float(29, 0.6);
    m.set_float(30, 1.8);
    m.set_float(31, 0.0);
    m.set_float(32, 0.0);
    m.set_float(33, 0.0);
    m.set_float(34, 0.0);
    m.set_float(35, 0.0);
    m.set_byte(36, 0);
    m.set_byte(37, 0);
    m.set_byte(38, 0);
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
        let _ = c.save(&path);
        c
    }
}

impl Clone for OxideServer {
    fn clone(&self) -> Self {
        Self {
            rak: self.rak.clone(),
            world: self.world.clone(),
            players: self.players.clone(),
            config: self.config.clone(),
            next_entity_id: self.next_entity_id.clone(),
        }
    }
}
