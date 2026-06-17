use super::tag::{NbtValue, NbtCompound, TAG_END, TAG_BYTE, TAG_SHORT, TAG_INT, TAG_LONG, TAG_FLOAT, TAG_DOUBLE, TAG_BYTE_ARRAY, TAG_STRING, TAG_LIST, TAG_COMPOUND, TAG_INT_ARRAY};
use crate::utils::{BinaryStream, binary::BinaryError};

pub struct NbtReader<'a> {
    stream: BinaryStream,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> NbtReader<'a> {
    pub fn new(data: Vec<u8>) -> Self {
        Self { stream: BinaryStream::from_bytes(data), _phantom: std::marker::PhantomData }
    }

    pub fn read_root(&mut self) -> Result<(String, NbtValue), BinaryError> {
        let tag_id = self.stream.get_byte()?;
        if tag_id == TAG_END {
            return Ok((String::new(), NbtValue::End));
        }
        let name = self.read_string()?;
        let value = self.read_value(tag_id)?;
        Ok((name, value))
    }

    fn read_value(&mut self, tag_id: u8) -> Result<NbtValue, BinaryError> {
        Ok(match tag_id {
            TAG_END => NbtValue::End,
            TAG_BYTE => NbtValue::Byte(self.stream.get_signed_byte()?),
            TAG_SHORT => NbtValue::Short(self.stream.get_signed_lshort()?),
            TAG_INT => NbtValue::Int(self.stream.get_lint()?),
            TAG_LONG => NbtValue::Long(self.stream.get_llong()?),
            TAG_FLOAT => NbtValue::Float(self.stream.get_lfloat()?),
            TAG_DOUBLE => {
                let b = self.stream.get(8)?;
                let v = f64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                NbtValue::Double(v)
            }
            TAG_BYTE_ARRAY => {
                let len = self.stream.get_lint()?;
                let mut arr = Vec::with_capacity(len.max(0) as usize);
                for _ in 0..len.max(0) {
                    arr.push(self.stream.get_signed_byte()?);
                }
                NbtValue::ByteArray(arr)
            }
            TAG_STRING => NbtValue::String(self.read_string()?),
            TAG_LIST => {
                let list_type = self.stream.get_byte()?;
                let count = self.stream.get_lint()?.max(0) as usize;
                let mut list = Vec::with_capacity(count);
                for _ in 0..count {
                    list.push(self.read_value(list_type)?);
                }
                NbtValue::List(list)
            }
            TAG_COMPOUND => {
                let mut compound = NbtCompound::new();
                loop {
                    let child_id = self.stream.get_byte()?;
                    if child_id == TAG_END { break; }
                    let name = self.read_string()?;
                    let value = self.read_value(child_id)?;
                    compound.insert(name, value);
                }
                NbtValue::Compound(compound)
            }
            TAG_INT_ARRAY => {
                let len = self.stream.get_lint()?.max(0) as usize;
                let mut arr = Vec::with_capacity(len);
                for _ in 0..len {
                    arr.push(self.stream.get_lint()?);
                }
                NbtValue::IntArray(arr)
            }
            _ => return Err(BinaryError::Invalid(format!("unknown NBT tag id {}", tag_id))),
        })
    }

    fn read_string(&mut self) -> Result<String, BinaryError> {
        let len = self.stream.get_lshort()? as usize;
        let bytes = self.stream.get(len)?;
        String::from_utf8(bytes).map_err(|_| BinaryError::Invalid("invalid utf-8 in nbt string".into()))
    }
}
