use crate::command::{Command, CommandSender};
use crate::server::OxideServer;

pub struct StopCommand;

impl Command for StopCommand {
    fn name(&self) -> &str { "stop" }
    fn description(&self) -> &str { "Stops the server" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.stop") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, _args: Vec<String>) {
        sender.send_message("Stopping the server and saving data...");
        server.broadcast_message("Server is shutting down...");
        server.request_shutdown();
    }
}
