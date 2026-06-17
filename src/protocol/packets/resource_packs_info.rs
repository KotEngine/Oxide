use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

#[derive(Debug, Clone)]
pub struct ResourcePackInfoEntry {
    pub pack_id: String,
    pub pack_version: String,
    pub pack_size: i64,
}

pub struct ResourcePacksInfoPacket {
    pub must_accept: bool,
    pub behavior_pack_entries: Vec<ResourcePackInfoEntry>,
    pub resource_pack_entries: Vec<ResourcePackInfoEntry>,
}

impl Default for ResourcePacksInfoPacket {
    fn default() -> Self {
        Self { must_accept: false, behavior_pack_entries: Vec::new(), resource_pack_entries: Vec::new() }
    }
}

impl DataPacket for ResourcePacksInfoPacket {
    const NETWORK_ID: u8 = ProtocolInfo::RESOURCE_PACKS_INFO_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_bool(self.must_accept);
        stream.put_lshort(self.behavior_pack_entries.len() as u16);
        for entry in &self.behavior_pack_entries {
            stream.put_string(&entry.pack_id);
            stream.put_string(&entry.pack_version);
            stream.put_llong(entry.pack_size);
        }
        stream.put_lshort(self.resource_pack_entries.len() as u16);
        for entry in &self.resource_pack_entries {
            stream.put_string(&entry.pack_id);
            stream.put_string(&entry.pack_version);
            stream.put_llong(entry.pack_size);
        }
    }

    fn decode(_buf: Vec<u8>) -> Self where Self: Sized {
        Self::default()
    }

    fn name(&self) -> &'static str { "ResourcePacksInfoPacket" }
    fn can_be_sent_before_login(&self) -> bool { true }
}
