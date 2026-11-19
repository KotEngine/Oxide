use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

#[derive(Debug, Clone)]
pub struct CommandStepPacket {
    pub command: String,
    pub overload: String,
    pub uvarint1: u32,
    pub current_step: u32,
    pub done: bool,
    pub client_id: u64,
    pub input_json: String,
    pub output_json: String,
}

impl DataPacket for CommandStepPacket {
    const NETWORK_ID: u8 = ProtocolInfo::COMMAND_STEP_PACKET;

    // Client to server packet only, payload encode is empty
    fn encode_payload(&self, _stream: &mut BinaryStream) {}

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let command = s.get_string().unwrap_or_default();
        let overload = s.get_string().unwrap_or_default();
        let uvarint1 = s.get_unsigned_var_int().unwrap_or(0);
        let current_step = s.get_unsigned_var_int().unwrap_or(0);
        let done = s.get_bool().unwrap_or(false);
        let client_id = s.get_unsigned_var_long().unwrap_or(0);
        let input_json = s.get_string().unwrap_or_default();
        let output_json = s.get_string().unwrap_or_default();
        Self {
            command, overload, uvarint1, current_step,
            done, client_id, input_json, output_json,
        }
    }

    fn name(&self) -> &'static str { "CommandStepPacket" }
}

pub struct CommandBlockUpdatePacket {
    pub is_block: bool,
    pub x: i32,
    pub y: u32,
    pub z: i32,
    pub command_block_mode: u32,
    pub is_redstone_mode: bool,
    pub is_conditional: bool,
    pub minecart_eid: i64,
    pub command: String,
    pub last_output: String,
    pub name: String,
    pub should_track_output: bool,
}

impl DataPacket for CommandBlockUpdatePacket {
    const NETWORK_ID: u8 = ProtocolInfo::COMMAND_BLOCK_UPDATE_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_bool(self.is_block);
        if self.is_block {
            stream.put_block_coords(self.x, self.y, self.z);
            stream.put_unsigned_var_int(self.command_block_mode);
            stream.put_bool(self.is_redstone_mode);
            stream.put_bool(self.is_conditional);
        } else {
            stream.put_entity_id(self.minecart_eid);
        }
        stream.put_string(&self.command);
        stream.put_string(&self.last_output);
        stream.put_string(&self.name);
        stream.put_bool(self.should_track_output);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let is_block = s.get_bool().unwrap_or(false);
        let mut x = 0;
        let mut y = 0;
        let mut z = 0;
        let mut command_block_mode = 0;
        let mut is_redstone_mode = false;
        let mut is_conditional = false;
        let mut minecart_eid = 0;
        if is_block {
            let (bx, by, bz) = s.get_block_coords().unwrap_or((0, 0, 0));
            x = bx;
            y = by;
            z = bz;
            command_block_mode = s.get_unsigned_var_int().unwrap_or(0);
            is_redstone_mode = s.get_bool().unwrap_or(false);
            is_conditional = s.get_bool().unwrap_or(false);
        } else {
            minecart_eid = s.get_entity_id().unwrap_or(0);
        }
        let command = s.get_string().unwrap_or_default();
        let last_output = s.get_string().unwrap_or_default();
        let name = s.get_string().unwrap_or_default();
        let should_track_output = s.get_bool().unwrap_or(false);
        Self {
            is_block, x, y, z, command_block_mode, is_redstone_mode, is_conditional,
            minecart_eid, command, last_output, name, should_track_output,
        }
    }

    fn name(&self) -> &'static str { "CommandBlockUpdatePacket" }
}
