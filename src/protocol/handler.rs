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
            _ => IncomingPacket::Unknown(id, payload),
        }
    }
}

pub struct PlayerState {
    pub username: String,
    pub uuid: UUID,
    pub entity_id: i64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
    pub gamemode: i32,
    pub health: u32,
    pub metadata: EntityMetadata,
    pub chunk_radius: i32,
    pub logged_in: bool,
    pub spawned: bool,
    /// Chunks already sent to the player to avoid duplicate transmissions.
    pub sent_chunks: std::collections::HashSet<(i32, i32)>,
    pub inventory: crate::inventory::inventories::PlayerInventory,
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
            x: 0.0, y: 64.0, z: 0.0,
            yaw: 0.0, pitch: 0.0,
            on_ground: false,
            gamemode: 0,
            health: 20,
            metadata: EntityMetadata::new(),
            chunk_radius: 8,
            logged_in: false,
            spawned: false,
            sent_chunks: std::collections::HashSet::new(),
            inventory: crate::inventory::inventories::PlayerInventory::new(),
        }
    }
}
