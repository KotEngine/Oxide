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

        let block_registry = Arc::new(crate::block::BlockRegistry::default());
        let item_registry = Arc::new(crate::item::ItemRegistry::default());
        let crafting_manager = Arc::new(crate::inventory::CraftingManager::default());
        let event_bus = Arc::new(crate::event::EventBus::new());
        let scheduler = Arc::new(crate::scheduler::Scheduler::new());
        let plugin_manager = Arc::new(crate::plugin::PluginManager::new());
        let permission_manager = Arc::new(crate::permission::PermissionManager::new());

        Ok(Self {
            rak,
            world: Arc::new(RwLock::new(world)),
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
            next_entity_id: Arc::new(RwLock::new(1)),
        })
    }

    pub async fn start(self) -> anyhow::Result<()> {
        let rak = self.rak.clone();
        rak.clone().start().await?;

        let tick_handles = TickHandles {
            rak: self.rak.clone(),
            world: self.world.clone(),
            players: self.players.clone(),
            scheduler: self.scheduler.clone(),
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
        let guard = self.players.read();
        let lower = name.to_lowercase();
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
}

struct TickHandles {
    rak: Arc<RakNetServer>,
    world: Arc<RwLock<World>>,
    players: Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    scheduler: Arc<crate::scheduler::Scheduler>,
}

async fn tick_loop(h: TickHandles) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50));
    let mut tick_counter: u64 = 0;
    loop {
        interval.tick().await;
        tick_counter += 1;

        h.scheduler.tick(tick_counter);

        {
            let mut world = h.world.write();
            world.tick();
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
        gamemode: s.gamemode,
        health: s.health,
        metadata: s.metadata.clone(),
        chunk_radius: s.chunk_radius,
        logged_in: s.logged_in,
        spawned: s.spawned,
        sent_chunks: s.sent_chunks.clone(),
        inventory: inv,
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

                    send_spawn_chunks(self_data, addr, spawn_x, spawn_z, chunk_radius).await;

                    if let Some(state) = self_data.players.write().get_mut(&addr) {
                        state.x = spawn_x as f64 + 0.5;
                        state.y = spawn_y as f64;
                        state.z = spawn_z as f64 + 0.5;
                        state.logged_in = true;
                        state.spawned = true;
                    }

                    let spawn_status = PlayStatusPacket::new(play_status::PLAYER_SPAWN);
                    send_packet(&self_data.rak, addr, spawn_status.encode()).await;

                    spawn_existing_players_to(self_data, addr).await;
                    announce_new_player_to_others(self_data, addr, entity_id, username.clone(), uuid).await;
                    broadcast_player_list_add(self_data).await;

                    let join_msg = TextPacket::system(format!("{} joined the game", username));
                    broadcast_except(&self_data.rak, &self_data.players, None, join_msg.encode()).await;

                    self_data.event_bus.fire(crate::event::event::PlayerJoinEvent {
                        player_name: username.clone(),
                    });
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
            if action.action == player_action::ACTION_STOP_BREAK {
                handle_break_block(self_data, addr, action.x, action.y as i32, action.z).await;
            }
        }
        IncomingPacket::UseItem(ui) => {
            handle_place_block(self_data, addr, ui.x, ui.y as i32, ui.z, ui.face, ui.item.id).await;
        }
        IncomingPacket::Interact(_) => {}
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
            }
        }
        IncomingPacket::ContainerClose(_) => {}
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
                let pos = self_data.players.read().get(&addr).map(|s| (s.x, s.y, s.z));
                if let Some((px, py, pz)) = pos {
                    let add_item = AddItemEntityPacket::new(
                        eid,
                        drop_pk.item.clone(),
                        px as f32,
                        (py + 1.2) as f32,
                        pz as f32,
                    );
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
            let cmd_line = cs.command.trim();
            let name = self_data.players.read().get(&addr).map(|s| s.username.clone()).unwrap_or_default();
            let mut sender = crate::command::sender::PlayerSender::new(name, addr, true, self_data.clone());
            let handled = self_data.command_manager.dispatch(&mut sender, self_data, cmd_line);
            if !handled {
                self_data.send_message_to(addr, &format!("Unknown command: /{}. Type /help for help.", cmd_line));
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

    let (player_name, old_id) = {
        let mut world = self_data.world.write();
        let id = world.get_block_id(x, y, z);
        if id == 0 || id == 7 { // cannot break air or bedrock
            return;
        }
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
            let revert_pk = UpdateBlockPacket::new(x, y as u32, z, id as u32, 0, update_block::FLAG_ALL);
            let rak = self_data.rak.clone();
            tokio::spawn(async move {
                send_packet(&rak, addr, revert_pk.encode()).await;
            });
            return;
        }
        world.set_block(x, y, z, 0, 0);
        (name, id)
    };
    let _ = player_name;

    let pk = UpdateBlockPacket::new(x, y as u32, z, 0, 0, update_block::FLAG_ALL);
    broadcast_except(&self_data.rak, &self_data.players, None, pk.encode()).await;

    // Spawn block drop item entity
    let entity_id = {
        let mut eid = self_data.next_entity_id.write();
        let id = *eid;
        *eid += 1;
        id
    };
    let drop_item = crate::protocol::types::Item::new(old_id as i32, 0, 1);
    let drop_pk = AddItemEntityPacket::new(
        entity_id,
        drop_item,
        x as f32 + 0.5,
        y as f32 + 0.25,
        z as f32 + 0.5,
    );
    broadcast_except(&self_data.rak, &self_data.players, None, drop_pk.encode()).await;
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
        let mut pk = AddPlayerPacket::new(uuid, name.clone(), eid, x as f32, y as f32, z as f32);
        pk.yaw = yaw;
        pk.pitch = pitch;
        send_packet(&self_data.rak, addr, pk.encode()).await;
    }
}

