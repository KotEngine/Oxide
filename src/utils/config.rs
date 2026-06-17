use std::path::Path;
use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Config {
    pub data: HashMap<String, serde_json::Value>,
}

impl Default for Config {
    fn default() -> Self { Self { data: HashMap::new() } }
}

impl Config {
    pub fn new() -> Self { Self::default() }

    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let data: HashMap<String, serde_json::Value> = serde_json::from_str(&content)
            .unwrap_or_default();
        Ok(Self { data })
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(&self.data)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }

    pub fn get_string(&self, key: &str) -> String {
        self.data.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default()
    }

    pub fn get_int(&self, key: &str) -> i64 {
        self.data.get(key)
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
    }

    pub fn get_bool(&self, key: &str) -> bool {
        self.data.get(key)
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    pub fn get_float(&self, key: &str) -> f64 {
        self.data.get(key)
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
    }

    pub fn set(&mut self, key: &str, value: serde_json::Value) {
        self.data.insert(key.to_string(), value);
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_motd")]
    pub motd: String,
    #[serde(default = "default_sub_motd")]
    pub sub_motd: String,
    #[serde(default = "default_max_players")]
    pub max_players: u32,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_bind")]
    pub bind_ip: String,
    #[serde(default = "default_view_distance")]
    pub view_distance: u32,
    #[serde(default = "default_white")]
    pub white_list: bool,
    #[serde(default)]
    pub op: Vec<String>,
}

fn default_motd() -> String { "Oxide Server".to_string() }
fn default_sub_motd() -> String { "Powered by Oxide".to_string() }
fn default_max_players() -> u32 { 20 }
fn default_port() -> u16 { 19132 }
fn default_bind() -> String { "0.0.0.0".to_string() }
fn default_view_distance() -> u32 { 8 }
fn default_white() -> bool { false }

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            motd: default_motd(),
            sub_motd: default_sub_motd(),
            max_players: default_max_players(),
            port: default_port(),
            bind_ip: default_bind(),
            view_distance: default_view_distance(),
            white_list: default_white(),
            op: Vec::new(),
        }
    }
}
