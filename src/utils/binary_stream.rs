use super::binary::{Binary, Result as BResult, BinaryError};
use super::uuid::UUID;
use crate::math::Vec3;

#[derive(Debug, Clone)]
pub struct BinaryStream {
    pub buffer: Vec<u8>,
    pub offset: usize,
}

impl Default for BinaryStream {
    fn default() -> Self { Self::new() }
}

impl BinaryStream {
    pub fn new() -> Self {
        Self { buffer: Vec::new(), offset: 0 }
    }

    pub fn from_bytes(buffer: Vec<u8>) -> Self {
        Self { buffer, offset: 0 }
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
        self.offset = 0;
    }

    pub fn rewind(&mut self) { self.offset = 0; }

    pub fn set_offset(&mut self, offset: usize) { self.offset = offset; }

    pub fn len(&self) -> usize { self.buffer.len() }

    pub fn is_empty(&self) -> bool { self.buffer.is_empty() }

    pub fn eof(&self) -> bool { self.offset >= self.buffer.len() }

    pub fn remaining(&self) -> usize {
        self.buffer.len().saturating_sub(self.offset)
    }

    pub fn get(&mut self, len: usize) -> BResult<Vec<u8>> {
        if len == 0 { return Ok(Vec::new()); }
        if self.remaining() < len {
            return Err(BinaryError::NotEnoughBytes { needed: len, have: self.remaining() });
        }
        let data = self.buffer[self.offset..self.offset + len].to_vec();
        self.offset += len;
        Ok(data)
    }

    pub fn get_remaining(&mut self) -> BResult<Vec<u8>> {
        if self.eof() {
            return Err(BinaryError::NotEnoughBytes { needed: 1, have: 0 });
        }
        let data = self.buffer[self.offset..].to_vec();
        self.offset = self.buffer.len();
        Ok(data)
    }

