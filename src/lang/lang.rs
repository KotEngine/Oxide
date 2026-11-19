use std::collections::HashMap;
use parking_lot::RwLock;

pub struct Language {
    translations: RwLock<HashMap<String, String>>,
    code: String,
}

impl Language {
    pub fn new(code: impl Into<String>) -> Self {
        Self { translations: RwLock::new(HashMap::new()), code: code.into() }
    }

    pub fn translate(&self, key: &str) -> String {
        self.translations.read().get(key).cloned().unwrap_or_else(|| key.to_string())
    }

    pub fn translate_with_args(&self, key: &str, args: &HashMap<String, String>) -> String {
        let template = self.translate(key);
        let mut result = template;
        for (k, v) in args {
            result = result.replace(&format!("{{{}}}", k), v);
        }
        result
    }

    pub fn add(&self, key: impl Into<String>, value: impl Into<String>) {
        self.translations.write().insert(key.into(), value.into());
    }

    pub fn load_defaults(&self) {
        let defaults = [
            ("chat.type.text", "<{0}> {1}"),
            ("multiplayer.player.joined", "{0} joined the game"),
            ("multiplayer.player.left", "{0} left the game"),
            ("commands.generic.permission", "You don't have permission"),
            ("commands.generic.notFound", "Unknown command"),
            ("commands.generic.player.notFound", "Player not found"),
            ("commands.gamemode.success.self", "Set own game mode to {0}"),
            ("commands.gamemode.success.other", "Set {0}'s game mode to {1}"),
            ("commands.give.success", "Gave {0} to {1}"),
            ("commands.tp.success", "Teleported {0} to {1}"),
            ("commands.time.set", "Set time to {0}"),
            ("commands.kill.success", "Killed {0}"),
            ("commands.op.success", "Made {0} a server operator"),
            ("commands.deop.success", "Removed {0} as server operator"),
            ("commands.kick.success", "Kicked {0}"),
            ("commands.ban.success", "Banned {0}"),
            ("commands.pardon.success", "Unbanned {0}"),
            ("commands.whitelist.add.success", "Added {0} to whitelist"),
            ("commands.whitelist.remove.success", "Removed {0} from whitelist"),
            ("commands.weather.clear", "Changing to clear weather"),
            ("commands.weather.rain", "Changing to rainy weather"),
            ("commands.weather.thunder", "Changing to thunderstorm"),
            ("commands.difficulty.success", "Set difficulty to {0}"),
            ("commands.seed.success", "Seed: {0}"),
            ("commands.list.players", "There are {0}/{1} players online"),
            ("commands.stop.start", "Stopping the server"),
            ("commands.save.start", "Saving the game"),
            ("commands.save.success", "Saved the game"),
        ];
        let mut t = self.translations.write();
        for (k, v) in defaults {
            t.insert(k.to_string(), v.to_string());
        }
    }

    pub fn code(&self) -> &str { &self.code }
}

impl Default for Language {
    fn default() -> Self {
        let l = Self::new("en_US");
        l.load_defaults();
        l
    }
}
