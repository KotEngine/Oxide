use std::collections::{HashMap, HashSet, VecDeque};
use parking_lot::Mutex;

use crate::block::physics::is_transparent;
use crate::level::light::update_light_around;
use crate::level::World;

pub const DIRS_6: [(i32, i32, i32); 6] = [
    (0, -1, 0),
    (0, 1, 0),
    (0, 0, -1),
    (0, 0, 1),
    (-1, 0, 0),
    (1, 0, 0),
];

pub const HORIZ_4: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

#[derive(Debug, Default)]
pub struct RedstoneUpdateResult {
    pub block_changes: Vec<(i32, i32, i32, u8, u8)>,
    pub ignited_tnts: Vec<(i32, i32, i32, u32)>,
}

/// Schedules timed redstone releases (buttons and pressure plates).
#[derive(Default)]
pub struct RedstoneScheduler {
    tasks: Mutex<HashMap<(i32, i32, i32, i32), (u64, u8)>>, // (dim,x,y,z) -> (release_tick, expected_id)
}

impl RedstoneScheduler {
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(HashMap::new()),
        }
    }

    pub fn schedule_release(&self, dim: i32, x: i32, y: i32, z: i32, release_tick: u64, block_id: u8) {
        self.tasks.lock().insert((dim, x, y, z), (release_tick, block_id));
    }

    pub fn take_due(&self, current_tick: u64) -> Vec<(i32, i32, i32, i32, u8)> {
        let mut guard = self.tasks.lock();
        let due_keys: Vec<(i32, i32, i32, i32)> = guard
            .iter()
            .filter(|(_, (t, _))| *t <= current_tick)
            .map(|(k, _)| *k)
            .collect();
        let mut out = Vec::with_capacity(due_keys.len());
        for k in due_keys {
            if let Some((_, bid)) = guard.remove(&k) {
                out.push((k.0, k.1, k.2, k.3, bid));
            }
        }
        out
    }
}

pub fn attached_offset_for_lever(meta: u8) -> (i32, i32, i32) {
    match meta & 0x07 {
        0 | 7 => (0, 1, 0),  // ceiling
        1 => (-1, 0, 0),     // east-facing -> attached west
        2 => (1, 0, 0),      // west-facing -> attached east
        3 => (0, 0, -1),     // south-facing -> attached north
        4 => (0, 0, 1),      // north-facing -> attached south
        _ => (0, -1, 0),     // floor
    }
}

pub fn attached_offset_for_button(meta: u8) -> (i32, i32, i32) {
    match meta & 0x07 {
        0 => (0, 1, 0),
        1 => (0, -1, 0),
        2 => (0, 0, 1),
        3 => (0, 0, -1),
        4 => (1, 0, 0),
        5 => (-1, 0, 0),
        _ => (0, -1, 0),
    }
}

pub fn attached_offset_for_torch(meta: u8) -> (i32, i32, i32) {
    match meta & 0x07 {
        1 => (-1, 0, 0),
        2 => (1, 0, 0),
        3 => (0, 0, -1),
        4 => (0, 0, 1),
        _ => (0, -1, 0),
    }
}

fn repeater_back_offset(meta: u8) -> (i32, i32) {
    match meta & 0x03 {
        0 => (0, 1),   // faces north -> input from south
        1 => (-1, 0),  // faces east -> input from west
        2 => (0, -1),  // faces south -> input from north
        _ => (1, 0),   // faces west -> input from east
    }
}

fn repeater_front_offset(meta: u8) -> (i32, i32) {
    let (bx, bz) = repeater_back_offset(meta);
    (-bx, -bz)
}

