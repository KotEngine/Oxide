use std::collections::{HashSet, VecDeque};
use crate::block::physics::{light_filter, light_level};
use crate::level::World;

fn get_block_light_loaded(world: &World, x: i32, y: i32, z: i32) -> u8 {
    if !(0..256).contains(&y) {
        return 0;
    }
    world
        .get_chunk(x >> 4, z >> 4)
        .map(|c| c.get_block_light((x & 0x0f) as usize, y as usize, (z & 0x0f) as usize))
        .unwrap_or(0)
}

fn set_block_light_loaded(world: &mut World, x: i32, y: i32, z: i32, level: u8) -> bool {
    if !(0..256).contains(&y) {
        return false;
    }
    if let Some(c) = world.chunks.get_mut(&(x >> 4, z >> 4)) {
        c.set_block_light((x & 0x0f) as usize, y as usize, (z & 0x0f) as usize, level);
        true
    } else {
        false
    }
}

fn get_sky_light_loaded(world: &World, x: i32, y: i32, z: i32) -> u8 {
    if y >= 256 {
        return 15;
    }
    if y < 0 {
        return 0;
    }
    world
        .get_chunk(x >> 4, z >> 4)
        .map(|c| c.get_sky_light((x & 0x0f) as usize, y as usize, (z & 0x0f) as usize))
        .unwrap_or(15)
}

fn set_sky_light_loaded(world: &mut World, x: i32, y: i32, z: i32, level: u8) -> bool {
    if !(0..256).contains(&y) {
        return false;
    }
    if let Some(c) = world.chunks.get_mut(&(x >> 4, z >> 4)) {
        c.set_sky_light((x & 0x0f) as usize, y as usize, (z & 0x0f) as usize, level);
        true
    } else {
        false
    }
}

/// Updates both block light and sky light around a modified block `(x, y, z)` using BFS.
pub fn update_light_around(world: &mut World, x: i32, y: i32, z: i32) {
    if !(0..256).contains(&y) {
        return;
    }

    // 1. Update heightmap column and vertical skylight
    let cx = x >> 4;
    let cz = z >> 4;
    let lx = (x & 0x0f) as usize;
    let lz = (z & 0x0f) as usize;
    let mut sky_updates = Vec::new();
    if let Some(chunk) = world.chunks.get_mut(&(cx, cz)) {
        let mut top = 0u16;
        for yy in (0..256).rev() {
            if light_filter(chunk.get_block_id(lx, yy, lz)) > 1 {
                top = (yy + 1) as u16;
                break;
            }
        }
        let old_top = chunk.height_map[(lz << 4) | lx] as i32;
        chunk.height_map[(lz << 4) | lx] = top;

        let min_y = (top as i32).min(old_top).max(0);
        let max_y = (top as i32).max(old_top).min(255);
        for yy in min_y..=max_y {
            let new_sky = if yy >= top as i32 { 15 } else { 0 };
            sky_updates.push((x, yy, z, new_sky));
        }
    }
    sky_updates.push((x, y, z, if y >= world.chunks.get(&(cx, cz)).map(|c| c.height_map[(lz << 4) | lx] as i32).unwrap_or(0) { 15 } else { 0 }));

    run_light_bfs(world, &sky_updates, true);

    // 2. Block light at `(x, y, z)` and neighbor seeds
    let id = world.get_block_id_opt(x, y, z);
    let emit = light_level(id);
    let mut block_seeds = vec![(x, y, z, emit)];
    for (nx, ny, nz) in [
        (x + 1, y, z), (x - 1, y, z),
        (x, y + 1, z), (x, y - 1, z),
        (x, y, z + 1), (x, y, z - 1),
    ] {
        if (0..256).contains(&ny) {
            let nl = get_block_light_loaded(world, nx, ny, nz);
            if nl > 0 {
                block_seeds.push((nx, ny, nz, nl));
            }
        }
    }
    run_light_bfs(world, &block_seeds, false);
}

