use std::collections::BTreeMap;
use crate::utils::BinaryStream;

pub const DATA_TYPE_BYTE: u8 = 0;
pub const DATA_TYPE_SHORT: u8 = 1;
pub const DATA_TYPE_INT: u8 = 2;
pub const DATA_TYPE_FLOAT: u8 = 3;
pub const DATA_TYPE_STRING: u8 = 4;
pub const DATA_TYPE_SLOT: u8 = 5;
pub const DATA_TYPE_POS: u8 = 6;
pub const DATA_TYPE_LONG: u8 = 7;
pub const DATA_TYPE_VECTOR3F: u8 = 8;

#[derive(Debug, Clone, PartialEq)]
pub enum MetadataValue {
    Byte(i8),
    Short(i16),
    Int(i32),
    Float(f32),
    String(String),
    Slot(crate::protocol::types::Item),
    Pos(i32, i32, i32),
    Long(i64),
    Vector3f(f32, f32, f32),
}

#[derive(Debug, Clone, Default)]
pub struct EntityMetadata {
    pub entries: BTreeMap<u32, (u8, MetadataValue)>,
}

impl EntityMetadata {
    pub fn new() -> Self { Self { entries: BTreeMap::new() } }

    pub fn set_byte(&mut self, key: u32, value: i8) {
        self.entries.insert(key, (DATA_TYPE_BYTE, MetadataValue::Byte(value)));
    }
    pub fn set_int(&mut self, key: u32, value: i32) {
        self.entries.insert(key, (DATA_TYPE_INT, MetadataValue::Int(value)));
    }
    pub fn set_float(&mut self, key: u32, value: f32) {
        self.entries.insert(key, (DATA_TYPE_FLOAT, MetadataValue::Float(value)));
    }
    pub fn set_string(&mut self, key: u32, value: String) {
        self.entries.insert(key, (DATA_TYPE_STRING, MetadataValue::String(value)));
    }
    pub fn set_long(&mut self, key: u32, value: i64) {
        self.entries.insert(key, (DATA_TYPE_LONG, MetadataValue::Long(value)));
    }
    pub fn set_vector3f(&mut self, key: u32, x: f32, y: f32, z: f32) {
        self.entries.insert(key, (DATA_TYPE_VECTOR3F, MetadataValue::Vector3f(x, y, z)));
    }
    pub fn set_pos(&mut self, key: u32, x: i32, y: i32, z: i32) {
        self.entries.insert(key, (DATA_TYPE_POS, MetadataValue::Pos(x, y, z)));
    }

    pub fn put(&self, stream: &mut BinaryStream) {
        stream.put_unsigned_var_int(self.entries.len() as u32);
        for (key, (ty, value)) in &self.entries {
            stream.put_unsigned_var_int(*key);
            stream.put_unsigned_var_int(*ty as u32);
            match value {
                MetadataValue::Byte(v) => stream.put_byte(*v as u8),
                MetadataValue::Short(v) => stream.put_lshort(*v as u16),
                MetadataValue::Int(v) => stream.put_var_int(*v),
                MetadataValue::Float(v) => stream.put_lfloat(*v),
                MetadataValue::String(v) => stream.put_string(v),
                MetadataValue::Slot(item) => item.put(stream),
                MetadataValue::Pos(x, y, z) => {
                    stream.put_var_int(*x);
                    stream.put_var_int(*y);
                    stream.put_var_int(*z);
                }
                MetadataValue::Long(v) => stream.put_var_long(*v),
                MetadataValue::Vector3f(x, y, z) => {
                    stream.put_lfloat(*x);
                    stream.put_lfloat(*y);
                    stream.put_lfloat(*z);
                }
            }
        }
    }

    pub fn get(stream: &mut BinaryStream) -> Self {
        let count = stream.get_unsigned_var_int().unwrap_or(0);
        let mut entries = BTreeMap::new();
        for _ in 0..count {
            if stream.eof() { break; }
            let key = stream.get_unsigned_var_int().unwrap_or(0);
            let ty = stream.get_unsigned_var_int().unwrap_or(0) as u8;
            let value = match ty {
                DATA_TYPE_BYTE => MetadataValue::Byte(stream.get_byte().unwrap_or(0) as i8),
                DATA_TYPE_SHORT => MetadataValue::Short(stream.get_signed_lshort().unwrap_or(0)),
                DATA_TYPE_INT => MetadataValue::Int(stream.get_var_int().unwrap_or(0)),
                DATA_TYPE_FLOAT => MetadataValue::Float(stream.get_lfloat().unwrap_or(0.0)),
                DATA_TYPE_STRING => MetadataValue::String(stream.get_string().unwrap_or_default()),
                DATA_TYPE_SLOT => {
                    let mut item = crate::protocol::types::Item::air();
                    item.get(stream);
                    MetadataValue::Slot(item)
                }
                DATA_TYPE_POS => {
                    let x = stream.get_var_int().unwrap_or(0);
                    let y = stream.get_var_int().unwrap_or(0);
                    let z = stream.get_var_int().unwrap_or(0);
                    MetadataValue::Pos(x, y, z)
                }
                DATA_TYPE_LONG => MetadataValue::Long(stream.get_var_long().unwrap_or(0)),
                DATA_TYPE_VECTOR3F => {
                    let x = stream.get_lfloat().unwrap_or(0.0);
                    let y = stream.get_lfloat().unwrap_or(0.0);
                    let z = stream.get_lfloat().unwrap_or(0.0);
                    MetadataValue::Vector3f(x, y, z)
                }
                _ => break,
            };
            entries.insert(key, (ty, value));
        }
        Self { entries }
    }
}
