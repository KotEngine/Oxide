use crate::command::{Command, CommandSender};
use crate::server::OxideServer;
use crate::protocol::packets::{PlaySoundPacket, LevelSoundEventPacket};
use crate::protocol::packet::DataPacket;

pub struct PlaysoundCommand;

impl Command for PlaysoundCommand {
    fn name(&self) -> &str { "playsound" }
    fn description(&self) -> &str { "Plays a sound for players" }
    fn usage(&self) -> &str { "/playsound <sound> [player] [x] [y] [z] [volume] [pitch]" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.playsound") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>) {
        if args.is_empty() {
            sender.send_message("Usage: /playsound <sound> [player] [x] [y] [z] [volume] [pitch]");
            return;
        }

        let sound_name = &args[0];
        let target_res = if args.len() > 1 && args[1] != "@a" {
            server.find_player_by_name(&args[1])
        } else if let Some(addr) = sender.player_addr() {
            Some((addr, sender.name().to_string()))
        } else {
            None
        };

        let (x, y, z) = if args.len() >= 5 {
            (
                args[2].parse::<f32>().unwrap_or(0.0),
                args[3].parse::<f32>().unwrap_or(4.0),
                args[4].parse::<f32>().unwrap_or(0.0),
            )
        } else if let Some((addr, _)) = target_res {
            server.players.read().get(&addr).map(|s| (s.x as f32, s.y as f32, s.z as f32)).unwrap_or((0.0, 4.0, 0.0))
        } else {
            (0.0, 4.0, 0.0)
        };

        let volume = args.get(5).and_then(|s| s.parse::<f32>().ok()).unwrap_or(1.0);
        let pitch = args.get(6).and_then(|s| s.parse::<f32>().ok()).unwrap_or(1.0);

        if let Ok(sound_id) = sound_name.parse::<u8>() {
            let pk = LevelSoundEventPacket::new(sound_id, x, y, z);
            let encoded = pk.encode();
            if let Some((addr, _)) = target_res {
                server.send_packet_to(addr, encoded);
            } else {
                server.broadcast_packet(encoded);
            }
        } else {
            let pk = PlaySoundPacket {
                name: sound_name.to_string(),
                x: (x * 8.0) as i32,
                y: (y * 8.0) as i32,
                z: (z * 8.0) as i32,
                volume,
                pitch,
            };
            let encoded = pk.encode();
            if let Some((addr, _)) = target_res {
                server.send_packet_to(addr, encoded);
            } else {
                server.broadcast_packet(encoded);
            }
        }

        sender.send_message(&format!("Played sound '{}' at {}, {}, {}", sound_name, x, y, z));
    }
}
