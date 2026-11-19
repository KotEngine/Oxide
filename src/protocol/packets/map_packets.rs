use crate::utils::{BinaryStream, Color};
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

pub const BITFLAG_TEXTURE_UPDATE: u32 = 0x02;
pub const BITFLAG_DECORATION_UPDATE: u32 = 0x04;
pub const BITFLAG_ENTITY_UPDATE: u32 = 0x08;

#[derive(Debug, Clone)]
pub struct MapDecoration {
    pub rot: i32,
    pub img: i32,
    pub x_offset: u8,
    pub y_offset: u8,
    pub label: String,
    pub color: Color,
}

pub struct ClientboundMapItemDataPacket {
    pub map_id: i64,
    pub entity_runtime_ids: Vec<i64>,
    pub scale: u8,
    pub decorations: Vec<MapDecoration>,
    pub width: i32,
    pub height: i32,
    pub x_offset: i32,
    pub y_offset: i32,
    /// Colors in row-major order: `colors[y][x]`, dimension must be height x width.
    pub colors: Vec<Vec<Color>>,
}

impl Default for ClientboundMapItemDataPacket {
    fn default() -> Self {
        Self {
            map_id: 0,
            entity_runtime_ids: Vec::new(),
            scale: 0,
            decorations: Vec::new(),
            width: 0,
            height: 0,
            x_offset: 0,
            y_offset: 0,
            colors: Vec::new(),
        }
    }
}

impl DataPacket for ClientboundMapItemDataPacket {
    const NETWORK_ID: u8 = ProtocolInfo::CLIENTBOUND_MAP_ITEM_DATA_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_entity_id(self.map_id);

        let mut flags = 0u32;
        if !self.entity_runtime_ids.is_empty() {
            flags |= BITFLAG_ENTITY_UPDATE;
        }
        if !self.decorations.is_empty() {
            flags |= BITFLAG_DECORATION_UPDATE;
        }
        if !self.colors.is_empty() {
            flags |= BITFLAG_TEXTURE_UPDATE;
        }
        stream.put_unsigned_var_int(flags);

        if flags & BITFLAG_ENTITY_UPDATE != 0 {
            stream.put_unsigned_var_int(self.entity_runtime_ids.len() as u32);
            for eid in &self.entity_runtime_ids {
                stream.put_entity_id(*eid);
            }
        }

        if flags & (BITFLAG_TEXTURE_UPDATE | BITFLAG_DECORATION_UPDATE) != 0 {
            stream.put_byte(self.scale);
        }

        if flags & BITFLAG_DECORATION_UPDATE != 0 {
            stream.put_unsigned_var_int(self.decorations.len() as u32);
            for d in &self.decorations {
                stream.put_var_int((d.rot & 0x0f) | (d.img << 4));
                stream.put_byte(d.x_offset);
                stream.put_byte(d.y_offset);
                stream.put_string(&d.label);
                stream.put_lint(d.color.to_abgr() as i32);
            }
        }

        if flags & BITFLAG_TEXTURE_UPDATE != 0 {
            stream.put_var_int(self.width);
            stream.put_var_int(self.height);
            stream.put_var_int(self.x_offset);
            stream.put_var_int(self.y_offset);
            for y in 0..self.height as usize {
                for x in 0..self.width as usize {
                    let color = self.colors
                        .get(y)
                        .and_then(|row| row.get(x))
                        .copied()
                        .unwrap_or(Color::rgba(0, 0, 0, 0));
                    stream.put_unsigned_var_int(color.to_abgr());
                }
            }
        }
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let map_id = stream.get_entity_id().unwrap_or(0);
        let flags = stream.get_unsigned_var_int().unwrap_or(0);

        let mut entity_runtime_ids = Vec::new();
        if flags & BITFLAG_ENTITY_UPDATE != 0 {
            let count = stream.get_unsigned_var_int().unwrap_or(0) as usize;
            for _ in 0..count {
                entity_runtime_ids.push(stream.get_entity_id().unwrap_or(0));
            }
        }

        let mut scale = 0;
        if flags & (BITFLAG_TEXTURE_UPDATE | BITFLAG_DECORATION_UPDATE) != 0 {
            scale = stream.get_byte().unwrap_or(0);
        }

        let mut decorations = Vec::new();
        if flags & BITFLAG_DECORATION_UPDATE != 0 {
            let count = stream.get_unsigned_var_int().unwrap_or(0) as usize;
            for _ in 0..count {
                let rot_img = stream.get_var_int().unwrap_or(0);
                let rot = rot_img & 0x0f;
                let img = rot_img >> 4;
                let x_offset = stream.get_byte().unwrap_or(0);
                let y_offset = stream.get_byte().unwrap_or(0);
                let label = stream.get_string().unwrap_or_default();
                let abgr = stream.get_lint().unwrap_or(0) as u32;
                decorations.push(MapDecoration {
                    rot, img, x_offset, y_offset, label, color: Color::from_abgr(abgr),
                });
            }
        }

        let mut width = 0;
        let mut height = 0;
        let mut x_offset = 0;
        let mut y_offset = 0;
        let mut colors = Vec::new();
        if flags & BITFLAG_TEXTURE_UPDATE != 0 {
            width = stream.get_var_int().unwrap_or(0);
            height = stream.get_var_int().unwrap_or(0);
            x_offset = stream.get_var_int().unwrap_or(0);
            y_offset = stream.get_var_int().unwrap_or(0);
            for _ in 0..height {
                let mut row = Vec::with_capacity(width as usize);
                for _ in 0..width {
                    let c = stream.get_unsigned_var_int().unwrap_or(0);
                    row.push(Color::from_abgr(c));
                }
                colors.push(row);
            }
        }

        Self {
            map_id, entity_runtime_ids, scale, decorations,
            width, height, x_offset, y_offset, colors,
        }
    }

    fn name(&self) -> &'static str { "ClientboundMapItemDataPacket" }
}

pub struct MapInfoRequestPacket {
    pub map_id: i64,
}

impl DataPacket for MapInfoRequestPacket {
    const NETWORK_ID: u8 = ProtocolInfo::MAP_INFO_REQUEST_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_entity_id(self.map_id);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut s = BinaryStream::from_bytes(buf);
        let _ = s.get_byte();
        Self { map_id: s.get_entity_id().unwrap_or(0) }
    }

    fn name(&self) -> &'static str { "MapInfoRequestPacket" }
}
