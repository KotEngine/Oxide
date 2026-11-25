use rand::Rng;
use crate::block::physics::{is_transparent, light_filter};
use crate::level::World;

/// Processes random block ticks around active player chunk coordinates.
/// Returns list of modified blocks `(x, y, z, new_id, new_meta)` to broadcast.
pub fn tick_random_blocks(
    world: &mut World,
    dim: i32,
    is_raining: bool,
    player_chunks: &[(i32, i32)],
) -> Vec<(i32, i32, i32, u8, u8)> {
    if player_chunks.is_empty() {
        return Vec::new();
    }

    let mut visited_chunks = std::collections::HashSet::new();
    let mut updates: Vec<(i32, i32, i32, u8, u8)> = Vec::new();
    let mut rng = rand::thread_rng();

    // Select loaded chunks within radius 3 of any player
    for &(pcx, pcz) in player_chunks {
        for dx in -3..=3 {
            for dz in -3..=3 {
                visited_chunks.insert((pcx + dx, pcz + dz));
            }
        }
    }

    let mut samples: Vec<(i32, i32, i32, u8, u8, u8)> = Vec::new();
    for (cx, cz) in visited_chunks {
        let Some(chunk) = world.get_chunk(cx, cz) else { continue; };
        for (sy_idx, sc) in chunk.sub_chunks.iter().enumerate() {
            if sc.is_empty() {
                continue;
            }
            for _ in 0..3 {
                let lx = rng.gen_range(0..16);
                let ly = rng.gen_range(0..16);
                let lz = rng.gen_range(0..16);
                let id = sc.get_block_id(lx, ly, lz);
                if matches!(id, 2 | 6 | 18 | 51 | 59 | 60 | 79 | 83 | 110 | 115 | 141 | 142 | 161 | 244) {
                    let meta = sc.get_block_data(lx, ly, lz);
                    let wx = (cx << 4) + lx as i32;
                    let wy = ((sy_idx << 4) + ly) as i32;
                    let wz = (cz << 4) + lz as i32;
                    let bl = sc.get_block_light(lx, ly, lz);
                    samples.push((wx, wy, wz, id, meta, bl));
                }
            }
        }
    }

    for (x, y, z, id, meta, block_light) in samples {
        match id {
            // Grass (2) spread and decay
            2 => {
                if y + 1 < 256 {
                    let above_id = world.get_block_id_opt(x, y + 1, z);
                    let light_above = world.get_full_light_opt(x, y + 1, z);
                    if light_above < 4 && light_filter(above_id) >= 3 {
                        world.set_block(x, y, z, 3, 0);
                        updates.push((x, y, z, 3, 0));
                    } else if light_above >= 9 {
                        for _ in 0..4 {
                            let tx = x + rng.gen_range(-1..=1);
                            let ty = (y + rng.gen_range(-3..=1)).clamp(1, 254);
                            let tz = z + rng.gen_range(-1..=1);
                            if world.get_block_id_opt(tx, ty, tz) == 3
                                && world.get_block_data_opt(tx, ty, tz) != 1
                                && world.get_full_light_opt(tx, ty + 1, tz) >= 4
                                && light_filter(world.get_block_id_opt(tx, ty + 1, tz)) < 3
                            {
                                world.set_block(tx, ty, tz, 2, 0);
                                updates.push((tx, ty, tz, 2, 0));
                            }
                        }
                    }
                }
            }
            // Mycelium (110) spread
            110 => {
                if y + 1 < 256 {
                    let light_above = world.get_full_light_opt(x, y + 1, z);
                    if light_above >= 9 {
                        for _ in 0..4 {
                            let tx = x + rng.gen_range(-1..=1);
                            let ty = (y + rng.gen_range(-3..=1)).clamp(1, 254);
                            let tz = z + rng.gen_range(-1..=1);
                            if world.get_block_id_opt(tx, ty, tz) == 3
                                && world.get_block_id_opt(tx, ty + 1, tz) == 0
                            {
                                world.set_block(tx, ty, tz, 110, 0);
                                updates.push((tx, ty, tz, 110, 0));
                            }
                        }
                    }
                }
            }
            // Farmland (60) hydration
            60 => {
                let mut hydrated = false;
                'water_scan: for wy in y..=(y + 1).min(255) {
                    for wz in (z - 4)..=(z + 4) {
                        for wx in (x - 4)..=(x + 4) {
                            let wid = world.get_block_id_opt(wx, wy, wz);
                            if wid == 8 || wid == 9 {
                                hydrated = true;
                                break 'water_scan;
                            }
                        }
                    }
                }
                if hydrated {
                    if meta < 7 {
                        world.set_block(x, y, z, 60, 7);
                        updates.push((x, y, z, 60, 7));
                    }
                } else if meta > 0 {
                    world.set_block(x, y, z, 60, meta - 1);
                    updates.push((x, y, z, 60, meta - 1));
                } else if world.get_block_id_opt(x, y + 1, z) == 0 {
                    world.set_block(x, y, z, 3, 0);
                    updates.push((x, y, z, 3, 0));
                }
            }
            // Crops (Wheat 59, Carrot 141, Potato 142, Beetroot 244, Nether Wart 115)
            59 | 141 | 142 => {
                if meta < 7 && rng.gen_range(0..3) == 1 {
                    world.set_block(x, y, z, id, meta + 1);
                    updates.push((x, y, z, id, meta + 1));
                }
            }
            244 => {
                if meta < 4 && rng.gen_range(0..3) == 1 {
                    world.set_block(x, y, z, 244, meta + 1);
                    updates.push((x, y, z, 244, meta + 1));
                }
            }
            115 => {
                if meta < 3 && rng.gen_range(0..10) == 0 {
                    world.set_block(x, y, z, 115, meta + 1);
                    updates.push((x, y, z, 115, meta + 1));
                }
            }
            // Sapling (6) growth
            6 => {
                if rng.gen_range(0..7) == 0 {
                    if (meta & 0x08) == 0 {
                        world.set_block(x, y, z, 6, meta | 0x08);
                        updates.push((x, y, z, 6, meta | 0x08));
                    } else {
                        let tree_changes = grow_simple_tree(world, x, y, z, meta & 0x03);
                        updates.extend(tree_changes);
                    }
                }
            }
            // Leaves (18, 161) decay if bit 0x04 is clear and no log nearby
            18 | 161 => {
                if (meta & 0x04) == 0 && !has_log_nearby(world, x, y, z, 4) {
                    world.set_block(x, y, z, 0, 0);
                    updates.push((x, y, z, 0, 0));
                }
            }
            // Fire (51) decay / rain extinguish
            51 => {
                if is_raining && dim == 0 {
                    world.set_block(x, y, z, 0, 0);
                    updates.push((x, y, z, 0, 0));
                } else {
                    let down = world.get_block_id_opt(x, y - 1, z);
                    if down != 87 && down != 213 && rng.gen_range(0..3) == 0 {
                        if meta < 15 {
                            world.set_block(x, y, z, 51, meta + 1);
                            updates.push((x, y, z, 51, meta + 1));
                        } else {
                            world.set_block(x, y, z, 0, 0);
                            updates.push((x, y, z, 0, 0));
                        }
                    }
                }
            }
            // Ice (79) melts in high block light
            79 => {
                if block_light > 11 && dim != 1 {
                    world.set_block(x, y, z, 8, 0);
                    updates.push((x, y, z, 8, 0));
                }
            }
            _ => {}
        }
    }

    updates
}

