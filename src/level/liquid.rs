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
    pending: RwLock<Vec<(i32, i32, i32)>>,
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
        if y < 0 || y >= 256 {
            return;
        }
        self.pending.write().push((x, y, z));
    }

    pub async fn tick(
        &self,
        world: &Arc<RwLock<World>>,
        rak: &RakNetServer,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    ) {
        let coords: Vec<(i32, i32, i32)> = {
            let mut list = self.pending.write();
            if list.is_empty() {
                return;
            }
            std::mem::take(&mut *list)
        };

        // Deduplicate updates for this tick
        let mut unique_coords: HashSet<(i32, i32, i32)> = HashSet::new();
        for pos in coords {
            unique_coords.insert(pos);
        }

        let mut next_tick_queue = Vec::new();

        for (x, y, z) in unique_coords {
            if y < 0 || y >= 256 {
                continue;
            }

            let (block_id, meta) = {
                let mut w = world.write();
                let id = w.get_block_id(x, y, z);
                let chunk_x = x >> 4;
                let chunk_z = z >> 4;
                let cx = (x & 0x0f) as usize;
                let cz = (z & 0x0f) as usize;
                let chunk = w.get_chunk_or_generate(chunk_x, chunk_z);
                let meta = chunk.get_block_data(cx, y as usize, cz);
                (id, meta)
            };

            if block_id == 8 || block_id == 9 {
                // Water flow simulation
                self.simulate_water(x, y, z, meta, world, rak, players, &mut next_tick_queue).await;
            } else if block_id == 10 || block_id == 11 {
                // Lava flow simulation
                self.simulate_lava(x, y, z, meta, world, rak, players, &mut next_tick_queue).await;
            }
        }

        if !next_tick_queue.is_empty() {
            let mut list = self.pending.write();
            list.extend(next_tick_queue);
        }
    }

    async fn simulate_water(
        &self,
        x: i32,
        y: i32,
        z: i32,
        meta: u8,
        world: &Arc<RwLock<World>>,
        rak: &RakNetServer,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
        next_queue: &mut Vec<(i32, i32, i32)>,
    ) {
        // Downward flow check
        let mut flowed_down = false;
        if y > 0 {
            let below_id = {
                let mut w = world.write();
                w.get_block_id(x, y - 1, z)
            };

            if is_replaceable(below_id) {
                // Flow down
                {
                    let mut w = world.write();
                    w.set_block(x, y - 1, z, 8, 8); // Flowing water downwards
                }
                let pk = UpdateBlockPacket::new(x, (y - 1) as u32, z, 8, 8, update_block::FLAG_ALL_PRIORITY);
                broadcast_packet(rak, players, pk.encode()).await;
                next_queue.push((x, y - 1, z));
                flowed_down = true;
            } else if below_id == 10 || below_id == 11 {
                // Water hits lava from above -> Obsidian or Cobblestone
                let lava_meta = get_block_meta(world, x, y - 1, z);
                let new_block = if lava_meta == 0 { 49 } else { 4 }; // 49 = Obsidian, 4 = Cobblestone
                {
                    let mut w = world.write();
                    w.set_block(x, y - 1, z, new_block, 0);
                }
                let pk = UpdateBlockPacket::new(x, (y - 1) as u32, z, new_block as u32, 0, update_block::FLAG_ALL_PRIORITY);
                broadcast_packet(rak, players, pk.encode()).await;
                send_fizz(x as f32, (y - 1) as f32, z as f32, rak, players).await;
                flowed_down = true;
            }
        }

        // Only stop horizontal flow if non-source water flowed down
        if flowed_down && meta != 0 {
            return;
        }

        // Horizontal spread
        let current_level = if meta >= 8 { 0 } else { meta };
        if current_level < 7 {
            let next_decay = current_level + 1;
            let sides = [(x - 1, y, z), (x + 1, y, z), (x, y, z - 1), (x, y, z + 1)];
            for (sx, sy, sz) in sides {
                let side_id = {
                    let mut w = world.write();
                    w.get_block_id(sx, sy, sz)
                };

                if is_replaceable(side_id) {
                    {
                        let mut w = world.write();
                        w.set_block(sx, sy, sz, 8, next_decay);
                    }
                    let pk = UpdateBlockPacket::new(sx, sy as u32, sz, 8, next_decay as u32, update_block::FLAG_ALL_PRIORITY);
                    broadcast_packet(rak, players, pk.encode()).await;
                    next_queue.push((sx, sy, sz));
                } else if side_id == 8 {
                    // Update existing flowing water if new decay is stronger
                    let existing_decay = get_block_meta(world, sx, sy, sz);
                    if existing_decay > next_decay {
                        {
                            let mut w = world.write();
                            w.set_block(sx, sy, sz, 8, next_decay);
                        }
                        let pk = UpdateBlockPacket::new(sx, sy as u32, sz, 8, next_decay as u32, update_block::FLAG_ALL_PRIORITY);
                        broadcast_packet(rak, players, pk.encode()).await;
                        next_queue.push((sx, sy, sz));
                    }
                } else if side_id == 10 || side_id == 11 {
                    // Water touches lava horizontally -> Obsidian or Cobblestone
                    let lava_meta = get_block_meta(world, sx, sy, sz);
                    let result_block = if lava_meta == 0 { 49 } else { 4 };
                    {
                        let mut w = world.write();
                        w.set_block(sx, sy, sz, result_block, 0);
                    }
                    let pk = UpdateBlockPacket::new(sx, sy as u32, sz, result_block as u32, 0, update_block::FLAG_ALL_PRIORITY);
                    broadcast_packet(rak, players, pk.encode()).await;
                    send_fizz(sx as f32, sy as f32, sz as f32, rak, players).await;
                }
            }
        }
    }

    async fn simulate_lava(
        &self,
        x: i32,
        y: i32,
        z: i32,
        meta: u8,
        world: &Arc<RwLock<World>>,
        rak: &RakNetServer,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
        next_queue: &mut Vec<(i32, i32, i32)>,
    ) {
        // Downward check
        let mut flowed_down = false;
        if y > 0 {
            let below_id = {
                let mut w = world.write();
                w.get_block_id(x, y - 1, z)
            };

            if is_replaceable(below_id) {
                {
                    let mut w = world.write();
                    w.set_block(x, y - 1, z, 10, 8); // Downward flowing lava
                }
                let pk = UpdateBlockPacket::new(x, (y - 1) as u32, z, 10, 8, update_block::FLAG_ALL_PRIORITY);
                broadcast_packet(rak, players, pk.encode()).await;
                next_queue.push((x, y - 1, z));
                flowed_down = true;
            } else if below_id == 8 || below_id == 9 {
                // Lava flows down onto water -> Stone
                {
                    let mut w = world.write();
                    w.set_block(x, y - 1, z, 1, 0); // Stone
                }
                let pk = UpdateBlockPacket::new(x, (y - 1) as u32, z, 1, 0, update_block::FLAG_ALL_PRIORITY);
                broadcast_packet(rak, players, pk.encode()).await;
                send_fizz(x as f32, (y - 1) as f32, z as f32, rak, players).await;
                flowed_down = true;
            }
        }

        // Only stop horizontal flow if non-source lava flowed down
        if flowed_down && meta != 0 {
            return;
        }

        // Horizontal spread for lava (decay step = 2)
        let current_level = if meta >= 8 { 0 } else { meta };
        if current_level < 6 {
            let next_decay = current_level + 2;
            let sides = [(x - 1, y, z), (x + 1, y, z), (x, y, z - 1), (x, y, z + 1)];
            for (sx, sy, sz) in sides {
                let side_id = {
                    let mut w = world.write();
                    w.get_block_id(sx, sy, sz)
                };

                if is_replaceable(side_id) {
                    {
                        let mut w = world.write();
                        w.set_block(sx, sy, sz, 10, next_decay);
                    }
                    let pk = UpdateBlockPacket::new(sx, sy as u32, sz, 10, next_decay as u32, update_block::FLAG_ALL_PRIORITY);
                    broadcast_packet(rak, players, pk.encode()).await;
                    next_queue.push((sx, sy, sz));
                } else if side_id == 10 {
                    // Update existing flowing lava if new decay is stronger
                    let existing_decay = get_block_meta(world, sx, sy, sz);
                    if existing_decay > next_decay {
                        {
                            let mut w = world.write();
                            w.set_block(sx, sy, sz, 10, next_decay);
                        }
                        let pk = UpdateBlockPacket::new(sx, sy as u32, sz, 10, next_decay as u32, update_block::FLAG_ALL_PRIORITY);
                        broadcast_packet(rak, players, pk.encode()).await;
                        next_queue.push((sx, sy, sz));
                    }
                } else if side_id == 8 || side_id == 9 {
                    // Lava flows into water horizontally -> Cobblestone
                    {
                        let mut w = world.write();
                        w.set_block(sx, sy, sz, 4, 0);
                    }
                    let pk = UpdateBlockPacket::new(sx, sy as u32, sz, 4, 0, update_block::FLAG_ALL_PRIORITY);
                    broadcast_packet(rak, players, pk.encode()).await;
                    send_fizz(sx as f32, sy as f32, sz as f32, rak, players).await;
                }
            }
        }
    }
}

