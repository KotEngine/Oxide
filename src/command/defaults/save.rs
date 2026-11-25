use crate::command::{Command, CommandSender};
use crate::server::OxideServer;

pub struct SaveAllCommand;

impl Command for SaveAllCommand {
    fn name(&self) -> &str { "save-all" }
    fn aliases(&self) -> Vec<&str> { vec!["save"] }
    fn description(&self) -> &str { "Saves the server to disk" }
    fn usage(&self) -> &str { "/save-all" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.save") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, _args: Vec<String>) {
        sender.send_message("Saving...");
        let saved_chunks = server.save_all();
        sender.send_message(&format!("Saved world, players, and data ({} chunks flushed).", saved_chunks));
    }
}
