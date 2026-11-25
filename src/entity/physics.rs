use crate::block::physics::is_solid;
use crate::level::World;

#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

impl Aabb {
    pub fn new(min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> Self {
        Self { min_x, min_y, min_z, max_x, max_y, max_z }
    }

    pub fn from_entity(x: f64, y: f64, z: f64, half_w: f64, height: f64) -> Self {
        Self {
            min_x: x - half_w,
            min_y: y,
            min_z: z - half_w,
            max_x: x + half_w,
            max_y: y + height,
            max_z: z + half_w,
        }
    }

    pub fn offset(self, dx: f64, dy: f64, dz: f64) -> Self {
        Self {
            min_x: self.min_x + dx,
            min_y: self.min_y + dy,
            min_z: self.min_z + dz,
            max_x: self.max_x + dx,
            max_y: self.max_y + dy,
            max_z: self.max_z + dz,
        }
    }

    pub fn expand_motion(self, dx: f64, dy: f64, dz: f64) -> Self {
        Self {
            min_x: if dx < 0.0 { self.min_x + dx } else { self.min_x },
            min_y: if dy < 0.0 { self.min_y + dy } else { self.min_y },
            min_z: if dz < 0.0 { self.min_z + dz } else { self.min_z },
            max_x: if dx > 0.0 { self.max_x + dx } else { self.max_x },
            max_y: if dy > 0.0 { self.max_y + dy } else { self.max_y },
            max_z: if dz > 0.0 { self.max_z + dz } else { self.max_z },
        }
    }

    pub fn calculate_y_offset(&self, other: &Aabb, mut dy: f64) -> f64 {
        if other.max_x <= self.min_x || other.min_x >= self.max_x {
            return dy;
        }
        if other.max_z <= self.min_z || other.min_z >= self.max_z {
            return dy;
        }
        if dy > 0.0 && other.max_y <= self.min_y {
            let diff = self.min_y - other.max_y;
            if diff < dy {
                dy = diff;
            }
        } else if dy < 0.0 && other.min_y >= self.max_y {
            let diff = self.max_y - other.min_y;
            if diff > dy {
                dy = diff;
            }
        }
        dy
    }

    pub fn calculate_x_offset(&self, other: &Aabb, mut dx: f64) -> f64 {
        if other.max_y <= self.min_y || other.min_y >= self.max_y {
            return dx;
        }
        if other.max_z <= self.min_z || other.min_z >= self.max_z {
            return dx;
        }
        if dx > 0.0 && other.max_x <= self.min_x {
            let diff = self.min_x - other.max_x;
            if diff < dx {
                dx = diff;
            }
        } else if dx < 0.0 && other.min_x >= self.max_x {
            let diff = self.max_x - other.min_x;
            if diff > dx {
                dx = diff;
            }
        }
        dx
    }

    pub fn calculate_z_offset(&self, other: &Aabb, mut dz: f64) -> f64 {
        if other.max_x <= self.min_x || other.min_x >= self.max_x {
            return dz;
        }
        if other.max_y <= self.min_y || other.min_y >= self.max_y {
            return dz;
        }
        if dz > 0.0 && other.max_z <= self.min_z {
            let diff = self.min_z - other.max_z;
            if diff < dz {
                dz = diff;
            }
        } else if dz < 0.0 && other.min_z >= self.max_z {
            let diff = self.max_z - other.min_z;
            if diff > dz {
                dz = diff;
            }
        }
        dz
    }
}

/// Returns `(half_width, height)` for an entity network ID.
pub fn entity_half_size(entity_type: i32) -> (f64, f64) {
    match entity_type {
        64 | 69 | 68 | 80 | 81 | 82 | 86 | 87 => (0.125, 0.25),
        10 => (0.2, 0.7),   // Chicken
        14 | 18 | 22 => (0.3, 0.8), // Wolf / Rabbit / Ocelot
        11 | 12 | 13 | 16 => (0.45, 1.3), // Cow / Pig / Sheep / Mooshroom
        17 => (0.4, 0.8),   // Squid
        15 => (0.3, 1.95),  // Villager
        20 => (0.7, 2.7),   // Iron Golem
        21 => (0.35, 1.9),  // Snow Golem
        35 | 40 => (0.7, 0.9), // Spider / Cave Spider
        38 => (0.3, 2.9),   // Enderman
        41 => (2.0, 4.0),   // Ghast
        _ => (0.3, 1.8),    // Humanoid (Zombie, Skeleton, Creeper, Blaze, Witch)
    }
}

/// Returns collision AABB for a block at `(bx, by, bz)`, or `None` if passable.
pub fn block_collision_box(block_id: u8, meta: u8, bx: i32, by: i32, bz: i32) -> Option<Aabb> {
    if block_id == 0 || !is_solid(block_id) {
        return None;
    }
    let fx = bx as f64;
    let fy = by as f64;
    let fz = bz as f64;
    match block_id {
        // Open doors / open gates / pressure plates / signs / ladders
        63 | 68 | 69 | 70 | 72 | 77 | 143 | 147 | 148 | 65 | 106 | 66 | 27 | 28 | 126 => None,
        64 | 71 | 193..=197 => {
            if (meta & 0x04) != 0 {
                None
            } else {
                Some(Aabb::new(fx, fy, fz, fx + 1.0, fy + 1.0, fz + 1.0))
            }
        }
        // Slabs (wooden 158, stone 44, red sandstone 182)
        44 | 158 | 182 => {
            if (meta & 0x08) != 0 {
                Some(Aabb::new(fx, fy + 0.5, fz, fx + 1.0, fy + 1.0, fz + 1.0))
            } else {
                Some(Aabb::new(fx, fy, fz, fx + 1.0, fy + 0.5, fz + 1.0))
            }
        }
        // Fences & Nether Brick Fence & Cobble Walls (tall 1.5 blocks)
        85 | 113 | 139 | 188..=192 => Some(Aabb::new(fx, fy, fz, fx + 1.0, fy + 1.5, fz + 1.0)),
        // Fence Gates
        107 | 183..=187 => {
            if (meta & 0x04) != 0 {
                None
            } else {
                Some(Aabb::new(fx, fy, fz, fx + 1.0, fy + 1.5, fz + 1.0))
            }
        }
        // Beds & Carpet & Cake & Snow Layer
        26 => Some(Aabb::new(fx, fy, fz, fx + 1.0, fy + 0.5625, fz + 1.0)),
        78 => {
            let layers = ((meta & 0x07) + 1) as f64 * 0.125;
            Some(Aabb::new(fx, fy, fz, fx + 1.0, fy + layers, fz + 1.0))
        }
        171 => Some(Aabb::new(fx, fy, fz, fx + 1.0, fy + 0.0625, fz + 1.0)),
        60 => Some(Aabb::new(fx, fy, fz, fx + 1.0, fy + 0.9375, fz + 1.0)),
        88 => Some(Aabb::new(fx, fy, fz, fx + 1.0, fy + 0.875, fz + 1.0)),
        // Trapdoors
        96 | 167 => {
            if (meta & 0x08) != 0 {
                None
            } else if (meta & 0x04) != 0 {
                Some(Aabb::new(fx, fy + 0.8125, fz, fx + 1.0, fy + 1.0, fz + 1.0))
            } else {
                Some(Aabb::new(fx, fy, fz, fx + 1.0, fy + 0.1875, fz + 1.0))
            }
        }
        _ => Some(Aabb::new(fx, fy, fz, fx + 1.0, fy + 1.0, fz + 1.0)),
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MoveCollisionResult {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub dx: f64,
    pub dy: f64,
    pub dz: f64,
    pub on_ground: bool,
    pub collided_horizontally: bool,
}

fn collect_collision_boxes(world: &World, area: Aabb) -> Vec<Aabb> {
    let min_x = area.min_x.floor() as i32 - 1;
    let max_x = area.max_x.ceil() as i32 + 1;
    let min_y = (area.min_y.floor() as i32 - 1).max(0);
    let max_y = (area.max_y.ceil() as i32 + 1).min(255);
    let min_z = area.min_z.floor() as i32 - 1;
    let max_z = area.max_z.ceil() as i32 + 1;

    let mut boxes = Vec::new();
    for bz in min_z..=max_z {
        for bx in min_x..=max_x {
            for by in min_y..=max_y {
                let id = world.get_block_id_opt(bx, by, bz);
                if id == 0 {
                    continue;
                }
                let meta = world.get_block_data_opt(bx, by, bz);
                if let Some(b) = block_collision_box(id, meta, bx, by, bz) {
                    boxes.push(b);
                }
            }
        }
    }
    boxes
}

/// Resolves entity motion against world block AABBs with optional step-up support.
pub fn move_entity_with_collision(
    world: &World,
    x: f64,
    y: f64,
    z: f64,
    mut dx: f64,
    mut dy: f64,
    mut dz: f64,
    half_w: f64,
    height: f64,
    step_height: f64,
    was_on_ground: bool,
) -> MoveCollisionResult {
    let mov_x = dx;
    let mov_y = dy;
    let mov_z = dz;

    let initial_bb = Aabb::from_entity(x, y, z, half_w, height);
    let boxes = collect_collision_boxes(world, initial_bb.expand_motion(dx, dy, dz));

    let mut move_bb = initial_bb;
    for b in &boxes {
        dy = b.calculate_y_offset(&move_bb, dy);
    }
    move_bb = move_bb.offset(0.0, dy, 0.0);

    let falling_flag = was_on_ground || (dy != mov_y && mov_y < 0.0);

    for b in &boxes {
        dx = b.calculate_x_offset(&move_bb, dx);
    }
    move_bb = move_bb.offset(dx, 0.0, 0.0);

    for b in &boxes {
        dz = b.calculate_z_offset(&move_bb, dz);
    }
    move_bb = move_bb.offset(0.0, 0.0, dz);

    if step_height > 0.0 && falling_flag && ((mov_x - dx).abs() > 1e-6 || (mov_z - dz).abs() > 1e-6) {
        let cx = dx;
        let _cy = dy;
        let cz = dz;

        let mut sx = mov_x;
        let mut sy = step_height;
        let mut sz = mov_z;

        let step_boxes = collect_collision_boxes(world, initial_bb.expand_motion(sx, sy, sz));
        let mut step_bb = initial_bb;
        for b in &step_boxes {
            sy = b.calculate_y_offset(&step_bb, sy);
        }
        step_bb = step_bb.offset(0.0, sy, 0.0);

        for b in &step_boxes {
            sx = b.calculate_x_offset(&step_bb, sx);
        }
        step_bb = step_bb.offset(sx, 0.0, 0.0);

        for b in &step_boxes {
            sz = b.calculate_z_offset(&step_bb, sz);
        }
        step_bb = step_bb.offset(0.0, 0.0, sz);

        let mut rev_dy = -sy;
        for b in &step_boxes {
            rev_dy = b.calculate_y_offset(&step_bb, rev_dy);
        }
        step_bb = step_bb.offset(0.0, rev_dy, 0.0);

        if (sx * sx + sz * sz) > (cx * cx + cz * cz) {
            dx = sx;
            dy = sy + rev_dy;
            dz = sz;
            move_bb = step_bb;
        }
    }

    let new_x = (move_bb.min_x + move_bb.max_x) * 0.5;
    let new_y = move_bb.min_y;
    let new_z = (move_bb.min_z + move_bb.max_z) * 0.5;

    let on_ground = (dy - mov_y).abs() > 1e-6 && mov_y < 0.0;
    let collided_horizontally = (mov_x - dx).abs() > 1e-6 || (mov_z - dz).abs() > 1e-6;

    MoveCollisionResult {
        x: new_x,
        y: new_y,
        z: new_z,
        dx,
        dy,
        dz,
        on_ground,
        collided_horizontally,
    }
}

pub fn is_in_water(world: &World, x: f64, y: f64, z: f64) -> bool {
    let id = world.get_block_id_opt(x.floor() as i32, y.floor() as i32, z.floor() as i32);
    id == 8 || id == 9
}

pub fn is_in_lava(world: &World, x: f64, y: f64, z: f64) -> bool {
    let id = world.get_block_id_opt(x.floor() as i32, y.floor() as i32, z.floor() as i32);
    id == 10 || id == 11
}

pub fn is_head_suffocating(world: &World, x: f64, eye_y: f64, z: f64) -> bool {
    let bx = x.floor() as i32;
    let by = eye_y.floor() as i32;
    let bz = z.floor() as i32;
    let id = world.get_block_id_opt(bx, by, bz);
    if id == 0 || crate::block::physics::is_transparent(id) || !is_solid(id) {
        return false;
    }
    let meta = world.get_block_data_opt(bx, by, bz);
    if let Some(bb) = block_collision_box(id, meta, bx, by, bz) {
        eye_y >= bb.min_y && eye_y <= bb.max_y
    } else {
        false
    }
}
