use super::perlin::PerlinNoise;

pub struct OctaveNoise {
    octaves: Vec<PerlinNoise>,
}

impl OctaveNoise {
    pub fn new(seed: u64, octave_count: usize) -> Self {
        let mut octaves = Vec::with_capacity(octave_count);
        for i in 0..octave_count {
            octaves.push(PerlinNoise::new(seed.wrapping_add(i as u64)));
        }
        Self { octaves }
    }

    pub fn noise2d(&self, x: f64, z: f64) -> f64 {
        let mut result = 0.0;
        let mut amp = 1.0;
        let mut freq = 1.0;
        let mut max = 0.0;
        for octave in &self.octaves {
            result += octave.noise2d(x * freq, z * freq) * amp;
            max += amp;
            amp *= 0.5;
            freq *= 2.0;
        }
        result / max
    }

    pub fn noise3d(&self, x: f64, y: f64, z: f64) -> f64 {
        let mut result = 0.0;
        let mut amp = 1.0;
        let mut freq = 1.0;
        let mut max = 0.0;
        for octave in &self.octaves {
            result += octave.noise3d(x * freq, y * freq, z * freq) * amp;
            max += amp;
            amp *= 0.5;
            freq *= 2.0;
        }
        result / max
    }
}
