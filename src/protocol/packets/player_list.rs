use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;
use crate::utils::UUID;
use crate::protocol::types::Item;

pub const TYPE_ADD: u8 = 0;
pub const TYPE_REMOVE: u8 = 1;

#[derive(Debug, Clone)]
pub struct PlayerListEntry {
    pub uuid: UUID,
    pub entity_id: i64,
    pub name: String,
    pub skin_id: String,
    pub skin: Vec<u8>,
    pub xbox_user_id: String,
    pub platform_chat_id: String,
    pub build_platform: i32,
}

pub struct PlayerListPacket {
    pub list_type: u8,
    pub entries: Vec<PlayerListEntry>,
}

impl PlayerListPacket {
    pub fn add(entries: Vec<PlayerListEntry>) -> Self { Self { list_type: TYPE_ADD, entries } }
    pub fn remove(uuids: Vec<UUID>) -> Self {
        let entries = uuids.into_iter().map(|uuid| PlayerListEntry {
            uuid, entity_id: 0, name: String::new(), skin_id: String::new(),
            skin: Vec::new(), xbox_user_id: String::new(),
            platform_chat_id: String::new(), build_platform: -1,
        }).collect();
        Self { list_type: TYPE_REMOVE, entries }
    }
}

impl DataPacket for PlayerListPacket {
    const NETWORK_ID: u8 = ProtocolInfo::PLAYER_LIST_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_byte(self.list_type);
        stream.put_unsigned_var_int(self.entries.len() as u32);
        for entry in &self.entries {
            stream.put_uuid(entry.uuid);
            if self.list_type == TYPE_ADD {
                stream.put_entity_id(entry.entity_id);
                stream.put_string(&entry.name);
                stream.put_string(&entry.skin_id);
                stream.put_string(std::str::from_utf8(&entry.skin).unwrap_or(""));
                stream.put_string(&entry.xbox_user_id);
                stream.put_string(&entry.platform_chat_id);
                stream.put_var_int(entry.build_platform);
            }
        }
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let list_type = stream.get_byte().unwrap_or(0);
        let count = stream.get_unsigned_var_int().unwrap_or(0);
        let mut entries = Vec::new();
        for _ in 0..count {
            let uuid = stream.get_uuid().unwrap_or(UUID::nil());
            if list_type == TYPE_ADD {
                let entity_id = stream.get_entity_id().unwrap_or(0);
                let name = stream.get_string().unwrap_or_default();
                let skin_id = stream.get_string().unwrap_or_default();
                let skin = stream.get_string().unwrap_or_default().into_bytes();
                let xbox_user_id = stream.get_string().unwrap_or_default();
                let platform_chat_id = stream.get_string().unwrap_or_default();
                let build_platform = stream.get_var_int().unwrap_or(-1);
                entries.push(PlayerListEntry {
                    uuid, entity_id, name, skin_id, skin, xbox_user_id, platform_chat_id, build_platform,
                });
            } else {
                entries.push(PlayerListEntry {
                    uuid, entity_id: 0, name: String::new(), skin_id: String::new(),
                    skin: Vec::new(), xbox_user_id: String::new(),
                    platform_chat_id: String::new(), build_platform: -1,
                });
            }
        }
        let _ = Item::air();
        Self { list_type, entries }
    }

    fn name(&self) -> &'static str { "PlayerListPacket" }
}
