use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

pub struct SimplexNoise {
    perm: [u8; 512],
    grad3: [[f64; 3]; 12],
}

impl SimplexNoise {
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
        Self {
            perm,
            grad3: [
                [1.0, 1.0, 0.0], [-1.0, 1.0, 0.0], [1.0, -1.0, 0.0], [-1.0, -1.0, 0.0],
                [1.0, 0.0, 1.0], [-1.0, 0.0, 1.0], [1.0, 0.0, -1.0], [-1.0, 0.0, -1.0],
                [0.0, 1.0, 1.0], [0.0, -1.0, 1.0], [0.0, 1.0, -1.0], [0.0, -1.0, -1.0],
            ],
        }
    }

    pub fn noise2d(&self, xin: f64, yin: f64) -> f64 {
        let f2 = 0.5 * (3.0_f64.sqrt() - 1.0);
        let s = (xin + yin) * f2;
        let i = (xin + s).floor() as i32;
        let j = (yin + s).floor() as i32;
        let g2 = (3.0 - 3.0_f64.sqrt()) / 6.0;
        let t = (i + j) as f64 * g2;
        let x0 = xin - (i as f64 - t);
        let y0 = yin - (j as f64 - t);
        let (i1, j1) = if x0 > y0 { (1, 0) } else { (0, 1) };
        let x1 = x0 - i1 as f64 + g2;
        let y1 = y0 - j1 as f64 + g2;
        let x2 = x0 - 1.0 + 2.0 * g2;
        let y2 = y0 - 1.0 + 2.0 * g2;
        let ii = i & 255;
        let jj = j & 255;
        let gi0 = self.perm[(ii as usize + self.perm[jj as usize] as usize) as usize] % 12;
        let gi1 = self.perm[((ii + i1) as usize + self.perm[(jj + j1) as usize] as usize) as usize] % 12;
        let gi2 = self.perm[((ii + 1) as usize + self.perm[(jj + 1) as usize] as usize) as usize] % 12;
        let n0 = {
            let t0 = 0.5 - x0 * x0 - y0 * y0;
            if t0 < 0.0 { 0.0 } else { t0 * t0 * t0 * t0 * Self::dot2(self.grad3[gi0 as usize], x0, y0) }
        };
        let n1 = {
            let t1 = 0.5 - x1 * x1 - y1 * y1;
            if t1 < 0.0 { 0.0 } else { t1 * t1 * t1 * t1 * Self::dot2(self.grad3[gi1 as usize], x1, y1) }
        };
        let n2 = {
            let t2 = 0.5 - x2 * x2 - y2 * y2;
            if t2 < 0.0 { 0.0 } else { t2 * t2 * t2 * t2 * Self::dot2(self.grad3[gi2 as usize], x2, y2) }
        };
        70.0 * (n0 + n1 + n2)
    }

    fn dot2(g: [f64; 3], x: f64, y: f64) -> f64 { g[0] * x + g[1] * y }
}
