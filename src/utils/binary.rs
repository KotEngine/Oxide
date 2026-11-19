use thiserror::Error;

#[derive(Debug, Error)]
pub enum BinaryError {
    #[error("not enough bytes left in buffer: need {needed}, have {have}")]
    NotEnoughBytes { needed: usize, have: usize },
    #[error("varint did not terminate after 5 bytes")]
    VarIntTooBig,
    #[error("varlong did not terminate after 10 bytes")]
    VarLongTooBig,
    #[error("invalid value: {0}")]
    Invalid(String),
}

pub type Result<T> = std::result::Result<T, BinaryError>;

pub struct Binary;

impl Binary {
    #[inline]
    pub fn sign_byte(value: u8) -> i8 { value as i8 }

    #[inline]
    pub fn read_byte(b: u8) -> u8 { b }

    #[inline]
    pub fn read_signed_byte(b: u8) -> i8 { b as i8 }

    #[inline]
    pub fn write_byte(c: u8) -> [u8; 1] { [c] }

    #[inline]
    pub fn read_short(b: &[u8]) -> u16 {
        ((b[0] as u16) << 8) | b[1] as u16
    }

    #[inline]
    pub fn read_signed_short(b: &[u8]) -> i16 {
        Self::read_short(b) as i16
    }

    #[inline]
    pub fn write_short(v: u16) -> [u8; 2] {
        [(v >> 8) as u8, v as u8]
    }

    #[inline]
    pub fn read_lshort(b: &[u8]) -> u16 {
        ((b[1] as u16) << 8) | b[0] as u16
    }

    #[inline]
    pub fn read_signed_lshort(b: &[u8]) -> i16 {
        Self::read_lshort(b) as i16
    }

    #[inline]
    pub fn write_lshort(v: u16) -> [u8; 2] {
        [v as u8, (v >> 8) as u8]
    }

    #[inline]
    pub fn read_triad(b: &[u8]) -> u32 {
        ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32
    }

    #[inline]
    pub fn write_triad(v: u32) -> [u8; 3] {
        [(v >> 16) as u8, (v >> 8) as u8, v as u8]
    }

    #[inline]
    pub fn read_ltriad(b: &[u8]) -> u32 {
        ((b[2] as u32) << 16) | ((b[1] as u32) << 8) | b[0] as u32
    }

    #[inline]
    pub fn write_ltriad(v: u32) -> [u8; 3] {
        [v as u8, (v >> 8) as u8, (v >> 16) as u8]
    }

    #[inline]
    pub fn read_int(b: &[u8]) -> u32 {
        ((b[0] as u32) << 24) | ((b[1] as u32) << 16) | ((b[2] as u32) << 8) | b[3] as u32
    }

    #[inline]
    pub fn read_signed_int(b: &[u8]) -> i32 {
        Self::read_int(b) as i32
    }

    #[inline]
    pub fn write_int(v: u32) -> [u8; 4] {
        [(v >> 24) as u8, (v >> 16) as u8, (v >> 8) as u8, v as u8]
    }

    #[inline]
    pub fn read_lint(b: &[u8]) -> u32 {
        ((b[3] as u32) << 24) | ((b[2] as u32) << 16) | ((b[1] as u32) << 8) | b[0] as u32
    }

    #[inline]
    pub fn read_signed_lint(b: &[u8]) -> i32 {
        Self::read_lint(b) as i32
    }

    #[inline]
    pub fn write_lint(v: u32) -> [u8; 4] {
        [v as u8, (v >> 8) as u8, (v >> 16) as u8, (v >> 24) as u8]
    }

    #[inline]
    pub fn read_float(b: &[u8]) -> f32 {
        f32::from_be_bytes([b[0], b[1], b[2], b[3]])
    }

    #[inline]
    pub fn write_float(v: f32) -> [u8; 4] {
        v.to_be_bytes()
    }

    #[inline]
    pub fn read_lfloat(b: &[u8]) -> f32 {
        f32::from_le_bytes([b[0], b[1], b[2], b[3]])
    }

    #[inline]
    pub fn write_lfloat(v: f32) -> [u8; 4] {
        v.to_le_bytes()
    }

    #[inline]
    pub fn read_long(b: &[u8]) -> u64 {
        u64::from_be_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])
    }

    #[inline]
    pub fn write_long(v: u64) -> [u8; 8] {
        v.to_be_bytes()
    }

    #[inline]
    pub fn read_llong(b: &[u8]) -> u64 {
        u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])
    }

    #[inline]
    pub fn write_llong(v: u64) -> [u8; 8] {
        v.to_le_bytes()
    }

    pub fn read_unsigned_var_int(buf: &[u8], offset: &mut usize) -> Result<u32> {
        let mut value: u32 = 0;
        let mut i = 0usize;
        loop {
            if i > 28 {
                return Err(BinaryError::VarIntTooBig);
            }
            if *offset >= buf.len() {
                return Err(BinaryError::NotEnoughBytes { needed: 1, have: 0 });
            }
            let b = buf[*offset];
            *offset += 1;
            value |= ((b & 0x7f) as u32) << i;
            if b & 0x80 == 0 {
                return Ok(value);
            }
            i += 7;
        }
    }

    pub fn read_var_int(buf: &[u8], offset: &mut usize) -> Result<i32> {
        let raw = Self::read_unsigned_var_int(buf, offset)?;
        Ok(((raw >> 1) as i32) ^ (-((raw & 1) as i32)))
    }

    pub fn write_unsigned_var_int(mut value: u32) -> Vec<u8> {
        let mut buf = Vec::with_capacity(5);
        loop {
            if (value >> 7) != 0 {
                buf.push((value as u8) | 0x80);
            } else {
                buf.push(value as u8);
                return buf;
            }
            value >>= 7;
        }
    }

    pub fn write_var_int(v: i32) -> Vec<u8> {
        let zigzag = (v.wrapping_shl(1) ^ (v >> 31)) as u32;
        Self::write_unsigned_var_int(zigzag)
    }

    pub fn read_unsigned_var_long(buf: &[u8], offset: &mut usize) -> Result<u64> {
        let mut value: u64 = 0;
        let mut i = 0usize;
        loop {
            if i > 63 {
                return Err(BinaryError::VarLongTooBig);
            }
            if *offset >= buf.len() {
                return Err(BinaryError::NotEnoughBytes { needed: 1, have: 0 });
            }
            let b = buf[*offset];
            *offset += 1;
            value |= ((b & 0x7f) as u64) << i;
            if b & 0x80 == 0 {
                return Ok(value);
            }
            i += 7;
        }
    }

    pub fn read_var_long(buf: &[u8], offset: &mut usize) -> Result<i64> {
        let raw = Self::read_unsigned_var_long(buf, offset)?;
        Ok(((raw >> 1) as i64) ^ (-((raw & 1) as i64)))
    }

    pub fn write_unsigned_var_long(mut value: u64) -> Vec<u8> {
        let mut buf = Vec::with_capacity(10);
        loop {
            if (value >> 7) != 0 {
                buf.push((value as u8) | 0x80);
            } else {
                buf.push(value as u8);
                return buf;
            }
            value >>= 7;
        }
    }

    pub fn write_var_long(v: i64) -> Vec<u8> {
        let zigzag = (v.wrapping_shl(1) ^ (v >> 63)) as u64;
        Self::write_unsigned_var_long(zigzag)
    }
}
