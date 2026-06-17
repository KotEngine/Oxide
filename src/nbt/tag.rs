use std::collections::BTreeMap;

pub const TAG_END: u8 = 0;
pub const TAG_BYTE: u8 = 1;
pub const TAG_SHORT: u8 = 2;
pub const TAG_INT: u8 = 3;
pub const TAG_LONG: u8 = 4;
pub const TAG_FLOAT: u8 = 5;
pub const TAG_DOUBLE: u8 = 6;
pub const TAG_BYTE_ARRAY: u8 = 7;
pub const TAG_STRING: u8 = 8;
pub const TAG_LIST: u8 = 9;
pub const TAG_COMPOUND: u8 = 10;
pub const TAG_INT_ARRAY: u8 = 11;

#[derive(Debug, Clone, PartialEq)]
pub enum NbtValue {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(Vec<NbtValue>),
    Compound(NbtCompound),
    IntArray(Vec<i32>),
}

pub type NbtCompound = BTreeMap<String, NbtValue>;

impl NbtValue {
    pub fn tag_id(&self) -> u8 {
        match self {
            Self::End => TAG_END,
            Self::Byte(_) => TAG_BYTE,
            Self::Short(_) => TAG_SHORT,
            Self::Int(_) => TAG_INT,
            Self::Long(_) => TAG_LONG,
            Self::Float(_) => TAG_FLOAT,
            Self::Double(_) => TAG_DOUBLE,
            Self::ByteArray(_) => TAG_BYTE_ARRAY,
            Self::String(_) => TAG_STRING,
            Self::List(_) => TAG_LIST,
            Self::Compound(_) => TAG_COMPOUND,
            Self::IntArray(_) => TAG_INT_ARRAY,
        }
    }

    pub fn from_tag_id(id: u8) -> Option<Self> {
        Some(match id {
            TAG_END => Self::End,
            TAG_BYTE => Self::Byte(0),
            TAG_SHORT => Self::Short(0),
            TAG_INT => Self::Int(0),
            TAG_LONG => Self::Long(0),
            TAG_FLOAT => Self::Float(0.0),
            TAG_DOUBLE => Self::Double(0.0),
            TAG_BYTE_ARRAY => Self::ByteArray(Vec::new()),
            TAG_STRING => Self::String(String::new()),
            TAG_LIST => Self::List(Vec::new()),
            TAG_COMPOUND => Self::Compound(NbtCompound::new()),
            TAG_INT_ARRAY => Self::IntArray(Vec::new()),
            _ => return None,
        })
    }

    pub fn as_compound(&self) -> Option<&NbtCompound> {
        if let Self::Compound(c) = self { Some(c) } else { None }
    }
    pub fn as_compound_mut(&mut self) -> Option<&mut NbtCompound> {
        if let Self::Compound(c) = self { Some(c) } else { None }
    }
    pub fn as_string(&self) -> Option<&str> {
        if let Self::String(s) = self { Some(s) } else { None }
    }
    pub fn as_int(&self) -> Option<i32> {
        match self {
            Self::Int(v) => Some(*v),
            Self::Byte(v) => Some(*v as i32),
            Self::Short(v) => Some(*v as i32),
            Self::Long(v) => Some(*v as i32),
            _ => None,
        }
    }
    pub fn as_list(&self) -> Option<&Vec<NbtValue>> {
        if let Self::List(l) = self { Some(l) } else { None }
    }
}

pub struct Tag;

impl Tag {
    pub fn compound() -> NbtValue {
        NbtValue::Compound(NbtCompound::new())
    }

    pub fn create_compound() -> NbtCompound {
        NbtCompound::new()
    }
}
