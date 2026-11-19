use crate::level::chunk::Chunk;
use crate::level::generator::noise::OctaveNoise;

pub struct CavePopulator {
    noise: OctaveNoise,
}

impl CavePopulator {
    pub fn new(seed: u64) -> Self {
        Self {
            noise: OctaveNoise::new(seed.wrapping_mul(41), 3),
        }
    }

    pub fn populate(&self, chunk: &mut Chunk) {
        let base_x = (chunk.x * 16) as f64;
        let base_z = (chunk.z * 16) as f64;

        for x in 0..16 {
            for z in 0..16 {
                let wx = base_x + x as f64;
                let wz = base_z + z as f64;

                for y in 8..55usize {
                    let n = self.noise.noise3d(wx * 0.04, y as f64 * 0.06, wz * 0.04);
                    let n2 = self.noise.noise3d((wx + 50.0) * 0.03, (y as f64 + 20.0) * 0.05, (wz + 50.0) * 0.03);

                    if n > 0.52 && n2.abs() < 0.28 {
                        let cur_id = chunk.get_block_id(x, y, z);
                        if cur_id == 1 || cur_id == 3 || cur_id == 13 || cur_id == 24 {
                            if y <= 10 {
                                chunk.set_block(x, y, z, 11, 0);
                            } else {
                                chunk.set_block(x, y, z, 0, 0);
                            }
                        }
                    }
                }
            }
        }
    }
}
