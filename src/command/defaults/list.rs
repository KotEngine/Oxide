use crate::command::{Command, CommandSender};
use crate::server::OxideServer;

pub struct ListCommand;

impl Command for ListCommand {
    fn name(&self) -> &str { "list" }
    fn description(&self) -> &str { "Lists online players" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.list") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, _args: Vec<String>) {
        let max_players = server.config.read().get_int("max_players");
        let players = server.players.read();
        let names: Vec<String> = players.values().filter(|s| s.logged_in).map(|s| s.username.clone()).collect();
        sender.send_message(&format!("There are {}/{} players online: {}", names.len(), max_players, names.join(", ")));
    }
}
