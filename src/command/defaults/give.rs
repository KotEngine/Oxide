use crate::command::{Command, CommandSender};
use crate::protocol::packet::DataPacket;
use crate::server::OxideServer;

pub struct GiveCommand;

impl Command for GiveCommand {
    fn name(&self) -> &str { "give" }
    fn description(&self) -> &str { "Gives items to a player" }
    fn usage(&self) -> &str { "/give <player> <item_id> [count] [meta]" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.give") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>) {
        if args.len() < 2 {
            sender.send_message("Usage: /give <player> <item_id> [count] [meta]");
            return;
        }
        let target_name = &args[0];
        let item_id = args[1].parse::<i32>().unwrap_or(1);
        let count = args.get(2).and_then(|s| s.parse::<u32>().ok()).unwrap_or(1);
        let meta = args.get(3).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);

        if let Some((addr, actual_name)) = server.find_player_by_name(target_name) {
            let item = crate::protocol::types::Item::new(item_id, meta, count);
            let entity_id = {
                let mut eid = server.next_entity_id.write();
                let id = *eid;
                *eid += 1;
                id
            };
            let (px, py, pz) = {
                let p = server.players.read();
                p.get(&addr).map(|s| (s.x as f32, s.y as f32 + 1.0, s.z as f32)).unwrap_or((0.0, 5.0, 0.0))
            };
            let pk = crate::protocol::packets::AddItemEntityPacket::new(entity_id, item, px, py, pz);
            server.broadcast_packet(pk.encode());
            sender.send_message(&format!("Gave {} x{} (ID {}) to {}", count, item_id, item_id, actual_name));
        } else {
            sender.send_message(&format!("Player '{}' not found", target_name));
        }
    }
}
