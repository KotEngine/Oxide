use crate::command::{Command, CommandSender};
use crate::server::OxideServer;

pub struct WhitelistCommand;

impl Command for WhitelistCommand {
    fn name(&self) -> &str { "whitelist" }
    fn description(&self) -> &str { "Manages the server whitelist" }
    fn usage(&self) -> &str { "/whitelist <on|off|list|add|remove|reload>" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.whitelist") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>) {
        if args.is_empty() {
            sender.send_message("Usage: /whitelist <on|off|list|add|remove|reload>");
            return;
        }

        match args[0].to_lowercase().as_str() {
            "on" => {
                server.whitelist_enabled.store(true, std::sync::atomic::Ordering::SeqCst);
                sender.send_message("Whitelist is now turned on");
            }
            "off" => {
                server.whitelist_enabled.store(false, std::sync::atomic::Ordering::SeqCst);
                sender.send_message("Whitelist is now turned off");
            }
            "list" => {
                let guard = server.whitelist.read();
                sender.send_message(&format!("There are {} whitelisted players:", guard.len()));
                for name in guard.iter() {
                    sender.send_message(&format!("- {}", name));
                }
            }
            "add" => {
                if args.len() < 2 {
                    sender.send_message("Usage: /whitelist add <player>");
                    return;
                }
                let name = args[1].to_lowercase();
                server.whitelist.write().insert(name);
                sender.send_message(&format!("Added {} to the whitelist", args[1]));
            }
            "remove" => {
                if args.len() < 2 {
                    sender.send_message("Usage: /whitelist remove <player>");
                    return;
                }
                let name = args[1].to_lowercase();
                if server.whitelist.write().remove(&name) {
                    sender.send_message(&format!("Removed {} from the whitelist", args[1]));
                } else {
                    sender.send_message(&format!("Player '{}' is not whitelisted", args[1]));
                }
            }
            "reload" => {
                sender.send_message("Whitelist reloaded");
            }
            _ => {
                sender.send_message("Usage: /whitelist <on|off|list|add|remove|reload>");
            }
        }
    }
}
