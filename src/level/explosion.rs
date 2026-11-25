use std::collections::HashSet;
use rand::Rng;
use crate::block::physics::{blast_resistance, cascade_neighbor_breaks, get_block_drops_ext};
use crate::level::light::update_light_around;
use crate::level::World;
use crate::protocol::types::Item;

pub struct ExplosionResult {
    pub destroyed_blocks: Vec<(i32, i32, i32, u8, u8)>,
    pub ignited_tnts: Vec<(f64, f64, f64, u32)>,
    pub drops: Vec<(i32, i32, i32, Item)>,
}

/// Raycast explosion using a 16x16x16 boundary shell and block blast resistances.
pub fn compute_explosion(world: &mut World, cx: f64, cy: f64, cz: f64, size: f32) -> ExplosionResult {
    if size < 0.1 {
        return ExplosionResult {
            destroyed_blocks: Vec::new(),
            ignited_tnts: Vec::new(),
            drops: Vec::new(),
        };
    }

    let rays: i32 = 16;
    let m_rays = rays - 1;
    let step_len = 0.3f32;
    let mut affected: HashSet<(i32, i32, i32)> = HashSet::new();

    {
        let mut rng = rand::thread_rng();
        for i in 0..rays {
            for j in 0..rays {
                for k in 0..rays {
                    if i == 0 || i == m_rays || j == 0 || j == m_rays || k == 0 || k == m_rays {
                        let vx = (i as f64 / m_rays as f64) * 2.0 - 1.0;
                        let vy = (j as f64 / m_rays as f64) * 2.0 - 1.0;
                        let vz = (k as f64 / m_rays as f64) * 2.0 - 1.0;
                        let len = (vx * vx + vy * vy + vz * vz).sqrt();
                        if len < 1e-6 {
                            continue;
                        }
                        let dx = (vx / len) * step_len as f64;
                        let dy = (vy / len) * step_len as f64;
                        let dz = (vz / len) * step_len as f64;

                        let mut px = cx;
                        let mut py = cy;
                        let mut pz = cz;
                        let mut blast_force = size * (rng.gen_range(700..=1300) as f32 / 1000.0);

                        while blast_force > 0.0 {
                            let bx = px.floor() as i32;
                            let by = py.floor() as i32;
                            let bz = pz.floor() as i32;

                            px += dx;
                            py += dy;
                            pz += dz;

                            if !(1..256).contains(&by) {
                                blast_force -= step_len * 0.75;
                                continue;
                            }

                            let bid = world.get_block_id_opt(bx, by, bz);
                            if bid != 0 {
                                blast_force -= (blast_resistance(bid) / 5.0 + 0.3) * step_len;
                                if blast_force > 0.0 {
                                    affected.insert((bx, by, bz));
                                }
                            }
                            blast_force -= step_len * 0.75;
                        }
                    }
                }
            }
        }
    }

    let yield_pct = (1.0 / size) * 100.0;
    let mut destroyed_blocks = Vec::new();
    let mut ignited_tnts = Vec::new();
    let mut drops = Vec::new();
    let mut broken_coords = Vec::new();

    {
        let mut rng = rand::thread_rng();
        for &(bx, by, bz) in &affected {
            let bid = world.get_block_id(bx, by, bz);
            if bid == 0 {
                continue;
            }
            let meta = {
                let ch = world.get_chunk_or_generate(bx >> 4, bz >> 4);
                ch.get_block_data((bx & 0x0f) as usize, by as usize, (bz & 0x0f) as usize)
            };

            if bid == 46 {
                let fuse = rng.gen_range(10..=30);
                ignited_tnts.push((bx as f64 + 0.5, by as f64, bz as f64 + 0.5, fuse));
            } else if (rng.gen_range(0..100) as f32) < yield_pct {
                for drop in get_block_drops_ext(bid, meta, &Item::air()) {
                    drops.push((bx, by, bz, drop));
                }
            }

            world.set_block(bx, by, bz, 0, 0);
            destroyed_blocks.push((bx, by, bz, 0, 0));
            broken_coords.push((bx, by, bz));
        }
    }

    if !broken_coords.is_empty() {
        let (cascaded, cascade_drops) = cascade_neighbor_breaks(world, &broken_coords);
        destroyed_blocks.extend(cascaded);
        drops.extend(cascade_drops);
        let center_bx = cx.floor() as i32;
        let center_by = (cy.floor() as i32).clamp(1, 255);
        let center_bz = cz.floor() as i32;
        update_light_around(world, center_bx, center_by, center_bz);
    }

    ExplosionResult {
        destroyed_blocks,
        ignited_tnts,
        drops,
    }
}

/// Calculates explosion damage and directional push velocity for an entity at `(ex, ey, ez)`.
pub fn explosion_damage_and_knockback(
    ex: f64,
    ey: f64,
    ez: f64,
    cx: f64,
    cy: f64,
    cz: f64,
    size: f32,
) -> Option<(f32, f32, f32, f32)> {
    let explosion_size = size as f64 * 2.0;
    let dx = ex - cx;
    let dy = ey - cy;
    let dz = ez - cz;
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    let ratio = dist / explosion_size;
    if ratio > 1.0 {
        return None;
    }
    let impact = 1.0 - ratio;
    let damage = (((impact * impact + impact) / 2.0) * 8.0 * explosion_size + 1.0) as f32;
    let inv = if dist > 1e-4 { 1.0 / dist } else { 1.0 };
    let vx = (dx * inv * impact) as f32;
    let vy = ((dy * inv + 0.35) * impact) as f32;
    let vz = (dz * inv * impact) as f32;
    Some((damage, vx, vy, vz))
}
