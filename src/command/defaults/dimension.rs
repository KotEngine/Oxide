use crate::command::{Command, CommandSender};
use crate::server::OxideServer;

pub struct DimensionCommand;

impl Command for DimensionCommand {
    fn name(&self) -> &str {
        "dimension"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["dim"]
    }

    fn description(&self) -> &str {
        "Changes a player's dimension (0: Overworld, 1: Nether, 2: The End)"
    }

    fn usage(&self) -> &str {
        "/dimension [player] <0|1|2|overworld|nether|end>"
    }

    fn permission(&self) -> Option<&str> {
        Some("oxide.command.dimension")
    }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>) {
        if args.is_empty() {
            sender.send_message("Usage: /dimension [player] <0|1|2|overworld|nether|end>");
            return;
        }

        let (target_addr, target_name, dim_arg) = if args.len() == 1 {
            if let Some(addr) = sender.player_addr() {
                let name = sender.name().to_string();
                (addr, name, &args[0])
            } else {
                sender.send_message("Console must specify a player: /dimension <player> <dim>");
                return;
            }
        } else {
            if let Some((addr, name)) = server.find_player_by_name(&args[0]) {
                (addr, name, &args[1])
            } else {
                sender.send_message(&format!("Player '{}' not found", args[0]));
                return;
            }
        };

        let dim = match dim_arg.to_lowercase().as_str() {
            "0" | "overworld" | "world" => 0,
            "1" | "nether" | "hell" => 1,
            "2" | "the_end" | "end" | "ender" => 2,
            _ => {
                sender.send_message("Invalid dimension. Valid choices: 0 (Overworld), 1 (Nether), 2 (The End)");
                return;
            }
        };

        let (tx, ty, tz) = match dim {
            1 => (0.0, 64.0, 0.0),
            2 => (0.0, 66.0, 0.0),
            _ => (0.0, 68.0, 0.0),
        };

        let server_clone = server.clone();
        tokio::spawn(async move {
            server_clone.change_player_dimension(target_addr, dim, tx, ty, tz).await;
        });

        let dim_name = match dim {
            1 => "Nether",
            2 => "The End",
            _ => "Overworld",
        };
        sender.send_message(&format!("Teleported {} to dimension {} ({})", target_name, dim, dim_name));
    }
}
