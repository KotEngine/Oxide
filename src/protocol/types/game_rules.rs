use std::collections::BTreeMap;

pub const TYPE_BOOL: u32 = 1;
pub const TYPE_INT: u32 = 2;
pub const TYPE_FLOAT: u32 = 3;

#[derive(Debug, Clone, PartialEq)]
pub enum GameRuleValue {
    Bool(bool),
    Int(u32),
    Float(f32),
}

#[derive(Debug, Clone, Default)]
pub struct GameRules {
    pub rules: BTreeMap<String, (u32, GameRuleValue)>,
}

impl GameRules {
    pub fn new() -> Self { Self { rules: BTreeMap::new() } }

    pub fn set_bool(&mut self, name: &str, value: bool) {
        self.rules.insert(name.to_string(), (TYPE_BOOL, GameRuleValue::Bool(value)));
    }
    pub fn set_int(&mut self, name: &str, value: u32) {
        self.rules.insert(name.to_string(), (TYPE_INT, GameRuleValue::Int(value)));
    }
    pub fn set_float(&mut self, name: &str, value: f32) {
        self.rules.insert(name.to_string(), (TYPE_FLOAT, GameRuleValue::Float(value)));
    }

    pub fn default_rules() -> Self {
        let mut rules = Self::new();
        rules.set_bool("doDaylightCycle", true);
        rules.set_bool("doFireTick", true);
        rules.set_bool("doMobSpawning", true);
        rules.set_bool("doTileDrops", true);
        rules.set_bool("doWeatherCycle", true);
        rules.set_bool("keepInventory", false);
        rules.set_bool("mobGriefing", true);
        rules.set_bool("naturalRegeneration", true);
        rules.set_bool("pvp", true);
        rules.set_bool("sendCommandFeedback", true);
        rules.set_bool("showcoordinates", false);
        rules.set_bool("tntexplodes", true);
        rules
    }

    pub fn put(&self, stream: &mut crate::utils::BinaryStream) {
        stream.put_unsigned_var_int(self.rules.len() as u32);
        for (name, (ty, value)) in &self.rules {
            stream.put_string(name);
            stream.put_unsigned_var_int(*ty);
            match value {
                GameRuleValue::Bool(v) => stream.put_bool(*v),
                GameRuleValue::Int(v) => stream.put_unsigned_var_int(*v),
                GameRuleValue::Float(v) => stream.put_lfloat(*v),
            }
        }
    }

    pub fn get(stream: &mut crate::utils::BinaryStream) -> Self {
        let count = stream.get_unsigned_var_int().unwrap_or(0);
        let mut rules = BTreeMap::new();
        for _ in 0..count {
            let name = stream.get_string().unwrap_or_default();
            let ty = stream.get_unsigned_var_int().unwrap_or(0);
            let value = match ty {
                TYPE_BOOL => GameRuleValue::Bool(stream.get_bool().unwrap_or(false)),
                TYPE_INT => GameRuleValue::Int(stream.get_unsigned_var_int().unwrap_or(0)),
                TYPE_FLOAT => GameRuleValue::Float(stream.get_lfloat().unwrap_or(0.0)),
                _ => break,
            };
            rules.insert(name, (ty, value));
        }
        Self { rules }
    }
}
