use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;
use crate::protocol::types::EntityMetadata;
use super::entity_packets::Attribute;

pub struct AddEntityPacket {
    pub entity_runtime_id: i64,
    pub entity_type: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub speed_x: f32,
    pub speed_y: f32,
    pub speed_z: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub attributes: Vec<Attribute>,
    pub metadata: EntityMetadata,
    pub links: Vec<(i64, i64, u8)>,
}

impl AddEntityPacket {
    pub fn new(entity_runtime_id: i64, entity_type: u32, x: f32, y: f32, z: f32) -> Self {
        Self {
            entity_runtime_id, entity_type, x, y, z,
            speed_x: 0.0, speed_y: 0.0, speed_z: 0.0,
            yaw: 0.0, pitch: 0.0,
            attributes: Vec::new(),
            metadata: EntityMetadata::new(),
            links: Vec::new(),
        }
    }
}

impl DataPacket for AddEntityPacket {
    const NETWORK_ID: u8 = ProtocolInfo::ADD_ENTITY_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_entity_id(self.entity_runtime_id);
        stream.put_entity_id(self.entity_runtime_id);
        stream.put_unsigned_var_int(self.entity_type);
        stream.put_vector3f(self.x, self.y, self.z);
        stream.put_vector3f(self.speed_x, self.speed_y, self.speed_z);
        stream.put_lfloat(self.pitch);
        stream.put_lfloat(self.yaw);
        stream.put_unsigned_var_int(self.attributes.len() as u32);
        for attr in &self.attributes {
            stream.put_string(&attr.name);
            stream.put_lfloat(attr.min);
            stream.put_lfloat(attr.current);
            stream.put_lfloat(attr.max);
        }
        self.metadata.put(stream);
        stream.put_unsigned_var_int(self.links.len() as u32);
        for link in &self.links {
            stream.put_entity_id(link.0);
            stream.put_entity_id(link.1);
            stream.put_byte(link.2);
        }
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let _unique_id = stream.get_entity_id().unwrap_or(0);
        let entity_runtime_id = stream.get_entity_id().unwrap_or(0);
        let entity_type = stream.get_unsigned_var_int().unwrap_or(0);
        let (x, y, z) = stream.get_vector3f().unwrap_or((0.0, 0.0, 0.0));
        let (speed_x, speed_y, speed_z) = stream.get_vector3f().unwrap_or((0.0, 0.0, 0.0));
        let pitch = stream.get_lfloat().unwrap_or(0.0);
        let yaw = stream.get_lfloat().unwrap_or(0.0);
        let attr_count = stream.get_unsigned_var_int().unwrap_or(0) as usize;
        let mut attributes = Vec::with_capacity(attr_count);
        for _ in 0..attr_count {
            let name = stream.get_string().unwrap_or_default();
            let min = stream.get_lfloat().unwrap_or(0.0);
            let current = stream.get_lfloat().unwrap_or(0.0);
            let max = stream.get_lfloat().unwrap_or(0.0);
            attributes.push(Attribute::new(name, min, max, current));
        }
        let metadata = EntityMetadata::get(&mut stream);
        let link_count = stream.get_unsigned_var_int().unwrap_or(0) as usize;
        let mut links = Vec::with_capacity(link_count);
        for _ in 0..link_count {
            let from = stream.get_entity_id().unwrap_or(0);
            let to = stream.get_entity_id().unwrap_or(0);
            let ty = stream.get_byte().unwrap_or(0);
            links.push((from, to, ty));
        }
        Self {
            entity_runtime_id, entity_type, x, y, z,
            speed_x, speed_y, speed_z, yaw, pitch,
            attributes, metadata, links,
        }
    }

    fn name(&self) -> &'static str { "AddEntityPacket" }
}
