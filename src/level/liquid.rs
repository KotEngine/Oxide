use std::net::SocketAddr;
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use parking_lot::RwLock;
use crate::level::World;
use crate::protocol::handler::PlayerState;
use crate::raknet::RakNetServer;
use crate::protocol::packets::{UpdateBlockPacket, LevelSoundEventPacket, LevelEventPacket, update_block, level_event};
use crate::protocol::packet::DataPacket;

pub struct LiquidSimulation {
    pending: RwLock<Vec<(i32, i32, i32, i32)>>,
}

impl Default for LiquidSimulation {
    fn default() -> Self {
        Self::new()
    }
}

impl LiquidSimulation {
    pub fn new() -> Self {
        Self {
            pending: RwLock::new(Vec::new()),
        }
    }

    pub fn schedule_update(&self, x: i32, y: i32, z: i32) {
        self.schedule_update_dim(0, x, y, z);
    }

    pub fn schedule_update_dim(&self, dim: i32, x: i32, y: i32, z: i32) {
        if y < 0 || y >= 256 {
            return;
        }
        self.pending.write().push((dim, x, y, z));
    }

    pub async fn tick(
        &self,
        world: &Arc<RwLock<World>>,
        rak: &RakNetServer,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    ) {
        self.tick_all(&[world, world, world], rak, players).await;
    }

