use super::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self { Self { min, max } }

    pub fn from_corners(x1: f64, y1: f64, z1: f64, x2: f64, y2: f64, z2: f64) -> Self {
        Self {
            min: Vec3::new(x1.min(x2), y1.min(y2), z1.min(z2)),
            max: Vec3::new(x1.max(x2), y1.max(y2), z1.max(z2)),
        }
    }

    pub fn offset(self, dx: f64, dy: f64, dz: f64) -> Self {
        Self {
            min: Vec3::new(self.min.x + dx, self.min.y + dy, self.min.z + dz),
            max: Vec3::new(self.max.x + dx, self.max.y + dy, self.max.z + dz),
        }
    }

    pub fn expand(self, dx: f64, dy: f64, dz: f64) -> Self {
        Self {
            min: Vec3::new(self.min.x - dx.abs(), self.min.y - dy.abs(), self.min.z - dz.abs()),
            max: Vec3::new(self.max.x + dx.abs(), self.max.y + dy.abs(), self.max.z + dz.abs()),
        }
    }

    pub fn grow(self, v: f64) -> Self {
        self.expand(v, v, v)
    }

    pub fn intersects(self, other: Self) -> bool {
        self.max.x > other.min.x && self.min.x < other.max.x
            && self.max.y > other.min.y && self.min.y < other.max.y
            && self.max.z > other.min.z && self.min.z < other.max.z
    }

    pub fn intersect(self, other: Self) -> Option<Self> {
        if !self.intersects(other) { return None; }
        Some(Self::new(
            Vec3::new(
                self.min.x.max(other.min.x),
                self.min.y.max(other.min.y),
                self.min.z.max(other.min.z),
            ),
            Vec3::new(
                self.max.x.min(other.max.x),
                self.max.y.min(other.max.y),
                self.max.z.min(other.max.z),
            ),
        ))
    }

    pub fn calculate_x_offset(self, other: Self, dx: f64) -> f64 {
        if !self.intersects(other.offset(dx, 0.0, 0.0)) { return dx; }
        if dx > 0.0 { (other.min.x - self.max.x).min(dx) } else { (other.max.x - self.min.x).max(dx) }
    }

    pub fn calculate_y_offset(self, other: Self, dy: f64) -> f64 {
        if !self.intersects(other.offset(0.0, dy, 0.0)) { return dy; }
        if dy > 0.0 { (other.min.y - self.max.y).min(dy) } else { (other.max.y - self.min.y).max(dy) }
    }

    pub fn calculate_z_offset(self, other: Self, dz: f64) -> f64 {
        if !self.intersects(other.offset(0.0, 0.0, dz)) { return dz; }
        if dz > 0.0 { (other.min.z - self.max.z).min(dz) } else { (other.max.z - self.min.z).max(dz) }
    }

    pub fn contains_point(self, point: Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x
            && point.y >= self.min.y && point.y <= self.max.y
            && point.z >= self.min.z && point.z <= self.max.z
    }
}
