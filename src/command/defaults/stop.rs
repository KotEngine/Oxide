use crate::command::{Command, CommandSender};
use crate::server::OxideServer;

pub struct StopCommand;

impl Command for StopCommand {
    fn name(&self) -> &str { "stop" }
    fn description(&self) -> &str { "Stops the server" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.stop") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, _args: Vec<String>) {
        sender.send_message("Stopping the server...");
        server.broadcast_message("Server is shutting down...");
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
            std::process::exit(0);
        });
    }
}