    pub async fn tick_all(
        &self,
        worlds: &[&Arc<RwLock<World>>; 3],
        rak: &RakNetServer,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    ) {
        let coords: Vec<(i32, i32, i32, i32)> = {
            let mut list = self.pending.write();
            if list.is_empty() {
                return;
            }
            std::mem::take(&mut *list)
        };

        let mut unique_coords: HashSet<(i32, i32, i32, i32)> = HashSet::new();
        for pos in coords {
            unique_coords.insert(pos);
        }

        let mut next_tick_queue: Vec<(i32, i32, i32, i32)> = Vec::new();
        let mut packets_by_dim: [Vec<Vec<u8>>; 3] = [Vec::new(), Vec::new(), Vec::new()];

        for (dim, x, y, z) in unique_coords {
            if y < 0 || y >= 256 {
                continue;
            }
            let dim_idx = match dim {
                1 => 1,
                2 => 2,
                _ => 0,
            };
            let world = worlds[dim_idx];

            let (block_id, meta) = read_block_and_meta(world, x, y, z);

            if block_id == 8 || block_id == 9 {
                self.simulate_liquid(
                    dim, x, y, z, block_id, meta, true, world,
                    &mut packets_by_dim[dim_idx], &mut next_tick_queue,
                );
            } else if block_id == 10 || block_id == 11 {
                self.simulate_liquid(
                    dim, x, y, z, block_id, meta, false, world,
                    &mut packets_by_dim[dim_idx], &mut next_tick_queue,
                );
            }
        }

        for dim_idx in 0..3 {
            if !packets_by_dim[dim_idx].is_empty() {
                broadcast_batch_dim(rak, players, dim_idx as i32, &packets_by_dim[dim_idx]).await;
            }
        }

        if !next_tick_queue.is_empty() {
            let mut list = self.pending.write();
            list.extend(next_tick_queue);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn simulate_liquid(
        &self,
        dim: i32,
        x: i32,
        y: i32,
        z: i32,
        block_id: u8,
        meta: u8,
        is_water: bool,
        world: &Arc<RwLock<World>>,
        packets: &mut Vec<Vec<u8>>,
        next_queue: &mut Vec<(i32, i32, i32, i32)>,
    ) {
        let base_flowing = if is_water { 8u8 } else { 10u8 };
        let base_still = if is_water { 9u8 } else { 11u8 };
        let decay_step = if is_water || dim == 1 { 1u8 } else { 2u8 };

        let mut current_meta = meta;
        let active = true;

        // 1. Decay & infinite water recalculation for non-source blocks
        if current_meta > 0 {
            let mut smallest: i16 = 100;
            let mut adj_sources = 0u8;
            for (sx, sz) in [(x - 1, z), (x + 1, z), (x, z - 1), (x, z + 1)] {
                let (sid, smeta) = read_block_and_meta(world, sx, y, sz);
                if sid == base_flowing || sid == base_still {
                    if smeta == 0 {
                        adj_sources += 1;
                    }
                    let eff = if smeta >= 8 { 0 } else { smeta as i16 };
                    if eff < smallest {
                        smallest = eff;
                    }
                }
            }

            let mut new_decay: i16 = if smallest == 100 {
                -1
            } else {
                smallest + decay_step as i16
            };
            if new_decay >= 8 {
                new_decay = -1;
            }

            // Liquid above forces downward falling state
            if y + 1 < 256 {
                let (above_id, above_meta) = read_block_and_meta(world, x, y + 1, z);
                if above_id == base_flowing || above_id == base_still {
                    new_decay = (above_meta | 0x08) as i16;
                }
            }

            // Infinite water source formation
            if is_water && adj_sources >= 2 && y > 0 {
                let (below_id, below_meta) = read_block_and_meta(world, x, y - 1, z);
                let solid_or_source = (below_id != 0 && !is_replaceable(below_id) && below_id != 10 && below_id != 11)
                    || ((below_id == 8 || below_id == 9) && below_meta == 0);
                if solid_or_source {
                    new_decay = 0;
                }
            }

            if new_decay < 0 {
                set_block_world(world, x, y, z, 0, 0);
                packets.push(UpdateBlockPacket::new(x, y as u32, z, 0, 0, update_block::FLAG_ALL_PRIORITY).encode());
                schedule_neighbors(dim, x, y, z, next_queue);
                return;
            } else if new_decay as u8 != current_meta {
                current_meta = new_decay as u8;
                let id_to_set = if current_meta == 0 { block_id } else { base_flowing };
                set_block_world(world, x, y, z, id_to_set, current_meta);
                packets.push(
                    UpdateBlockPacket::new(x, y as u32, z, id_to_set as u32, current_meta as u32, update_block::FLAG_ALL_PRIORITY).encode()
                );
                next_queue.push((dim, x, y, z));
            }
        } else {
            // Check infinite water hardening / neighbors on source
            if is_water {
                let mut touched_lava = false;
                for (sx, sy, sz) in [(x - 1, y, z), (x + 1, y, z), (x, y - 1, z), (x, y, z - 1), (x, y, z + 1)] {
                    if sy >= 0 && sy < 256 {
                        let (sid, smeta) = read_block_and_meta(world, sx, sy, sz);
                        if sid == 10 || sid == 11 {
                            let res = if smeta == 0 { 49 } else { 4 };
                            set_block_world(world, sx, sy, sz, res, 0);
                            packets.push(UpdateBlockPacket::new(sx, sy as u32, sz, res as u32, 0, update_block::FLAG_ALL_PRIORITY).encode());
                            push_fizz(sx as f32, sy as f32, sz as f32, packets);
                            touched_lava = true;
                        }
                    }
                }
                if touched_lava {
                    schedule_neighbors(dim, x, y, z, next_queue);
                }
            }
        }

        if !active {
            return;
        }

        // 2. Downward flow check
        let mut flowed_down = false;
        let mut can_flow_below = false;
        if y > 0 {
            let (below_id, below_meta) = read_block_and_meta(world, x, y - 1, z);
            if is_replaceable(below_id) {
                let fall_meta = current_meta | 0x08;
                set_block_world(world, x, y - 1, z, base_flowing, fall_meta);
                packets.push(UpdateBlockPacket::new(x, (y - 1) as u32, z, base_flowing as u32, fall_meta as u32, update_block::FLAG_ALL_PRIORITY).encode());
                next_queue.push((dim, x, y - 1, z));
                flowed_down = true;
                can_flow_below = true;
            } else if is_water && (below_id == 10 || below_id == 11) {
                let new_block = if below_meta == 0 { 49 } else { 4 };
                set_block_world(world, x, y - 1, z, new_block, 0);
                packets.push(UpdateBlockPacket::new(x, (y - 1) as u32, z, new_block as u32, 0, update_block::FLAG_ALL_PRIORITY).encode());
                push_fizz(x as f32, (y - 1) as f32, z as f32, packets);
                flowed_down = true;
            } else if !is_water && (below_id == 8 || below_id == 9) {
                set_block_world(world, x, y - 1, z, 1, 0); // Stone
                packets.push(UpdateBlockPacket::new(x, (y - 1) as u32, z, 1, 0, update_block::FLAG_ALL_PRIORITY).encode());
                push_fizz(x as f32, (y - 1) as f32, z as f32, packets);
                flowed_down = true;
            } else if below_id == base_flowing || below_id == base_still {
                can_flow_below = true;
            }
        }

        if flowed_down && current_meta != 0 {
            let _ = can_flow_below;
            return;
        }

        // 3. Horizontal spread
        let current_level = if current_meta >= 8 { 0 } else { current_meta };
        let next_decay = current_level + decay_step;
        if next_decay < 8 {
            let sides = [(x - 1, y, z), (x + 1, y, z), (x, y, z - 1), (x, y, z + 1)];
            for (sx, sy, sz) in sides {
                let (side_id, side_meta) = read_block_and_meta(world, sx, sy, sz);

                if is_replaceable(side_id) {
                    set_block_world(world, sx, sy, sz, base_flowing, next_decay);
                    packets.push(
                        UpdateBlockPacket::new(sx, sy as u32, sz, base_flowing as u32, next_decay as u32, update_block::FLAG_ALL_PRIORITY).encode()
                    );
                    next_queue.push((dim, sx, sy, sz));
                } else if side_id == base_flowing && side_meta > next_decay && side_meta < 8 {
                    set_block_world(world, sx, sy, sz, base_flowing, next_decay);
                    packets.push(
                        UpdateBlockPacket::new(sx, sy as u32, sz, base_flowing as u32, next_decay as u32, update_block::FLAG_ALL_PRIORITY).encode()
                    );
                    next_queue.push((dim, sx, sy, sz));
                } else if is_water && (side_id == 10 || side_id == 11) {
                    let result_block = if side_meta == 0 { 49 } else { 4 };
                    set_block_world(world, sx, sy, sz, result_block, 0);
                    packets.push(
                        UpdateBlockPacket::new(sx, sy as u32, sz, result_block as u32, 0, update_block::FLAG_ALL_PRIORITY).encode()
                    );
                    push_fizz(sx as f32, sy as f32, sz as f32, packets);
                } else if !is_water && (side_id == 8 || side_id == 9) {
                    set_block_world(world, sx, sy, sz, 4, 0);
                    packets.push(
                        UpdateBlockPacket::new(sx, sy as u32, sz, 4, 0, update_block::FLAG_ALL_PRIORITY).encode()
                    );
                    push_fizz(sx as f32, sy as f32, sz as f32, packets);
                }
            }
        }
    }
}

pub fn is_replaceable(id: u8) -> bool {
    // Air, saplings, tall grass, dead bush, flowers, mushrooms, torches, fire, redstone, crops, signs, snow, reeds, vines, lily pads, double plants
    matches!(id, 0 | 6 | 31 | 32 | 37 | 38 | 39 | 40 | 50 | 51 | 55 | 59 | 63 | 68 | 78 | 83 | 106 | 111 | 175)
}

fn read_block_and_meta(world: &Arc<RwLock<World>>, x: i32, y: i32, z: i32) -> (u8, u8) {
    if y < 0 || y >= 256 {
        return (0, 0);
    }
    let mut w = world.write();
    let id = w.get_block_id(x, y, z);
    let chunk_x = x >> 4;
    let chunk_z = z >> 4;
    let cx = (x & 0x0f) as usize;
    let cz = (z & 0x0f) as usize;
    let chunk = w.get_chunk_or_generate(chunk_x, chunk_z);
    let meta = chunk.get_block_data(cx, y as usize, cz);
    (id, meta)
}

fn set_block_world(world: &Arc<RwLock<World>>, x: i32, y: i32, z: i32, id: u8, meta: u8) {
    world.write().set_block(x, y, z, id, meta);
}

fn schedule_neighbors(dim: i32, x: i32, y: i32, z: i32, queue: &mut Vec<(i32, i32, i32, i32)>) {
    for (nx, ny, nz) in [
        (x - 1, y, z), (x + 1, y, z),
        (x, y - 1, z), (x, y + 1, z),
        (x, y, z - 1), (x, y, z + 1),
    ] {
        if ny >= 0 && ny < 256 {
            queue.push((dim, nx, ny, nz));
        }
    }
}

fn push_fizz(x: f32, y: f32, z: f32, packets: &mut Vec<Vec<u8>>) {
    packets.push(LevelSoundEventPacket::new(level_event::SOUND_FIZZ, x + 0.5, y + 0.5, z + 0.5).encode());
    packets.push(LevelEventPacket::new(level_event::EVENT_SOUND_FIZZ, x + 0.5, y + 0.5, z + 0.5, 0).encode());
}

async fn broadcast_batch_dim(
    rak: &RakNetServer,
    players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    dim: i32,
    packets: &[Vec<u8>],
) {
    let recipients: Vec<SocketAddr> = {
        let guard = players.read();
        guard.iter()
            .filter(|(_, s)| s.logged_in && s.dimension == dim)
            .map(|(a, _)| *a)
            .collect()
    };
    if recipients.is_empty() || packets.is_empty() {
        return;
    }
    let batch = crate::protocol::batch::BatchPacket::encode(packets);
    for addr in recipients {
        rak.send_encapsulated(addr, batch.clone(), true).await;
    }
}
