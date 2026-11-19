use crate::command::{Command, CommandSender};
use crate::server::OxideServer;
use crate::protocol::packets::{LevelEventPacket, level_event};
use crate::protocol::packet::DataPacket;

pub struct ParticleCommand;

fn parse_coord(coord_str: &str, base: f64) -> Option<f32> {
    if let Some(rest) = coord_str.strip_prefix('~') {
        if rest.is_empty() {
            Some(base as f32)
        } else {
            let offset: f64 = rest.parse().ok()?;
            Some((base + offset) as f32)
        }
    } else {
        coord_str.parse::<f32>().ok()
    }
}

impl Command for ParticleCommand {
    fn name(&self) -> &str { "particle" }
    fn description(&self) -> &str { "Spawns visual particles at coordinates" }
    fn usage(&self) -> &str { "/particle <name> <x> <y> <z> [data]" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.particle") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>) {
        if args.len() < 4 {
            sender.send_message("Usage: /particle <name> <x> <y> <z> [data]");
            return;
        }

        let (base_x, base_y, base_z) = if let Some(addr) = sender.player_addr() {
            server.players.read().get(&addr).map(|s| (s.x, s.y, s.z)).unwrap_or((0.0, 4.0, 0.0))
        } else {
            (0.0, 4.0, 0.0)
        };

        let part_name = args[0].to_lowercase();
        let evid = match part_name.as_str() {
            "destroy" => level_event::EVENT_PARTICLE_DESTROY,
            "splash" => level_event::EVENT_PARTICLE_SPLASH,
            "spawn" => level_event::EVENT_PARTICLE_SPAWN,
            "teleport" => level_event::EVENT_PARTICLE_ENDERMAN_TELEPORT,
            "shoot" => level_event::EVENT_PARTICLE_SHOOT,
            "fizz" | "smoke" => level_event::EVENT_SOUND_FIZZ,
            _ => {
                if let Ok(id) = part_name.parse::<i32>() {
                    id
                } else {
                    level_event::EVENT_PARTICLE_SPAWN
                }
            }
        };

        let x = match parse_coord(&args[1], base_x) { Some(v) => v, None => return };
        let y = match parse_coord(&args[2], base_y) { Some(v) => v, None => return };
        let z = match parse_coord(&args[3], base_z) { Some(v) => v, None => return };
        let data = args.get(4).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);

        let pk = LevelEventPacket::new(evid, x, y, z, data);
        server.broadcast_packet(pk.encode());

        sender.send_message(&format!("Spawned particle '{}' at {}, {}, {}", part_name, x, y, z));
    }
}
