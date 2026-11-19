use crate::command::{Command, CommandSender};
use crate::server::OxideServer;

pub struct TpCommand;

impl Command for TpCommand {
    fn name(&self) -> &str { "tp" }
    fn aliases(&self) -> Vec<&str> { vec!["teleport"] }
    fn description(&self) -> &str { "Teleports a player to a location" }
    fn usage(&self) -> &str { "/tp [player] <x> <y> <z> | /tp [player] <target>" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.tp") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>) {
        if args.is_empty() {
            sender.send_message("Usage: /tp [player] <x> <y> <z> or /tp [player] <target>");
            return;
        }

        if args.len() == 1 {
            if let Some(sender_addr) = sender.player_addr() {
                if let Some((target_addr, target_name)) = server.find_player_by_name(&args[0]) {
                    let target_pos = {
                        let p = server.players.read();
                        p.get(&target_addr).map(|s| (s.x, s.y, s.z))
                    };
                    if let Some((x, y, z)) = target_pos {
                        server.teleport_player(sender_addr, x, y, z);
                        sender.send_message(&format!("Teleported to {}", target_name));
                    }
                } else {
                    sender.send_message(&format!("Player '{}' not found", args[0]));
                }
            } else {
                sender.send_message("Console usage: /tp <player> <target> or /tp <player> <x> <y> <z>");
            }
        } else if args.len() == 2 {
            if let (Some((p1_addr, p1_name)), Some((p2_addr, p2_name))) = (server.find_player_by_name(&args[0]), server.find_player_by_name(&args[1])) {
                let target_pos = {
                    let p = server.players.read();
                    p.get(&p2_addr).map(|s| (s.x, s.y, s.z))
                };
                if let Some((x, y, z)) = target_pos {
                    server.teleport_player(p1_addr, x, y, z);
                    sender.send_message(&format!("Teleported {} to {}", p1_name, p2_name));
                }
            } else {
                sender.send_message("One or both players not found");
            }
        } else if args.len() == 3 {
            if let Some(sender_addr) = sender.player_addr() {
                let x = args[0].parse::<f64>().unwrap_or(0.0);
                let y = args[1].parse::<f64>().unwrap_or(64.0);
                let z = args[2].parse::<f64>().unwrap_or(0.0);
                server.teleport_player(sender_addr, x, y, z);
                sender.send_message(&format!("Teleported to {:.2}, {:.2}, {:.2}", x, y, z));
            } else {
                sender.send_message("Console usage: /tp <player> <x> <y> <z>");
            }
        } else if args.len() >= 4 {
            if let Some((addr, name)) = server.find_player_by_name(&args[0]) {
                let x = args[1].parse::<f64>().unwrap_or(0.0);
                let y = args[2].parse::<f64>().unwrap_or(64.0);
                let z = args[3].parse::<f64>().unwrap_or(0.0);
                server.teleport_player(addr, x, y, z);
                sender.send_message(&format!("Teleported {} to {:.2}, {:.2}, {:.2}", name, x, y, z));
            } else {
                sender.send_message(&format!("Player '{}' not found", args[0]));
            }
        }
    }
}
