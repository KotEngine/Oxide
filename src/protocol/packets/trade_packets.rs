use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;
use crate::protocol::types::WindowTypes;

pub struct UpdateTradePacket {
    pub window_id: u8,
    pub window_type: i8,
    pub varint1: i32,
    pub varint2: i32,
    pub is_willing: bool,
    pub trader_eid: i64,
    pub player_eid: i64,
    pub display_name: String,
    /// Raw NBT data containing trade recipes
    pub offers: Vec<u8>,
}

impl Default for UpdateTradePacket {
    fn default() -> Self {
        Self {
            window_id: 0,
            window_type: WindowTypes::TRADING,
            varint1: 0,
            varint2: 0,
            is_willing: false,
            trader_eid: 0,
            player_eid: 0,
            display_name: String::new(),
            offers: Vec::new(),
        }
    }
}

impl DataPacket for UpdateTradePacket {
    const NETWORK_ID: u8 = ProtocolInfo::UPDATE_TRADE_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_byte(self.window_id);
        stream.put_byte(self.window_type as u8);
        stream.put_var_int(self.varint1);
        stream.put_var_int(self.varint2);
        stream.put_bool(self.is_willing);
        stream.put_entity_id(self.trader_eid);
        stream.put_entity_id(self.player_eid);
        stream.put_string(&self.display_name);
        stream.put(&self.offers);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let window_id = stream.get_byte().unwrap_or(0);
        let window_type = stream.get_byte().unwrap_or(WindowTypes::TRADING as u8) as i8;
        let varint1 = stream.get_var_int().unwrap_or(0);
        let varint2 = stream.get_var_int().unwrap_or(0);
        let is_willing = stream.get_bool().unwrap_or(false);
        let trader_eid = stream.get_entity_id().unwrap_or(0);
        let player_eid = stream.get_entity_id().unwrap_or(0);
        let display_name = stream.get_string().unwrap_or_default();
        let offers = stream.get_remaining().unwrap_or_default();
        Self {
            window_id, window_type, varint1, varint2,
            is_willing, trader_eid, player_eid, display_name, offers,
        }
    }

    fn name(&self) -> &'static str { "UpdateTradePacket" }
}

pub struct UpdateEquipPacket {
    pub window_id: u8,
    pub window_type: i8,
    pub unknown_varint: i32,
    pub entity_unique_id: i64,
    /// Raw NBT data
    pub namedtag: Vec<u8>,
}

impl DataPacket for UpdateEquipPacket {
    const NETWORK_ID: u8 = ProtocolInfo::UPDATE_EQUIP_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_byte(self.window_id);
        stream.put_byte(self.window_type as u8);
        stream.put_var_int(self.unknown_varint);
        stream.put_entity_id(self.entity_unique_id);
        stream.put(&self.namedtag);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        let window_id = s.get_byte().unwrap_or(0);
        let window_type = s.get_byte().unwrap_or(0) as i8;
        let unknown_varint = s.get_var_int().unwrap_or(0);
        let entity_unique_id = s.get_entity_id().unwrap_or(0);
        let namedtag = s.get_remaining().unwrap_or_default();
        Self { window_id, window_type, unknown_varint, entity_unique_id, namedtag }
    }

    fn name(&self) -> &'static str { "UpdateEquipPacket" }
}
