use crate::command::{Command, CommandSender};
use crate::server::OxideServer;

pub struct TimeCommand;

impl Command for TimeCommand {
    fn name(&self) -> &str { "time" }
    fn description(&self) -> &str { "Changes the world time" }
    fn usage(&self) -> &str { "/time set <value> | /time add <value> | /time query" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.time") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>) {
        if args.is_empty() {
            sender.send_message("Usage: /time set <value> | /time add <value> | /time query");
            return;
        }
        let action = args[0].to_lowercase();
        match action.as_str() {
            "set" => {
                if args.len() < 2 {
                    sender.send_message("Usage: /time set <day|night|noon|midnight|value>");
                    return;
                }
                let val = match args[1].to_lowercase().as_str() {
                    "day" => 1000,
                    "noon" => 6000,
                    "sunset" => 12000,
                    "night" => 14000,
                    "midnight" => 18000,
                    "sunrise" => 23000,
                    other => other.parse::<i32>().unwrap_or(0),
                };
                server.set_time(val);
                sender.send_message(&format!("Set time to {}", val));
            }
            "add" => {
                if args.len() < 2 {
                    sender.send_message("Usage: /time add <value>");
                    return;
                }
                let val = args[1].parse::<i32>().unwrap_or(0);
                server.add_time(val);
                sender.send_message(&format!("Added {} to time", val));
            }
            "query" => {
                let t = server.world.read().time;
                sender.send_message(&format!("Current time is {}", t));
            }
            _ => sender.send_message("Usage: /time set <value> | /time add <value> | /time query"),
        }
    }
}