fn is_replaceable(id: u8) -> bool {
    // Air, saplings, tall grass, dead bush, flowers, mushrooms, torches, fire, redstone, crops, signs, snow, reeds, vines, lily pads, double plants
    matches!(id, 0 | 6 | 31 | 32 | 37 | 38 | 39 | 40 | 50 | 51 | 55 | 59 | 63 | 68 | 78 | 83 | 106 | 111 | 175)
}

fn get_block_meta(world: &Arc<RwLock<World>>, x: i32, y: i32, z: i32) -> u8 {
    let mut w = world.write();
    let chunk_x = x >> 4;
    let chunk_z = z >> 4;
    let cx = (x & 0x0f) as usize;
    let cz = (z & 0x0f) as usize;
    let chunk = w.get_chunk_or_generate(chunk_x, chunk_z);
    chunk.get_block_data(cx, y as usize, cz)
}

async fn send_fizz(
    x: f32,
    y: f32,
    z: f32,
    rak: &RakNetServer,
    players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
) {
    let sound = LevelSoundEventPacket::new(level_event::SOUND_FIZZ, x + 0.5, y + 0.5, z + 0.5);
    broadcast_packet(rak, players, sound.encode()).await;

    let part = LevelEventPacket::new(level_event::EVENT_SOUND_FIZZ, x + 0.5, y + 0.5, z + 0.5, 0);
    broadcast_packet(rak, players, part.encode()).await;
}

async fn broadcast_packet(
    rak: &RakNetServer,
    players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    data: Vec<u8>,
) {
    let recipients: Vec<SocketAddr> = {
        let guard = players.read();
        guard.iter()
            .filter(|(_, s)| s.logged_in)
            .map(|(a, _)| *a)
            .collect()
    };
    if recipients.is_empty() {
        return;
    }
    let batch = crate::protocol::batch::BatchPacket::encode(&[data]);
    for addr in recipients {
        rak.send_encapsulated(addr, batch.clone(), true).await;
    }
}
