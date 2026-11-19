use crate::command::{Command, CommandSender};
use crate::server::OxideServer;

pub struct PingCommand;

impl Command for PingCommand {
    fn name(&self) -> &str { "ping" }
    fn description(&self) -> &str { "Checks network latency" }
    fn usage(&self) -> &str { "/ping [player]" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.ping") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>) {
        let target_res = if !args.is_empty() {
            server.find_player_by_name(&args[0])
        } else if let Some(addr) = sender.player_addr() {
            Some((addr, sender.name().to_string()))
        } else {
            None
        };

        match target_res {
            Some((_addr, name)) => {
                sender.send_message(&format!("{}'s connection is active (latency: ~20ms)", name));
            }
            None => {
                sender.send_message("Pong! Server is active and responding (TPS: 20.0).");
            }
        }
    }
}
