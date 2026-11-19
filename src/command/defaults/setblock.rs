use crate::command::{Command, CommandSender};
use crate::server::OxideServer;
use crate::protocol::packets::{UpdateBlockPacket, LevelEventPacket, update_block, level_event};
use crate::protocol::packet::DataPacket;

pub struct SetBlockCommand;

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

impl Command for SetBlockCommand {
    fn name(&self) -> &str { "setblock" }
    fn description(&self) -> &str { "Sets a block at specified coordinates" }
    fn usage(&self) -> &str { "/setblock <x> <y> <z> <block> [data] [replace|destroy|keep]" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.setblock") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>) {
        if args.len() < 4 {
            sender.send_message("Usage: /setblock <x> <y> <z> <block> [data] [replace|destroy|keep]");
            return;
        }

        let (base_x, base_y, base_z) = if let Some(addr) = sender.player_addr() {
            server.players.read().get(&addr).map(|s| (s.x, s.y, s.z)).unwrap_or((0.0, 4.0, 0.0))
        } else {
            (0.0, 4.0, 0.0)
        };

        let x = match parse_coord(&args[0], base_x) {
            Some(v) => v,
            None => { sender.send_message(&format!("Invalid X coordinate: {}", args[0])); return; }
        };
        let y = match parse_coord(&args[1], base_y) {
            Some(v) if (0..256).contains(&v) => v as u32,
            _ => { sender.send_message(&format!("Invalid Y coordinate (must be 0-255): {}", args[1])); return; }
        };
        let z = match parse_coord(&args[2], base_z) {
            Some(v) => v,
            None => { sender.send_message(&format!("Invalid Z coordinate: {}", args[2])); return; }
        };

        let block_id = match args[3].parse::<u32>() {
            Ok(id) => id,
            Err(_) => {
                // Try matching common block names
                match args[3].to_lowercase().as_str() {
                    "air" => 0,
                    "stone" => 1,
                    "grass" => 2,
                    "dirt" => 3,
                    "cobblestone" | "cobble" => 4,
                    "planks" | "wood" => 5,
                    "bedrock" => 7,
                    "water" => 8,
                    "lava" => 10,
                    "sand" => 12,
                    "gravel" => 13,
                    "gold_ore" => 14,
                    "iron_ore" => 15,
                    "coal_ore" => 16,
                    "glass" => 20,
                    "lapis_block" => 22,
                    "sandstone" => 24,
                    "wool" => 35,
                    "gold_block" => 41,
                    "iron_block" => 42,
                    "brick" => 45,
                    "tnt" => 46,
                    "bookshelf" => 47,
                    "moss_stone" => 48,
                    "obsidian" => 49,
                    "torch" => 50,
                    "fire" => 51,
                    "diamond_block" => 57,
                    "crafting_table" => 58,
                    "furnace" => 61,
                    "ladder" => 65,
                    "snow" => 78,
                    "ice" => 79,
                    "glowstone" => 89,
                    "stone_brick" => 98,
                    "emerald_block" => 133,
                    "quartz_block" => 155,
                    "sea_lantern" => 169,
                    _ => { sender.send_message(&format!("Unknown block name: {}", args[3])); return; }
                }
            }
        };

        let block_data = args.get(4).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        let mode = args.get(5).map(|s| s.to_lowercase()).unwrap_or_else(|| "replace".to_string());

        let old_id = {
            let mut w = server.world.write();
            w.get_block_id(x, y as i32, z)
        };

        if mode == "keep" && old_id != 0 {
            sender.send_message("Could not place block: location is not empty (keep mode)");
            return;
        }

        if mode == "destroy" && old_id != 0 {
            let destroy_pk = LevelEventPacket::new(
                level_event::EVENT_PARTICLE_DESTROY,
                x as f32 + 0.5,
                y as f32 + 0.5,
                z as f32 + 0.5,
                old_id as i32,
            );
            server.broadcast_packet(destroy_pk.encode());
        }

        {
            let mut w = server.world.write();
            w.set_block(x, y as i32, z, block_id as u8, block_data as u8);
        }

        let pk = UpdateBlockPacket::new(x, y, z, block_id, block_data, update_block::FLAG_ALL);
        server.broadcast_packet(pk.encode());

        // Schedule liquid simulation if placing liquid
        if block_id == 8 || block_id == 9 || block_id == 10 || block_id == 11 {
            server.liquid_sim.schedule_update(x, y as i32, z);
        }

        sender.send_message(&format!("Set block at {}, {}, {} to ID {}", x, y, z, block_id));
    }
}
