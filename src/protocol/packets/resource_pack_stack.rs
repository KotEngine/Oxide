use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

#[derive(Debug, Clone)]
pub struct ResourcePackStackEntry {
    pub pack_id: String,
    pub pack_version: String,
}

pub struct ResourcePackStackPacket {
    pub must_accept: bool,
    pub behavior_pack_stack: Vec<ResourcePackStackEntry>,
    pub resource_pack_stack: Vec<ResourcePackStackEntry>,
}

impl Default for ResourcePackStackPacket {
    fn default() -> Self {
        Self { must_accept: false, behavior_pack_stack: Vec::new(), resource_pack_stack: Vec::new() }
    }
}

impl DataPacket for ResourcePackStackPacket {
    const NETWORK_ID: u8 = ProtocolInfo::RESOURCE_PACK_STACK_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_bool(self.must_accept);
        stream.put_unsigned_var_int(self.behavior_pack_stack.len() as u32);
        for entry in &self.behavior_pack_stack {
            stream.put_string(&entry.pack_id);
            stream.put_string(&entry.pack_version);
        }
        stream.put_unsigned_var_int(self.resource_pack_stack.len() as u32);
        for entry in &self.resource_pack_stack {
            stream.put_string(&entry.pack_id);
            stream.put_string(&entry.pack_version);
        }
    }

    fn decode(_buf: Vec<u8>) -> Self where Self: Sized {
        Self::default()
    }

    fn name(&self) -> &'static str { "ResourcePackStackPacket" }
    fn can_be_sent_before_login(&self) -> bool { true }
}