async fn announce_new_player_to_others(self_data: &OxideServer, addr: SocketAddr, entity_id: i64, username: String, uuid: UUID) {
    let (x, y, z, yaw, pitch) = self_data.players.read().get(&addr)
        .map(|s| (s.x, s.y, s.z, s.yaw, s.pitch))
        .unwrap_or((0.0, 64.0, 0.0, 0.0, 0.0));
    let pk = AddPlayerPacket::new(uuid, username, entity_id, x as f32, y as f32, z as f32);
    let _ = (yaw, pitch);
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

/// Number of chunks sent per burst before yielding.
/// Analogous to chunks-per-tick rate limiter.
const CHUNKS_PER_BURST: usize = 4;

async fn send_spawn_chunks(self_data: &OxideServer, addr: SocketAddr, cx: i32, cz: i32, radius: i32) {
    // Select only chunks not previously sent to this player
    let wanted: Vec<(i32, i32)> = {
        let mut players = self_data.players.write();
        let Some(state) = players.get_mut(&addr) else { return };
        let mut list = Vec::new();
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let coord = ((cx >> 4) + dx, (cz >> 4) + dz);
                if state.sent_chunks.insert(coord) {
                    list.push(coord);
                }
            }
        }
        list
    };
    if wanted.is_empty() { return; }

    let chunks: Vec<Vec<u8>> = {
        let mut world = self_data.world.write();
        wanted.iter()
            .map(|&(x, z)| FullChunkDataPacket::new(x, z, world.serialize_chunk(x, z)).encode())
            .collect()
    };

    let total = chunks.len();
    // Send chunks in separate batches to prevent client buffer overflows
    // and reduce packet loss on mobile connections
    for (i, pk) in chunks.into_iter().enumerate() {
        send_raw(&self_data.rak, addr, BatchPacket::encode(&[pk])).await;
        if (i + 1) % CHUNKS_PER_BURST == 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
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
            next_entity_id: self.next_entity_id.clone(),
        }
    }
}
