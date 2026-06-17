#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xff) as u8,
            g: ((hex >> 8) & 0xff) as u8,
            b: (hex & 0xff) as u8,
            a: 255,
        }
    }

    pub fn to_argb(self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | self.b as u32
    }

    pub fn mix(self, other: Self, t: f32) -> Self {
        let r = (self.r as f32 * (1.0 - t) + other.r as f32 * t) as u8;
        let g = (self.g as f32 * (1.0 - t) + other.g as f32 * t) as u8;
        let b = (self.b as f32 * (1.0 - t) + other.b as f32 * t) as u8;
        Self::rgb(r, g, b)
    }
}