/// Returns direct strong power emitted from block `(x, y, z)` toward neighbor `(tx, ty, tz)`.
pub fn direct_source_power(world: &World, x: i32, y: i32, z: i32, tx: i32, ty: i32, tz: i32) -> u8 {
    let id = world.get_block_id_opt(x, y, z);
    let meta = world.get_block_data_opt(x, y, z);
    let rel = (tx - x, ty - y, tz - z);

    match id {
        152 => 15, // Block of Redstone
        76 => {
            // Lit Redstone Torch does not power the block it attaches to
            if attached_offset_for_torch(meta) == rel {
                0
            } else {
                15
            }
        }
        69 if (meta & 0x08) != 0 => 15,
        77 | 143 if (meta & 0x08) != 0 => 15,
        70 | 72 | 147 | 148 if meta > 0 => 15,
        28 if (meta & 0x08) != 0 => 15,
        94 => {
            let (fx, fz) = repeater_front_offset(meta);
            if rel == (fx, 0, fz) { 15 } else { 0 }
        }
        _ => 0,
    }
}

/// Checks if a solid opaque block at `(x, y, z)` is strongly powered by any switch/torch/block/repeater.
pub fn is_block_strongly_powered(world: &World, x: i32, y: i32, z: i32) -> bool {
    let id = world.get_block_id_opt(x, y, z);
    if id == 0 || is_transparent(id) {
        return false;
    }
    for (dx, dy, dz) in DIRS_6 {
        let nx = x + dx;
        let ny = y + dy;
        let nz = z + dz;
        let nid = world.get_block_id_opt(nx, ny, nz);
        let nmeta = world.get_block_data_opt(nx, ny, nz);
        match nid {
            76 if (dx, dy, dz) == (0, -1, 0) => return true, // Lit torch under block
            69 if (nmeta & 0x08) != 0 && attached_offset_for_lever(nmeta) == (-dx, -dy, -dz) => return true,
            77 | 143 if (nmeta & 0x08) != 0 && attached_offset_for_button(nmeta) == (-dx, -dy, -dz) => return true,
            70 | 72 | 147 | 148 if nmeta > 0 && (dx, dy, dz) == (0, 1, 0) => return true,
            94 => {
                let (fx, fz) = repeater_front_offset(nmeta);
                if (fx, 0, fz) == (-dx, -dy, -dz) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// Returns effective redstone signal (0..=15) reaching block `(x, y, z)` from neighbors.
pub fn get_block_power(world: &World, x: i32, y: i32, z: i32) -> u8 {
    let mut max_p = 0u8;
    for (dx, dy, dz) in DIRS_6 {
        let nx = x + dx;
        let ny = y + dy;
        let nz = z + dz;
        if ny < 0 || ny >= 256 {
            continue;
        }
        let nid = world.get_block_id_opt(nx, ny, nz);
        let nmeta = world.get_block_data_opt(nx, ny, nz);

        let src = direct_source_power(world, nx, ny, nz, x, y, z);
        if src > max_p {
            max_p = src;
        }
        if nid == 55 && nmeta > 0 {
            // Wire directly adjacent (excluding below powering unless strong)
            if (dx, dy, dz) != (0, -1, 0) {
                max_p = max_p.max(nmeta);
            }
        }
        if is_block_strongly_powered(world, nx, ny, nz) {
            return 15;
        }
    }
    max_p
}

fn wire_incoming_strength(world: &World, x: i32, y: i32, z: i32) -> u8 {
    let mut strength = 0u8;

    for (dx, dy, dz) in DIRS_6 {
        let nx = x + dx;
        let ny = y + dy;
        let nz = z + dz;
        if ny < 0 || ny >= 256 {
            continue;
        }
        let src = direct_source_power(world, nx, ny, nz, x, y, z);
        if src > strength {
            strength = src;
        }
        if is_block_strongly_powered(world, nx, ny, nz) {
            return 15;
        }
    }

    let above_transparent = is_transparent(world.get_block_id_opt(x, y + 1, z));

    for (hx, hz) in HORIZ_4 {
        let nx = x + hx;
        let nz = z + hz;

        // Same level wire
        if world.get_block_id_opt(nx, y, nz) == 55 {
            let p = world.get_block_data_opt(nx, y, nz);
            strength = strength.max(p.saturating_sub(1));
        }

        // Step up (nx, y + 1, nz) if block above self is transparent
        if above_transparent && y + 1 < 256 && world.get_block_id_opt(nx, y + 1, nz) == 55 {
            let p = world.get_block_data_opt(nx, y + 1, nz);
            strength = strength.max(p.saturating_sub(1));
        }

        // Step down (nx, y - 1, nz) if side block (nx, y, nz) is transparent
        if y > 0 && is_transparent(world.get_block_id_opt(nx, y, nz)) && world.get_block_id_opt(nx, y - 1, nz) == 55 {
            let p = world.get_block_data_opt(nx, y - 1, nz);
            strength = strength.max(p.saturating_sub(1));
        }
    }

    strength
}

/// Propagates redstone power across wires, torches, repeaters, lamps, doors, rails, and TNT.
pub fn update_redstone_around(world: &mut World, origins: &[(i32, i32, i32)]) -> RedstoneUpdateResult {
    let mut out = RedstoneUpdateResult::default();
    let mut wire_queue = VecDeque::new();
    let mut queued = HashSet::new();
    let mut affected = HashSet::new();

    for &(ox, oy, oz) in origins {
        for dx in -2..=2 {
            for dy in -2..=2 {
                for dz in -2..=2 {
                    let pos = (ox + dx, oy + dy, oz + dz);
                    if pos.1 < 0 || pos.1 >= 256 {
                        continue;
                    }
                    affected.insert(pos);
                    if world.get_block_id_opt(pos.0, pos.1, pos.2) == 55 && queued.insert(pos) {
                        wire_queue.push_back(pos);
                    }
                }
            }
        }
    }

    // 1. Solve Redstone Wire 0..=15 network (bounded BFS)
    let mut iterations = 0usize;
    while let Some((wx, wy, wz)) = wire_queue.pop_front() {
        queued.remove(&(wx, wy, wz));
        iterations += 1;
        if iterations > 4096 {
            break;
        }
        if world.get_block_id_opt(wx, wy, wz) != 55 {
            continue;
        }
        let cur_meta = world.get_block_data_opt(wx, wy, wz);
        let new_power = wire_incoming_strength(world, wx, wy, wz).min(15);
        if new_power != cur_meta {
            world.set_block(wx, wy, wz, 55, new_power);
            out.block_changes.push((wx, wy, wz, 55, new_power));
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        let np = (wx + dx, wy + dy, wz + dz);
                        if np.1 < 0 || np.1 >= 256 {
                            continue;
                        }
                        affected.insert(np);
                        if world.get_block_id_opt(np.0, np.1, np.2) == 55 && queued.insert(np) {
                            wire_queue.push_back(np);
                        }
                    }
                }
            }
        }
    }

    // 2. Check Redstone Torches (75 <-> 76) and Repeaters (93 <-> 94) once and re-run wire pass if any flipped
    let mut flipped_sources = Vec::new();
    let snapshot_positions: Vec<(i32, i32, i32)> = affected.iter().copied().collect();
    for (x, y, z) in snapshot_positions {
        let id = world.get_block_id_opt(x, y, z);
        let meta = world.get_block_data_opt(x, y, z);
        if id == 75 || id == 76 {
            let (ax, ay, az) = attached_offset_for_torch(meta);
            let attach_powered = get_block_power(world, x + ax, y + ay, z + az) > 0;
            let want_id = if attach_powered { 75u8 } else { 76u8 };
            if want_id != id {
                world.set_block(x, y, z, want_id, meta);
                update_light_around(world, x, y, z);
                out.block_changes.push((x, y, z, want_id, meta));
                flipped_sources.push((x, y, z));
            }
        } else if id == 93 || id == 94 {
            let (bx, bz) = repeater_back_offset(meta);
            let in_x = x + bx;
            let in_z = z + bz;
            let in_id = world.get_block_id_opt(in_x, y, in_z);
            let in_meta = world.get_block_data_opt(in_x, y, in_z);
            let powered = (in_id == 55 && in_meta > 0)
                || direct_source_power(world, in_x, y, in_z, x, y, z) > 0
                || is_block_strongly_powered(world, in_x, y, in_z);
            let want_id = if powered { 94u8 } else { 93u8 };
            if want_id != id {
                world.set_block(x, y, z, want_id, meta);
                out.block_changes.push((x, y, z, want_id, meta));
                flipped_sources.push((x, y, z));
            }
        }
    }

    if !flipped_sources.is_empty() {
        for &(ox, oy, oz) in &flipped_sources {
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        let np = (ox + dx, oy + dy, oz + dz);
                        if np.1 >= 0 && np.1 < 256 {
                            affected.insert(np);
                            if world.get_block_id_opt(np.0, np.1, np.2) == 55 && queued.insert(np) {
                                wire_queue.push_back(np);
                            }
                        }
                    }
                }
            }
        }
        let mut iter2 = 0usize;
        while let Some((wx, wy, wz)) = wire_queue.pop_front() {
            queued.remove(&(wx, wy, wz));
            iter2 += 1;
            if iter2 > 2048 {
                break;
            }
            if world.get_block_id_opt(wx, wy, wz) != 55 {
                continue;
            }
            let cur_meta = world.get_block_data_opt(wx, wy, wz);
            let new_power = wire_incoming_strength(world, wx, wy, wz).min(15);
            if new_power != cur_meta {
                world.set_block(wx, wy, wz, 55, new_power);
                out.block_changes.push((wx, wy, wz, 55, new_power));
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        for dz in -1..=1 {
                            let np = (wx + dx, wy + dy, wz + dz);
                            if np.1 >= 0 && np.1 < 256 {
                                affected.insert(np);
                                if world.get_block_id_opt(np.0, np.1, np.2) == 55 && queued.insert(np) {
                                    wire_queue.push_back(np);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Apply Redstone power to consumers (Lamps, TNT, Doors, Trapdoors, Gates, Rails)
    let consumer_positions: Vec<(i32, i32, i32)> = affected.into_iter().collect();
    for (x, y, z) in consumer_positions {
        let id = world.get_block_id_opt(x, y, z);
        let meta = world.get_block_data_opt(x, y, z);
        match id {
            123 | 124 => {
                let powered = get_block_power(world, x, y, z) > 0;
                let target_id = if powered { 124u8 } else { 123u8 };
                if target_id != id {
                    world.set_block(x, y, z, target_id, meta);
                    update_light_around(world, x, y, z);
                    out.block_changes.push((x, y, z, target_id, meta));
                }
            }
            46 => {
                if get_block_power(world, x, y, z) > 0 {
                    world.set_block(x, y, z, 0, 0);
                    out.block_changes.push((x, y, z, 0, 0));
                    out.ignited_tnts.push((x, y, z, 80));
                }
            }
            64 | 71 | 193..=197 => {
                // Bottom half controls door open/close state
                let bottom_y = if (meta & 0x08) != 0 && y > 0 { y - 1 } else { y };
                if world.get_block_id_opt(x, bottom_y, z) == id {
                    let bmeta = world.get_block_data_opt(x, bottom_y, z);
                    let powered = get_block_power(world, x, bottom_y, z) > 0
                        || get_block_power(world, x, bottom_y + 1, z) > 0;
                    let want = if powered { bmeta | 0x04 } else { bmeta & !0x04 };
                    if want != bmeta {
                        world.set_block(x, bottom_y, z, id, want);
                        out.block_changes.push((x, bottom_y, z, id, want));
                    }
                }
            }
            96 | 167 => {
                let powered = get_block_power(world, x, y, z) > 0;
                let want = if powered { meta | 0x08 } else { meta & !0x08 };
                if want != meta {
                    world.set_block(x, y, z, id, want);
                    out.block_changes.push((x, y, z, id, want));
                }
            }
            107 | 183..=187 => {
                let powered = get_block_power(world, x, y, z) > 0;
                let want = if powered { meta | 0x04 } else { meta & !0x04 };
                if want != meta {
                    world.set_block(x, y, z, id, want);
                    out.block_changes.push((x, y, z, id, want));
                }
            }
            27 | 126 => {
                let powered = get_block_power(world, x, y, z) > 0;
                let want = if powered { meta | 0x08 } else { meta & 0x07 };
                if want != meta {
                    world.set_block(x, y, z, id, want);
                    out.block_changes.push((x, y, z, id, want));
                }
            }
            _ => {}
        }
    }

    out
}
