use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use super::{Command, CommandSender};
use crate::server::OxideServer;

pub struct CommandManager {
    commands: RwLock<HashMap<String, Arc<dyn Command>>>,
}

impl CommandManager {
    pub fn new() -> Self {
        let mgr = Self { commands: RwLock::new(HashMap::new()) };
        mgr.register_defaults();
        mgr
    }

    pub fn register(&self, command: Arc<dyn Command>) {
        let name = command.name().to_lowercase();
        self.commands.write().insert(name.clone(), command.clone());
        for alias in command.aliases() {
            self.commands.write().insert(alias.to_lowercase(), command.clone());
        }
    }

    pub fn dispatch(&self, sender: &mut dyn CommandSender, server: &OxideServer, input: &str) -> bool {
        let input = input.trim();
        if input.is_empty() { return false; }
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() { return false; }
        let name = parts[0].to_lowercase();
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
        let cmd = self.commands.read().get(&name).cloned();
        match cmd {
            Some(command) => {
                command.execute(sender, server, args);
                true
            }
            None => false,
        }
    }

    pub fn count(&self) -> usize {
        self.commands.read().len()
    }

    pub fn all_commands(&self) -> Vec<Arc<dyn Command>> {
        let mut seen = std::collections::HashSet::new();
        let mut list = Vec::new();
        for cmd in self.commands.read().values() {
            let name = cmd.name().to_string();
            if seen.insert(name) {
                list.push(cmd.clone());
            }
        }
        list.sort_by(|a, b| a.name().cmp(b.name()));
        list
    }

    pub fn register_defaults(&self) {
        use super::defaults::*;
        self.register(Arc::new(GamemodeCommand));
        self.register(Arc::new(TpCommand));
        self.register(Arc::new(TimeCommand));
        self.register(Arc::new(StopCommand));
        self.register(Arc::new(ListCommand));
        self.register(Arc::new(GiveCommand));
        self.register(Arc::new(OpCommand));
        self.register(Arc::new(DeopCommand));
        self.register(Arc::new(KickCommand));
        self.register(Arc::new(KillCommand));
        self.register(Arc::new(SayCommand));
        self.register(Arc::new(MeCommand));
        self.register(Arc::new(TellCommand));
        self.register(Arc::new(HelpCommand));
        self.register(Arc::new(StatusCommand));
        self.register(Arc::new(VersionCommand));
        self.register(Arc::new(SeedCommand));
        self.register(Arc::new(DifficultyCommand));
        self.register(Arc::new(SpawnpointCommand));
        self.register(Arc::new(SetworldspawnCommand));
    }
}

impl Default for CommandManager {
    fn default() -> Self { Self::new() }
}