fn run_light_bfs(world: &mut World, initial_nodes: &[(i32, i32, i32, u8)], is_sky: bool) {
    let mut removal_queue: VecDeque<(i32, i32, i32, u8)> = VecDeque::new();
    let mut spread_queue: VecDeque<(i32, i32, i32)> = VecDeque::new();
    let mut removal_visited: HashSet<(i32, i32, i32)> = HashSet::new();
    let mut spread_visited: HashSet<(i32, i32, i32)> = HashSet::new();

    for &(nx, ny, nz, new_level) in initial_nodes {
        if !(0..256).contains(&ny) || world.get_chunk(nx >> 4, nz >> 4).is_none() {
            continue;
        }
        let old_level = if is_sky {
            get_sky_light_loaded(world, nx, ny, nz)
        } else {
            get_block_light_loaded(world, nx, ny, nz)
        };
        if old_level != new_level {
            if is_sky {
                set_sky_light_loaded(world, nx, ny, nz, new_level);
            } else {
                set_block_light_loaded(world, nx, ny, nz, new_level);
            }
            if old_level < new_level {
                if spread_visited.insert((nx, ny, nz)) {
                    spread_queue.push_back((nx, ny, nz));
                }
            } else if removal_visited.insert((nx, ny, nz)) {
                removal_queue.push_back((nx, ny, nz, old_level));
            }
        } else if new_level > 0 && spread_visited.insert((nx, ny, nz)) {
            spread_queue.push_back((nx, ny, nz));
        }
    }

    let mut steps = 0usize;
    while let Some((x, y, z, old_adj)) = removal_queue.pop_front() {
        steps += 1;
        if steps > 4096 {
            break;
        }
        for (cx, cy, cz) in [
            (x + 1, y, z), (x - 1, y, z),
            (x, y + 1, z), (x, y - 1, z),
            (x, y, z + 1), (x, y, z - 1),
        ] {
            if !(0..256).contains(&cy) || world.get_chunk(cx >> 4, cz >> 4).is_none() {
                continue;
            }
            let current = if is_sky {
                get_sky_light_loaded(world, cx, cy, cz)
            } else {
                get_block_light_loaded(world, cx, cy, cz)
            };
            if current != 0 && current < old_adj {
                // Keep intrinsic block emitter intact while removing stale propagated light
                let intrinsic = if is_sky {
                    0
                } else {
                    light_level(world.get_block_id_opt(cx, cy, cz))
                };
                if is_sky {
                    set_sky_light_loaded(world, cx, cy, cz, 0);
                } else {
                    set_block_light_loaded(world, cx, cy, cz, intrinsic);
                }
                if intrinsic > 0 && spread_visited.insert((cx, cy, cz)) {
                    spread_queue.push_back((cx, cy, cz));
                }
                if removal_visited.insert((cx, cy, cz)) && current > 1 {
                    removal_queue.push_back((cx, cy, cz, current));
                }
            } else if current >= old_adj && spread_visited.insert((cx, cy, cz)) {
                spread_queue.push_back((cx, cy, cz));
            }
        }
    }

    steps = 0;
    while let Some((x, y, z)) = spread_queue.pop_front() {
        spread_visited.remove(&(x, y, z));
        steps += 1;
        if steps > 8192 {
            break;
        }
        let adj_light = if is_sky {
            get_sky_light_loaded(world, x, y, z)
        } else {
            get_block_light_loaded(world, x, y, z)
        };
        if adj_light <= 1 {
            continue;
        }
        for (cx, cy, cz) in [
            (x + 1, y, z), (x - 1, y, z),
            (x, y + 1, z), (x, y - 1, z),
            (x, y, z + 1), (x, y, z - 1),
        ] {
            if !(0..256).contains(&cy) || world.get_chunk(cx >> 4, cz >> 4).is_none() {
                continue;
            }
            let bid = world.get_block_id_opt(cx, cy, cz);
            let filter = light_filter(bid);
            let potential = adj_light.saturating_sub(filter);
            let current = if is_sky {
                get_sky_light_loaded(world, cx, cy, cz)
            } else {
                get_block_light_loaded(world, cx, cy, cz)
            };
            if current < potential {
                if is_sky {
                    set_sky_light_loaded(world, cx, cy, cz, potential);
                } else {
                    set_block_light_loaded(world, cx, cy, cz, potential);
                }
                if potential > 1 && spread_visited.insert((cx, cy, cz)) {
                    spread_queue.push_back((cx, cy, cz));
                }
            }
        }
    }
}
