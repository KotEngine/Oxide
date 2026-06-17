use std::ops::{Add, Sub, Mul, Neg};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Default for BlockPos {
    fn default() -> Self { Self { x: 0, y: 0, z: 0 } }
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self { Self { x, y, z } }

    pub fn from_vec3(v: crate::math::Vec3) -> Self {
        Self {
            x: v.x.floor() as i32,
            y: v.y.floor() as i32,
            z: v.z.floor() as i32,
        }
    }

    pub fn chunk_x(self) -> i32 { self.x >> 4 }
    pub fn chunk_z(self) -> i32 { self.z >> 4 }

    pub fn in_chunk_x(self) -> u8 { (self.x & 0x0f) as u8 }
    pub fn in_chunk_z(self) -> u8 { (self.z & 0x0f) as u8 }

    pub fn up(self) -> Self { self + Self::new(0, 1, 0) }
    pub fn down(self) -> Self { self + Self::new(0, -1, 0) }
    pub fn north(self) -> Self { self + Self::new(0, 0, -1) }
    pub fn south(self) -> Self { self + Self::new(0, 0, 1) }
    pub fn east(self) -> Self { self + Self::new(1, 0, 0) }
    pub fn west(self) -> Self { self + Self::new(-1, 0, 0) }

    pub fn offset(self, face: crate::math::BlockFace) -> Self {
        self + face.direction()
    }

    pub fn distance_squared(self, other: Self) -> i64 {
        let dx = (self.x - other.x) as i64;
        let dy = (self.y - other.y) as i64;
        let dz = (self.z - other.z) as i64;
        dx * dx + dy * dy + dz * dz
    }
}

impl Add for BlockPos {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for BlockPos {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<i32> for BlockPos {
    type Output = Self;
    fn mul(self, s: i32) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s)
    }
}

impl Neg for BlockPos {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}
