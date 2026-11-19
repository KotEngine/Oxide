use super::CommandSender;
use crate::server::OxideServer;

pub trait Command: Send + Sync {
    fn name(&self) -> &str;
    fn aliases(&self) -> Vec<&str> { Vec::new() }
    fn description(&self) -> &str { "" }
    fn usage(&self) -> &str { "" }
    fn permission(&self) -> Option<&str> { None }
    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>);
}
