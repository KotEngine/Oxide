use crate::command::{Command, CommandSender};
use crate::server::OxideServer;

pub struct GamemodeCommand;

const MODES: &[(i32, &str)] = &[
    (0, "survival"), (1, "creative"), (2, "adventure"), (3, "spectator"),
];

fn parse_mode(input: &str) -> Option<i32> {
    let clean = input.trim().trim_matches('"').to_lowercase();
    if let Ok(n) = clean.parse::<i32>() {
        return Some(n.clamp(0, 3));
    }
    match clean.as_str() {
        "s" | "surv" | "survival" => Some(0),
        "c" | "create" | "creative" => Some(1),
        "a" | "adv" | "adventure" => Some(2),
        "v" | "spec" | "spectator" => Some(3),
        _ => None,
    }
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

        // Determine which argument is the mode and which is the player
        let (mode_opt, target_arg) = if let Some(m) = parse_mode(&args[0]) {
            let target = if args.len() > 1 { Some(args[1].as_str()) } else { None };
            (Some(m), target)
        } else if args.len() > 1 {
            if let Some(m) = parse_mode(&args[1]) {
                (Some(m), Some(args[0].as_str()))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        let mode = match mode_opt {
            Some(m) => m,
            None => {
                sender.send_message(&format!("Unknown game mode: {}", args[0]));
                return;
            }
        };

        let mode_name = MODES.iter().find(|(id, _)| *id == mode).map(|(_, n)| *n).unwrap_or("unknown");

        let target_res = match target_arg {
            Some(t) => {
                let clean = t.trim().trim_matches('"');
                if clean == "@s" || clean == "@p" || clean == "@self" || clean == "@me" {
                    if let Some(addr) = sender.player_addr() {
                        Some((addr, sender.name().to_string()))
                    } else {
                        server.find_player_by_name(clean)
                    }
                } else {
                    server.find_player_by_name(clean)
                }
            }
            None => {
                if let Some(addr) = sender.player_addr() {
                    Some((addr, sender.name().to_string()))
                } else {
                    None
                }
            }
        };

        match target_res {
            Some((addr, name)) => {
                server.set_player_gamemode(addr, mode);
                sender.send_message(&format!("Set game mode of {} to {}", name, mode_name));
            }
            None => {
                if let Some(t) = target_arg {
                    sender.send_message(&format!("Player '{}' not found", t));
                } else {
                    sender.send_message("Usage: /gamemode <mode> [player]");
                }
            }
        }
    }
}
