use crate::command::{Command, CommandSender};
use crate::server::OxideServer;
use crate::protocol::packets::{LevelEventPacket, level_event};
use crate::protocol::packet::DataPacket;

pub struct XpCommand;

impl Command for XpCommand {
    fn name(&self) -> &str { "xp" }
    fn description(&self) -> &str { "Gives experience to a player" }
    fn usage(&self) -> &str { "/xp <amount[L]> [player]" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.xp") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>) {
        if args.is_empty() {
            sender.send_message("Usage: /xp <amount[L]> [player]");
            return;
        }

        let input = &args[0];
        let is_level = input.ends_with('L') || input.ends_with('l');
        let num_str = if is_level { &input[..input.len() - 1] } else { input };
        let amount = match num_str.parse::<i32>() {
            Ok(n) => n,
            Err(_) => {
                sender.send_message(&format!("Invalid amount: {}", input));
                return;
            }
        };

        let target_res = if args.len() > 1 {
            server.find_player_by_name(&args[1])
        } else if let Some(addr) = sender.player_addr() {
            Some((addr, sender.name().to_string()))
        } else {
            None
        };

        match target_res {
            Some((addr, name)) => {
                let (px, py, pz) = {
                    let mut guard = server.players.write();
                    if let Some(s) = guard.get_mut(&addr) {
                        if is_level {
                            s.xp_level = (s.xp_level as i32 + amount).max(0) as u32;
                        } else {
                            s.xp_points = (s.xp_points as i32 + amount).max(0) as u32;
                            // Recalculate level if points increased
                            let lvl = (s.xp_points / 100).min(100);
                            s.xp_level = lvl;
                        }
                        (s.x, s.y, s.z)
                    } else {
                        (0.0, 4.0, 0.0)
                    }
                };

                let sound = LevelEventPacket::new(level_event::EVENT_SOUND_ORB, px as f32, py as f32, pz as f32, 0);
                server.send_packet_to(addr, sound.encode());

                if is_level {
                    sender.send_message(&format!("Given {} experience levels to {}", amount, name));
                    if sender.name() != name {
                        server.send_message_to(addr, &format!("You were given {} experience levels", amount));
                    }
                } else {
                    sender.send_message(&format!("Given {} experience points to {}", amount, name));
                    if sender.name() != name {
                        server.send_message_to(addr, &format!("You were given {} experience points", amount));
                    }
                }
            }
            None => {
                sender.send_message("Target player not found");
            }
        }
    }
}
