use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub struct PerlinNoise {
    perm: [u8; 512],
}

impl PerlinNoise {
    pub fn new(seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut p = [0u8; 256];
        for i in 0..256 { p[i] = i as u8; }
        for i in (1..256).rev() {
            let j = rng.gen_range(0..=i);
            p.swap(i, j);
        }
        let mut perm = [0u8; 512];
        for i in 0..512 { perm[i] = p[i & 255]; }
        Self { perm }
    }

    fn fade(t: f64) -> f64 { t * t * t * (t * (t * 6.0 - 15.0) + 10.0) }
    fn lerp(a: f64, b: f64, t: f64) -> f64 { a + t * (b - a) }

    fn grad(hash: u8, x: f64, y: f64, z: f64) -> f64 {
        let h = hash & 15;
        let u = if h < 8 { x } else { y };
        let v = if h < 4 { y } else if h == 12 || h == 14 { x } else { z };
        (if h & 1 == 0 { u } else { -u }) + (if h & 2 == 0 { v } else { -v })
    }

    pub fn noise3d(&self, x: f64, y: f64, z: f64) -> f64 {
        let xi = (x.floor() as i64 & 255) as usize;
        let yi = (y.floor() as i64 & 255) as usize;
        let zi = (z.floor() as i64 & 255) as usize;
        let xf = x - x.floor();
        let yf = y - y.floor();
        let zf = z - z.floor();
        let u = Self::fade(xf);
        let v = Self::fade(yf);
        let w = Self::fade(zf);
        let aaa = self.perm[self.perm[self.perm[xi] as usize + yi] as usize + zi];
        let aba = self.perm[self.perm[self.perm[xi] as usize + yi + 1] as usize + zi];
        let aab = self.perm[self.perm[self.perm[xi] as usize + yi] as usize + zi + 1];
        let abb = self.perm[self.perm[self.perm[xi] as usize + yi + 1] as usize + zi + 1];
        let baa = self.perm[self.perm[self.perm[xi + 1] as usize + yi] as usize + zi];
        let bba = self.perm[self.perm[self.perm[xi + 1] as usize + yi + 1] as usize + zi];
        let bab = self.perm[self.perm[self.perm[xi + 1] as usize + yi] as usize + zi + 1];
        let bbb = self.perm[self.perm[self.perm[xi + 1] as usize + yi + 1] as usize + zi + 1];
        let x1 = Self::lerp(Self::grad(aaa, xf, yf, zf), Self::grad(baa, xf - 1.0, yf, zf), u);
        let x2 = Self::lerp(Self::grad(aba, xf, yf - 1.0, zf), Self::grad(bba, xf - 1.0, yf - 1.0, zf), u);
        let y1 = Self::lerp(x1, x2, v);
        let x3 = Self::lerp(Self::grad(aab, xf, yf, zf - 1.0), Self::grad(bab, xf - 1.0, yf, zf - 1.0), u);
        let x4 = Self::lerp(Self::grad(abb, xf, yf - 1.0, zf - 1.0), Self::grad(bbb, xf - 1.0, yf - 1.0, zf - 1.0), u);
        let y2 = Self::lerp(x3, x4, v);
        Self::lerp(y1, y2, w)
    }

    pub fn noise2d(&self, x: f64, z: f64) -> f64 {
        self.noise3d(x, 0.0, z)
    }
}
