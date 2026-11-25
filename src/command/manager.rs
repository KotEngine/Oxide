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
        self.register(Arc::new(WeatherCommand));
        self.register(Arc::new(XpCommand));
        self.register(Arc::new(EffectCommand));
        self.register(Arc::new(EnchantCommand));
        self.register(Arc::new(ClearCommand));
        self.register(Arc::new(SetBlockCommand));
        self.register(Arc::new(FillCommand));
        self.register(Arc::new(TitleCommand));
        self.register(Arc::new(ParticleCommand));
        self.register(Arc::new(PlaysoundCommand));
        self.register(Arc::new(PingCommand));
        self.register(Arc::new(DefaultGamemodeCommand));
        self.register(Arc::new(BanCommand));
        self.register(Arc::new(BanIpCommand));
        self.register(Arc::new(PardonCommand));
        self.register(Arc::new(PardonIpCommand));
        self.register(Arc::new(BanListCommand));
        self.register(Arc::new(WhitelistCommand));
        self.register(Arc::new(PluginsCommand));
        self.register(Arc::new(ReloadCommand));
        self.register(Arc::new(SummonCommand));
        self.register(Arc::new(DimensionCommand));
        self.register(Arc::new(SaveAllCommand));
    }

    pub fn generate_available_commands_packet(&self) -> crate::protocol::packets::AvailableCommandsPacket {
        let mut map = serde_json::Map::new();
        for cmd in self.all_commands() {
            let mut ver = serde_json::Map::new();
            ver.insert("description".to_string(), serde_json::Value::String(cmd.description().to_string()));
            let aliases: Vec<serde_json::Value> = cmd.aliases().iter().map(|a| serde_json::Value::String(a.to_string())).collect();
            ver.insert("aliases".to_string(), serde_json::Value::Array(aliases));
            ver.insert("permission".to_string(), serde_json::Value::String("any".to_string()));

            let mut overload_input = serde_json::Map::new();
            let mut param = serde_json::Map::new();
            param.insert("name".to_string(), serde_json::Value::String("args".to_string()));
            param.insert("type".to_string(), serde_json::Value::String("rawtext".to_string()));
            param.insert("optional".to_string(), serde_json::Value::Bool(true));
            overload_input.insert("parameters".to_string(), serde_json::Value::Array(vec![serde_json::Value::Object(param)]));

            let mut def = serde_json::Map::new();
            def.insert("input".to_string(), serde_json::Value::Object(overload_input));
            def.insert("output".to_string(), serde_json::Value::Object(serde_json::Map::new()));

            let mut overloads = serde_json::Map::new();
            overloads.insert("default".to_string(), serde_json::Value::Object(def));
            ver.insert("overloads".to_string(), serde_json::Value::Object(overloads));

            let mut cmd_obj = serde_json::Map::new();
            cmd_obj.insert("versions".to_string(), serde_json::Value::Array(vec![serde_json::Value::Object(ver)]));

            map.insert(cmd.name().to_string(), serde_json::Value::Object(cmd_obj));
        }
        let json_str = serde_json::Value::Object(map).to_string();
        crate::protocol::packets::AvailableCommandsPacket {
            commands: json_str,
            unknown: String::new(),
        }
    }
}

impl Default for CommandManager {
    fn default() -> Self { Self::new() }
}
