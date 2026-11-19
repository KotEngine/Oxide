use crate::command::{Command, CommandSender};
use crate::server::OxideServer;
use crate::protocol::packets::{UpdateBlockPacket, update_block};
use crate::protocol::packet::DataPacket;

pub struct FillCommand;

fn parse_coord(coord_str: &str, base: f64) -> Option<i32> {
    if let Some(rest) = coord_str.strip_prefix('~') {
        if rest.is_empty() {
            Some(base.floor() as i32)
        } else {
            let offset: f64 = rest.parse().ok()?;
            Some((base + offset).floor() as i32)
        }
    } else {
        coord_str.parse::<i32>().ok()
    }
}

impl Command for FillCommand {
    fn name(&self) -> &str { "fill" }
    fn description(&self) -> &str { "Fills a region with a specific block" }
    fn usage(&self) -> &str { "/fill <x1> <y1> <z1> <x2> <y2> <z2> <block> [data] [replace|keep|hollow|outline]" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.fill") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>) {
        if args.len() < 7 {
            sender.send_message("Usage: /fill <x1> <y1> <z1> <x2> <y2> <z2> <block> [data] [replace|keep|hollow|outline]");
            return;
        }

        let (base_x, base_y, base_z) = if let Some(addr) = sender.player_addr() {
            server.players.read().get(&addr).map(|s| (s.x, s.y, s.z)).unwrap_or((0.0, 4.0, 0.0))
        } else {
            (0.0, 4.0, 0.0)
        };

        let x1 = match parse_coord(&args[0], base_x) { Some(v) => v, None => return };
        let y1 = match parse_coord(&args[1], base_y) { Some(v) => v.clamp(0, 255), None => return };
        let z1 = match parse_coord(&args[2], base_z) { Some(v) => v, None => return };

        let x2 = match parse_coord(&args[3], base_x) { Some(v) => v, None => return };
        let y2 = match parse_coord(&args[4], base_y) { Some(v) => v.clamp(0, 255), None => return };
        let z2 = match parse_coord(&args[5], base_z) { Some(v) => v, None => return };

        let min_x = x1.min(x2);
        let max_x = x1.max(x2);
        let min_y = y1.min(y2);
        let max_y = y1.max(y2);
        let min_z = z1.min(z2);
        let max_z = z1.max(z2);

        let volume = (max_x - min_x + 1) as u64 * (max_y - min_y + 1) as u64 * (max_z - min_z + 1) as u64;
        if volume > 32768 {
            sender.send_message(&format!("Cannot fill {} blocks (maximum allowed is 32768)", volume));
            return;
        }

        let block_id = match args[6].parse::<u32>() {
            Ok(id) => id,
            Err(_) => {
                match args[6].to_lowercase().as_str() {
                    "air" => 0,
                    "stone" => 1,
                    "grass" => 2,
                    "dirt" => 3,
                    "cobblestone" | "cobble" => 4,
                    "planks" | "wood" => 5,
                    "bedrock" => 7,
                    "sand" => 12,
                    "glass" => 20,
                    "wool" => 35,
                    "brick" => 45,
                    "bookshelf" => 47,
                    "obsidian" => 49,
                    "diamond_block" => 57,
                    "glowstone" => 89,
                    "stone_brick" => 98,
                    "quartz_block" => 155,
                    _ => { sender.send_message(&format!("Unknown block name: {}", args[6])); return; }
                }
            }
        };

        let block_data = args.get(7).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        let mode = args.get(8).map(|s| s.to_lowercase()).unwrap_or_else(|| "replace".to_string());

        let mut changed = 0;
        let mut updates = Vec::new();

        {
            let mut w = server.world.write();
            for x in min_x..=max_x {
                for y in min_y..=max_y {
                    for z in min_z..=max_z {
                        let is_edge = x == min_x || x == max_x || y == min_y || y == max_y || z == min_z || z == max_z;
                        let old_id = w.get_block_id(x, y, z);

                        if mode == "keep" && old_id != 0 {
                            continue;
                        }

                        if mode == "outline" && !is_edge {
                            continue;
                        }

                        let target_id = if mode == "hollow" && !is_edge {
                            0 // Hollow center is air
                        } else {
                            block_id as u8
                        };

                        let target_data = if mode == "hollow" && !is_edge {
                            0
                        } else {
                            block_data as u8
                        };

                        w.set_block(x, y, z, target_id, target_data);
                        let pk = UpdateBlockPacket::new(x, y as u32, z, target_id as u32, target_data as u32, update_block::FLAG_ALL);
                        updates.push(pk.encode());
                        changed += 1;
                    }
                }
            }
        }

        let rak = server.rak.clone();
        let players = server.players.clone();
        tokio::spawn(async move {
            let addrs: Vec<std::net::SocketAddr> = {
                let guard = players.read();
                guard.iter().filter(|(_, s)| s.logged_in).map(|(a, _)| *a).collect()
            };
            if !addrs.is_empty() {
                // Chunk updates into batches of 100 packets
                for chunk in updates.chunks(100) {
                    let batch = crate::protocol::batch::BatchPacket::encode(chunk);
                    for addr in &addrs {
                        rak.send_encapsulated(*addr, batch.clone(), true).await;
                    }
                }
            }
        });

        sender.send_message(&format!("Successfully filled {} blocks", changed));
    }
}
