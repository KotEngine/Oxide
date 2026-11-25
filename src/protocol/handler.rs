use crate::utils::UUID;
use crate::protocol::info::ProtocolInfo;
use crate::protocol::packet::DataPacket;
use crate::protocol::packets::*;
use crate::protocol::types::EntityMetadata;

#[derive(Debug)]
pub enum IncomingPacket {
    Login(LoginPacket),
    ResourcePackClientResponse(ResourcePackClientResponsePacket),
    RequestChunkRadius(RequestChunkRadiusPacket),
    MovePlayer(MovePlayerPacket),
    PlayerAction(PlayerActionPacket),
    RemoveBlock(RemoveBlockPacket),
    Text(TextPacket),
    UseItem(UseItemPacket),
    Interact(InteractPacket),
    MobEquipment(MobEquipmentPacket),
    ContainerSetSlot(ContainerSetSlotPacket),
    ContainerClose(ContainerClosePacket),
    CraftingEvent(CraftingEventPacket),
    Animate(AnimatePacket),
    DropItem(DropItemPacket),
    PlayerInput(PlayerInputPacket),
    CommandStep(CommandStepPacket),
    BlockPickRequest(BlockPickRequestPacket),
    BlockEntityData(BlockEntityDataPacket),
    AdventureSettings(AdventureSettingsPacket),
    EntityEvent(EntityEventPacket),
    EntityFall(EntityFallPacket),
    SetPlayerGameType(SetPlayerGameTypePacket),
    ItemFrameDropItem(ItemFrameDropItemPacket),
    Unknown(u8, Vec<u8>),
}

pub struct PacketHandler;

impl PacketHandler {
    pub fn handle(payload: Vec<u8>) -> IncomingPacket {
        if payload.is_empty() {
            return IncomingPacket::Unknown(0, payload);
        }
        let id = payload[0];
        match id {
            ProtocolInfo::LOGIN_PACKET => IncomingPacket::Login(LoginPacket::decode(payload)),
            ProtocolInfo::RESOURCE_PACK_CLIENT_RESPONSE_PACKET => IncomingPacket::ResourcePackClientResponse(ResourcePackClientResponsePacket::decode(payload)),
            ProtocolInfo::REQUEST_CHUNK_RADIUS_PACKET => IncomingPacket::RequestChunkRadius(RequestChunkRadiusPacket::decode(payload)),
            ProtocolInfo::MOVE_PLAYER_PACKET => IncomingPacket::MovePlayer(MovePlayerPacket::decode(payload)),
            ProtocolInfo::PLAYER_ACTION_PACKET => IncomingPacket::PlayerAction(PlayerActionPacket::decode(payload)),
            ProtocolInfo::REMOVE_BLOCK_PACKET => IncomingPacket::RemoveBlock(RemoveBlockPacket::decode(payload)),
            ProtocolInfo::TEXT_PACKET => IncomingPacket::Text(TextPacket::decode(payload)),
            ProtocolInfo::USE_ITEM_PACKET => IncomingPacket::UseItem(UseItemPacket::decode(payload)),
            ProtocolInfo::INTERACT_PACKET => IncomingPacket::Interact(InteractPacket::decode(payload)),
            ProtocolInfo::MOB_EQUIPMENT_PACKET => IncomingPacket::MobEquipment(MobEquipmentPacket::decode(payload)),
            ProtocolInfo::CONTAINER_SET_SLOT_PACKET => IncomingPacket::ContainerSetSlot(ContainerSetSlotPacket::decode(payload)),
            ProtocolInfo::CONTAINER_CLOSE_PACKET => IncomingPacket::ContainerClose(ContainerClosePacket::decode(payload)),
            ProtocolInfo::CRAFTING_EVENT_PACKET => IncomingPacket::CraftingEvent(CraftingEventPacket::decode(payload)),
            ProtocolInfo::ANIMATE_PACKET => IncomingPacket::Animate(AnimatePacket::decode(payload)),
            ProtocolInfo::DROP_ITEM_PACKET => IncomingPacket::DropItem(DropItemPacket::decode(payload)),
            ProtocolInfo::PLAYER_INPUT_PACKET => IncomingPacket::PlayerInput(PlayerInputPacket::decode(payload)),
            ProtocolInfo::COMMAND_STEP_PACKET => IncomingPacket::CommandStep(CommandStepPacket::decode(payload)),
            ProtocolInfo::BLOCK_PICK_REQUEST_PACKET => IncomingPacket::BlockPickRequest(BlockPickRequestPacket::decode(payload)),
            ProtocolInfo::BLOCK_ENTITY_DATA_PACKET => IncomingPacket::BlockEntityData(BlockEntityDataPacket::decode(payload)),
            ProtocolInfo::ADVENTURE_SETTINGS_PACKET => IncomingPacket::AdventureSettings(AdventureSettingsPacket::decode(payload)),
            ProtocolInfo::ENTITY_EVENT_PACKET => IncomingPacket::EntityEvent(EntityEventPacket::decode(payload)),
            ProtocolInfo::ENTITY_FALL_PACKET => IncomingPacket::EntityFall(EntityFallPacket::decode(payload)),
            ProtocolInfo::SET_PLAYER_GAME_TYPE_PACKET => IncomingPacket::SetPlayerGameType(SetPlayerGameTypePacket::decode(payload)),
            ProtocolInfo::ITEM_FRAME_DROP_ITEM_PACKET => IncomingPacket::ItemFrameDropItem(ItemFrameDropItemPacket::decode(payload)),
            _ => IncomingPacket::Unknown(id, payload),
        }
    }
}

