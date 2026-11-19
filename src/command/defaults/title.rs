use crate::command::{Command, CommandSender};
use crate::server::OxideServer;
use crate::protocol::packets::SetTitlePacket;
use crate::protocol::packet::DataPacket;

pub struct TitleCommand;

impl Command for TitleCommand {
    fn name(&self) -> &str { "title" }
    fn description(&self) -> &str { "Controls screen titles" }
    fn usage(&self) -> &str { "/title <player> <clear|reset|title|subtitle|actionbar|times> [args...]" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.title") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>) {
        if args.len() < 2 {
            sender.send_message("Usage: /title <player> <clear|reset|title|subtitle|actionbar|times> [args...]");
            return;
        }

        let target_name = &args[0];
        let target = if target_name == "@a" {
            None
        } else {
            match server.find_player_by_name(target_name) {
                Some(t) => Some(t),
                None => {
                    sender.send_message(&format!("Player '{}' not found", target_name));
                    return;
                }
            }
        };

        let action = args[1].to_lowercase();
        let pk = match action.as_str() {
            "clear" => SetTitlePacket::new(SetTitlePacket::TYPE_CLEAR, ""),
            "reset" => SetTitlePacket::new(SetTitlePacket::TYPE_RESET, ""),
            "title" => {
                let text = args[2..].join(" ");
                SetTitlePacket::new(SetTitlePacket::TYPE_TITLE, text)
            }
            "subtitle" => {
                let text = args[2..].join(" ");
                SetTitlePacket::new(SetTitlePacket::TYPE_SUB_TITLE, text)
            }
            "actionbar" => {
                let text = args[2..].join(" ");
                SetTitlePacket::new(SetTitlePacket::TYPE_ACTION_BAR, text)
            }
            "times" => {
                let fade_in = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(20);
                let stay = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(60);
                let fade_out = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(20);
                let mut p = SetTitlePacket::new(SetTitlePacket::TYPE_TIMES, "");
                p.fade_in_duration = fade_in;
                p.duration = stay;
                p.fade_out_duration = fade_out;
                p
            }
            _ => {
                sender.send_message(&format!("Invalid title action: {}", args[1]));
                return;
            }
        };

        let encoded = pk.encode();
        if let Some((addr, actual_name)) = target {
            server.send_packet_to(addr, encoded);
            sender.send_message(&format!("Sent title command to {}", actual_name));
        } else {
            server.broadcast_packet(encoded);
            sender.send_message("Sent title command to all players");
        }
    }
}
