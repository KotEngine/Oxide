use crate::command::{Command, CommandSender};
use crate::server::OxideServer;

pub struct DefaultGamemodeCommand;

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

impl Command for DefaultGamemodeCommand {
    fn name(&self) -> &str { "defaultgamemode" }
    fn description(&self) -> &str { "Sets the default game mode for new players" }
    fn usage(&self) -> &str { "/defaultgamemode <mode>" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.defaultgamemode") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>) {
        if args.is_empty() {
            sender.send_message("Usage: /defaultgamemode <mode>");
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
        server.config.write().set("gamemode", mode.into());
        sender.send_message(&format!("Default game mode set to {}", mode_name));
    }
}
