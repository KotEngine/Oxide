use crate::utils::UUID;
use crate::protocol::info::ProtocolInfo;
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
        }
    }
}
