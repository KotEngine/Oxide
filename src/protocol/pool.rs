use crate::protocol::info::ProtocolInfo;

pub struct PacketPool;

impl PacketPool {
    pub fn is_handled(id: u8) -> bool {
        matches!(
            id,
            ProtocolInfo::LOGIN_PACKET
                | ProtocolInfo::RESOURCE_PACK_CLIENT_RESPONSE_PACKET
                | ProtocolInfo::REQUEST_CHUNK_RADIUS_PACKET
                | ProtocolInfo::MOVE_PLAYER_PACKET
                | ProtocolInfo::PLAYER_ACTION_PACKET
                | ProtocolInfo::REMOVE_BLOCK_PACKET
                | ProtocolInfo::TEXT_PACKET
        )
    }

    pub fn name(id: u8) -> &'static str {
        match id {
            ProtocolInfo::LOGIN_PACKET => "LoginPacket",
            ProtocolInfo::PLAY_STATUS_PACKET => "PlayStatusPacket",
            ProtocolInfo::TEXT_PACKET => "TextPacket",
            ProtocolInfo::SET_TIME_PACKET => "SetTimePacket",
            ProtocolInfo::START_GAME_PACKET => "StartGamePacket",
            ProtocolInfo::MOVE_PLAYER_PACKET => "MovePlayerPacket",
            ProtocolInfo::REMOVE_BLOCK_PACKET => "RemoveBlockPacket",
            ProtocolInfo::UPDATE_BLOCK_PACKET => "UpdateBlockPacket",
            ProtocolInfo::PLAYER_ACTION_PACKET => "PlayerActionPacket",
            ProtocolInfo::FULL_CHUNK_DATA_PACKET => "FullChunkDataPacket",
            ProtocolInfo::REQUEST_CHUNK_RADIUS_PACKET => "RequestChunkRadiusPacket",
            ProtocolInfo::CHUNK_RADIUS_UPDATED_PACKET => "ChunkRadiusUpdatedPacket",
            ProtocolInfo::BATCH_PACKET => "BatchPacket",
            _ => "UnknownPacket",
        }
    }
}