fn has_log_nearby(world: &World, ox: i32, oy: i32, oz: i32, radius: i32) -> bool {
    for dy in -radius..=radius {
        let yy = oy + dy;
        if !(0..256).contains(&yy) {
            continue;
        }
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                if dx.abs() + dy.abs() + dz.abs() > radius + 1 {
                    continue;
                }
                let id = world.get_block_id_opt(ox + dx, yy, oz + dz);
                if id == 17 || id == 162 {
                    return true;
                }
            }
        }
    }
    false
}

/// Grows a basic oak/spruce/birch/jungle tree from sapling `(x, y, z)` and returns modified blocks.
pub fn grow_simple_tree(world: &mut World, x: i32, y: i32, z: i32, tree_type: u8) -> Vec<(i32, i32, i32, u8, u8)> {
    let mut changes = Vec::new();
    let height = 5;
    if y < 1 || y + height + 2 >= 256 {
        return changes;
    }

    // Check trunk headroom
    for dy in 1..=(height + 1) {
        let bid = world.get_block_id_opt(x, y + dy, z);
        if bid != 0 && bid != 18 && bid != 161 {
            return changes;
        }
    }

    let wood_meta = tree_type & 0x03;
    // Place leaves canopy first where replaceable
    for ly in (y + height - 3)..=(y + height) {
        let y_off = ly - (y + height);
        let rad = 1 - y_off / 2;
        for lx in (x - rad)..=(x + rad) {
            for lz in (z - rad)..=(z + rad) {
                let d = (lx - x).abs() + (lz - z).abs();
                if (lx - x).abs() == rad && (lz - z).abs() == rad && (y_off == 0 || d > 2) {
                    continue;
                }
                let existing = world.get_block_id_opt(lx, ly, lz);
                if existing == 0 || is_transparent(existing) && existing != 17 && existing != 162 {
                    world.set_block(lx, ly, lz, 18, wood_meta);
                    changes.push((lx, ly, lz, 18, wood_meta));
                }
            }
        }
    }

    // Place log trunk
    for dy in 0..height {
        world.set_block(x, y + dy, z, 17, wood_meta);
        changes.push((x, y + dy, z, 17, wood_meta));
    }

    changes
}
