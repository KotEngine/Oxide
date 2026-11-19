use crate::command::{Command, CommandSender};
use crate::server::OxideServer;

pub struct OpCommand;

impl Command for OpCommand {
    fn name(&self) -> &str { "op" }
    fn description(&self) -> &str { "Gives operator status" }
    fn usage(&self) -> &str { "/op <player>" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.op") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>) {
        if args.is_empty() {
            sender.send_message("Usage: /op <player>");
            return;
        }
        if let Some((addr, actual_name)) = server.find_player_by_name(&args[0]) {
            server.send_message_to(addr, "You are now an operator!");
            sender.send_message(&format!("Opped {}", actual_name));
        } else {
            sender.send_message(&format!("Player '{}' not found", args[0]));
        }
    }
}

pub struct DeopCommand;

impl Command for DeopCommand {
    fn name(&self) -> &str { "deop" }
    fn description(&self) -> &str { "Removes operator status" }
    fn usage(&self) -> &str { "/deop <player>" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.op") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>) {
        if args.is_empty() {
            sender.send_message("Usage: /deop <player>");
            return;
        }
        if let Some((addr, actual_name)) = server.find_player_by_name(&args[0]) {
            server.send_message_to(addr, "You are no longer an operator!");
            sender.send_message(&format!("De-opped {}", actual_name));
        } else {
            sender.send_message(&format!("Player '{}' not found", args[0]));
        }
    }
}
