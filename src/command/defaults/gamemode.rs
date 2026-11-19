use crate::command::{Command, CommandSender};
use crate::server::OxideServer;

pub struct GamemodeCommand;

const MODES: &[(i32, &str)] = &[
    (0, "survival"), (1, "creative"), (2, "adventure"), (3, "spectator"),
];

fn parse_mode(input: &str) -> Option<i32> {
    if let Ok(n) = input.parse::<i32>() {
        return Some(n.clamp(0, 3));
    }
    for (id, name) in MODES {
        if name.starts_with(&input.to_lowercase()) || input.eq_ignore_ascii_case(name) {
            return Some(*id);
        }
    }
    None
}

impl Command for GamemodeCommand {
    fn name(&self) -> &str { "gamemode" }
    fn aliases(&self) -> Vec<&str> { vec!["gm"] }
    fn description(&self) -> &str { "Changes game mode" }
    fn usage(&self) -> &str { "/gamemode <mode> [player]" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.gamemode") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>) {
        if args.is_empty() {
            sender.send_message("Usage: /gamemode <mode> [player]");
            return;
        }
        let mode = match parse_mode(&args[0]) {
            Some(m) => m,
            None => {
                sender.send_message(&format!("Unknown game mode: {}", args[0]));
                return;
            }
        };

        let mode_name = MODES.iter().find(|(id, _)| *id == mode).map(|(_, n)| *n).unwrap_or("unknown");
        let target_res = if args.len() > 1 {
            server.find_player_by_name(&args[1])
        } else if let Some(addr) = sender.player_addr() {
            Some((addr, sender.name().to_string()))
        } else {
            None
        };

        match target_res {
            Some((addr, name)) => {
                server.set_player_gamemode(addr, mode);
                sender.send_message(&format!("Set game mode of {} to {}", name, mode_name));
            }
            None => {
                sender.send_message("Usage: /gamemode <mode> <player>");
            }
        }
    }
}