    pub fn put(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    pub fn put_byte_arr(&mut self, data: Vec<u8>) {
        self.buffer.extend(data);
    }

    pub fn get_bool(&mut self) -> BResult<bool> {
        Ok(self.get(1)?[0] != 0)
    }

    pub fn put_bool(&mut self, v: bool) {
        self.buffer.push(if v { 1 } else { 0 });
    }

    pub fn get_byte(&mut self) -> BResult<u8> {
        Ok(self.get(1)?[0])
    }

    pub fn put_byte(&mut self, v: u8) {
        self.buffer.push(v);
    }

    pub fn get_signed_byte(&mut self) -> BResult<i8> {
        Ok(self.get_byte()? as i8)
    }

    pub fn get_short(&mut self) -> BResult<u16> {
        let b = self.get(2)?;
        Ok(Binary::read_short(&b))
    }

    pub fn get_signed_short(&mut self) -> BResult<i16> {
        Ok(self.get_short()? as i16)
    }

    pub fn put_short(&mut self, v: u16) {
        self.buffer.extend_from_slice(&Binary::write_short(v));
    }

    pub fn put_signed_short(&mut self, v: i16) {
        self.put_short(v as u16);
    }

    pub fn get_lshort(&mut self) -> BResult<u16> {
        let b = self.get(2)?;
        Ok(Binary::read_lshort(&b))
    }

    pub fn get_signed_lshort(&mut self) -> BResult<i16> {
        Ok(self.get_lshort()? as i16)
    }

    pub fn put_lshort(&mut self, v: u16) {
        self.buffer.extend_from_slice(&Binary::write_lshort(v));
    }

    pub fn get_triad(&mut self) -> BResult<u32> {
        let b = self.get(3)?;
        Ok(Binary::read_triad(&b))
    }

    pub fn put_triad(&mut self, v: u32) {
        self.buffer.extend_from_slice(&Binary::write_triad(v));
    }

    pub fn get_ltriad(&mut self) -> BResult<u32> {
        let b = self.get(3)?;
        Ok(Binary::read_ltriad(&b))
    }

    pub fn put_ltriad(&mut self, v: u32) {
        self.buffer.extend_from_slice(&Binary::write_ltriad(v));
    }

    pub fn get_int(&mut self) -> BResult<i32> {
        let b = self.get(4)?;
        Ok(Binary::read_signed_int(&b))
    }

    pub fn put_int(&mut self, v: i32) {
        self.buffer.extend_from_slice(&Binary::write_int(v as u32));
    }

    pub fn get_lint(&mut self) -> BResult<i32> {
        let b = self.get(4)?;
        Ok(Binary::read_signed_lint(&b))
    }

    pub fn put_lint(&mut self, v: i32) {
        self.buffer.extend_from_slice(&Binary::write_lint(v as u32));
    }

    pub fn get_float(&mut self) -> BResult<f32> {
        let b = self.get(4)?;
        Ok(Binary::read_float(&b))
    }

    pub fn put_float(&mut self, v: f32) {
        self.buffer.extend_from_slice(&Binary::write_float(v));
    }

    pub fn get_lfloat(&mut self) -> BResult<f32> {
        let b = self.get(4)?;
        Ok(Binary::read_lfloat(&b))
    }

    pub fn put_lfloat(&mut self, v: f32) {
        self.buffer.extend_from_slice(&Binary::write_lfloat(v));
    }

    pub fn get_long(&mut self) -> BResult<i64> {
        let b = self.get(8)?;
        Ok(Binary::read_long(&b) as i64)
    }

    pub fn put_long(&mut self, v: i64) {
        self.buffer.extend_from_slice(&Binary::write_long(v as u64));
    }

    pub fn get_llong(&mut self) -> BResult<i64> {
        let b = self.get(8)?;
        Ok(Binary::read_llong(&b) as i64)
    }

    pub fn put_llong(&mut self, v: i64) {
        self.buffer.extend_from_slice(&Binary::write_llong(v as u64));
    }

    pub fn get_string(&mut self) -> BResult<String> {
        let len = self.get_unsigned_var_int()? as usize;
        let bytes = self.get(len)?;
        String::from_utf8(bytes).map_err(|_| BinaryError::Invalid("invalid utf-8 string".into()))
    }

    pub fn put_string(&mut self, v: &str) {
        self.put_unsigned_var_int(v.len() as u32);
        self.buffer.extend_from_slice(v.as_bytes());
    }

    pub fn get_uuid(&mut self) -> BResult<UUID> {
        let p1 = self.get_lint()? as u32;
        let p0 = self.get_lint()? as u32;
        let p3 = self.get_lint()? as u32;
        let p2 = self.get_lint()? as u32;
        Ok(UUID::from_parts(p0, p1, p2, p3))
    }

    pub fn put_uuid(&mut self, uuid: UUID) {
        let parts = uuid.parts();
        self.put_lint(parts.1 as i32);
        self.put_lint(parts.0 as i32);
        self.put_lint(parts.3 as i32);
        self.put_lint(parts.2 as i32);
    }

    pub fn get_unsigned_var_int(&mut self) -> BResult<u32> {
        Binary::read_unsigned_var_int(&self.buffer, &mut self.offset)
    }

    pub fn put_unsigned_var_int(&mut self, v: u32) {
        self.buffer.extend(Binary::write_unsigned_var_int(v));
    }

    pub fn get_var_int(&mut self) -> BResult<i32> {
        Binary::read_var_int(&self.buffer, &mut self.offset)
    }

    pub fn put_var_int(&mut self, v: i32) {
        self.buffer.extend(Binary::write_var_int(v));
    }

    pub fn get_unsigned_var_long(&mut self) -> BResult<u64> {
        Binary::read_unsigned_var_long(&self.buffer, &mut self.offset)
    }

    pub fn put_unsigned_var_long(&mut self, v: u64) {
        self.buffer.extend(Binary::write_unsigned_var_long(v));
    }

    pub fn get_var_long(&mut self) -> BResult<i64> {
        Binary::read_var_long(&self.buffer, &mut self.offset)
    }

    pub fn put_var_long(&mut self, v: i64) {
        self.buffer.extend(Binary::write_var_long(v));
    }

    pub fn get_entity_id(&mut self) -> BResult<i64> {
        self.get_var_int().map(|v| v as i64)
    }

    pub fn put_entity_id(&mut self, v: i64) {
        self.put_var_int(v as i32);
    }

    pub fn get_block_coords(&mut self) -> BResult<(i32, u32, i32)> {
        let x = self.get_var_int()?;
        let y = self.get_unsigned_var_int()?;
        let z = self.get_var_int()?;
        Ok((x, y, z))
    }

    pub fn put_block_coords(&mut self, x: i32, y: u32, z: i32) {
        self.put_var_int(x);
        self.put_unsigned_var_int(y);
        self.put_var_int(z);
    }

    pub fn get_signed_block_coords(&mut self) -> BResult<(i32, i32, i32)> {
        let x = self.get_var_int()?;
        let y = self.get_var_int()?;
        let z = self.get_var_int()?;
        Ok((x, y, z))
    }

    pub fn put_signed_block_coords(&mut self, x: i32, y: i32, z: i32) {
        self.put_var_int(x);
        self.put_var_int(y);
        self.put_var_int(z);
    }

    pub fn get_vector3f(&mut self) -> BResult<(f32, f32, f32)> {
        let x = self.get_lfloat()?;
        let y = self.get_lfloat()?;
        let z = self.get_lfloat()?;
        Ok((x, y, z))
    }

    pub fn put_vector3f(&mut self, x: f32, y: f32, z: f32) {
        self.put_lfloat(x);
        self.put_lfloat(y);
        self.put_lfloat(z);
    }

    pub fn get_vec3(&mut self) -> BResult<Vec3> {
        let (x, y, z) = self.get_vector3f()?;
        Ok(Vec3::new(x as f64, y as f64, z as f64))
    }

    pub fn put_vec3(&mut self, v: Vec3) {
        self.put_vector3f(v.x as f32, v.y as f32, v.z as f32);
    }

    pub fn get_byte_rotation(&mut self) -> BResult<f32> {
        let b = self.get_byte()?;
        Ok(b as f32 * (360.0 / 256.0))
    }

    pub fn put_byte_rotation(&mut self, rotation: f32) {
        let v = (rotation / (360.0 / 256.0)) as i32 & 0xff;
        self.put_byte(v as u8);
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.buffer
    }
}
