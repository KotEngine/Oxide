#[derive(Debug, Clone)]
pub struct PluginDescription {
    pub name: String,
    pub version: String,
    pub main: String,
    pub author: String,
    pub description: String,
    pub api_version: String,
    pub plugin_type: PluginType,
    pub depends: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginType {
    Lua,
    Native,
}

impl PluginDescription {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            main: String::new(),
            author: String::new(),
            description: String::new(),
            api_version: "0.1.0".to_string(),
            plugin_type: PluginType::Lua,
            depends: Vec::new(),
        }
    }
}

pub trait Plugin: Send + Sync {
    fn on_enable(&mut self);
    fn on_disable(&mut self);
    fn description(&self) -> &PluginDescription;
}
