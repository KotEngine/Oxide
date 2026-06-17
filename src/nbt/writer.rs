use super::tag::{NbtValue, NbtCompound, TAG_END, TAG_BYTE, TAG_SHORT, TAG_INT, TAG_LONG, TAG_FLOAT, TAG_DOUBLE, TAG_BYTE_ARRAY, TAG_STRING, TAG_LIST, TAG_COMPOUND, TAG_INT_ARRAY};
use crate::utils::BinaryStream;

pub struct NbtWriter {
    stream: BinaryStream,
}

impl Default for NbtWriter {
    fn default() -> Self { Self::new() }
}

impl NbtWriter {
    pub fn new() -> Self {
        Self { stream: BinaryStream::new() }
    }

    pub fn write_root(&mut self, name: &str, value: &NbtValue) {
        self.stream.put_byte(value.tag_id());
        self.write_string(name);
        self.write_value(value);
    }

    fn write_value(&mut self, value: &NbtValue) {
        match value {
            NbtValue::End => {}
            NbtValue::Byte(v) => self.stream.put_byte(*v as u8),
            NbtValue::Short(v) => self.stream.put_lshort(*v as u16),
            NbtValue::Int(v) => self.stream.put_lint(*v),
            NbtValue::Long(v) => self.stream.put_llong(*v),
            NbtValue::Float(v) => self.stream.put_lfloat(*v),
            NbtValue::Double(v) => self.stream.put(&v.to_le_bytes()),
            NbtValue::ByteArray(arr) => {
                self.stream.put_lint(arr.len() as i32);
                for &b in arr {
                    self.stream.put_byte(b as u8);
                }
            }
            NbtValue::String(s) => self.write_string(s),
            NbtValue::List(list) => {
                let list_type = if list.is_empty() { TAG_END } else { list[0].tag_id() };
                self.stream.put_byte(list_type);
                self.stream.put_lint(list.len() as i32);
                for item in list {
                    self.write_value(item);
                }
            }
            NbtValue::Compound(compound) => {
                self.write_compound(compound);
            }
            NbtValue::IntArray(arr) => {
                self.stream.put_lint(arr.len() as i32);
                for &v in arr {
                    self.stream.put_lint(v);
                }
            }
        }
    }

    fn write_compound(&mut self, compound: &NbtCompound) {
        for (name, value) in compound {
            self.stream.put_byte(value.tag_id());
            self.write_string(name);
            self.write_value(value);
        }
        self.stream.put_byte(TAG_END);
    }

    fn write_string(&mut self, s: &str) {
        self.stream.put_lshort(s.len() as u16);
        self.stream.put(s.as_bytes());
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.stream.into_inner()
    }
}