pub struct PlayerState {
    pub username: String,
    pub uuid: UUID,
    pub entity_id: i64,
    pub skin_id: String,
    pub skin: Vec<u8>,
    pub client_id: i64,
    pub device_os: i64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
    pub fall_distance: f32,
    pub gamemode: i32,
    pub allow_flight: bool,
    pub is_flying: bool,
    pub sprinting: bool,
    pub sneaking: bool,
    pub gliding: bool,
    pub health: u32,
    pub food_level: u32,
    pub food_saturation: f32,
    pub exhaustion: f32,
    pub air_ticks: i16,
    pub fire_ticks: i32,
    pub no_damage_ticks: u32,
    pub xp_points: u32,
    pub xp_level: u32,
    pub xp_progress: f32,
    pub metadata: EntityMetadata,
    pub chunk_radius: i32,
    pub logged_in: bool,
    pub spawned: bool,
    /// Chunks already sent to the player to avoid duplicate transmissions.
    pub sent_chunks: std::collections::HashSet<(i32, i32)>,
    pub inventory: crate::inventory::inventories::PlayerInventory,
    pub ender_chest: Vec<crate::protocol::types::Item>,
    pub open_container: Option<(i32, i32, i32)>,
    pub open_window_type: Option<i8>,
    pub dimension: i32,
    pub portal_ticks: u32,
    pub spawn_pos: Option<(i32, i32, i32, i32)>,
    pub effects: crate::entity::effect::EffectContainer,
    pub food_tick_timer: u32,
    pub start_action_tick: Option<u64>,
    pub break_start: Option<(i32, i32, i32, u64)>,
    pub item_cooldowns: std::collections::HashMap<i32, u64>,
    pub fishing_hook_eid: Option<i64>,
    pub pending_drops: Vec<crate::protocol::types::Item>,
    pub pending_xp_drop: u32,
}

impl Default for PlayerState {
    fn default() -> Self { Self::new() }
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            uuid: UUID::nil(),
            entity_id: 0,
            skin_id: "Standard_Custom".to_string(),
            skin: vec![0u8; 64 * 32 * 4],
            client_id: 0,
            device_os: 0,
            x: 0.0, y: 64.0, z: 0.0,
            yaw: 0.0, pitch: 0.0,
            on_ground: false,
            fall_distance: 0.0,
            gamemode: 0,
            allow_flight: false,
            is_flying: false,
            sprinting: false,
            sneaking: false,
            gliding: false,
            health: 20,
            food_level: 20,
            food_saturation: 5.0,
            exhaustion: 0.0,
            air_ticks: 400,
            fire_ticks: 0,
            no_damage_ticks: 0,
            xp_points: 0,
            xp_level: 0,
            xp_progress: 0.0,
            metadata: EntityMetadata::new(),
            chunk_radius: 8,
            logged_in: false,
            spawned: false,
            sent_chunks: std::collections::HashSet::new(),
            inventory: crate::inventory::inventories::PlayerInventory::new(),
            ender_chest: vec![crate::protocol::types::Item::air(); 27],
            open_container: None,
            open_window_type: None,
            dimension: 0,
            portal_ticks: 0,
            spawn_pos: None,
            effects: crate::entity::effect::EffectContainer::new(),
            food_tick_timer: 0,
            start_action_tick: None,
            break_start: None,
            item_cooldowns: std::collections::HashMap::new(),
            fishing_hook_eid: None,
            pending_drops: Vec::new(),
            pending_xp_drop: 0,
        }
    }
}
