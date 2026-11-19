use std::collections::HashMap;
use parking_lot::RwLock;
use super::{Plugin, PluginDescription, PluginType};

struct SimplePlugin {
    desc: PluginDescription,
}

impl Plugin for SimplePlugin {
    fn on_enable(&mut self) {
        tracing::info!("Enabling plugin {} v{}", self.desc.name, self.desc.version);
    }
    fn on_disable(&mut self) {
        tracing::info!("Disabling plugin {}", self.desc.name);
    }
    fn description(&self) -> &PluginDescription {
        &self.desc
    }
}

pub struct PluginManager {
    plugins: RwLock<HashMap<String, Box<dyn Plugin>>>,
    descriptions: RwLock<HashMap<String, PluginDescription>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            descriptions: RwLock::new(HashMap::new()),
        }
    }

    pub fn enable(&self, mut plugin: Box<dyn Plugin>) {
        let desc = plugin.description().clone();
        plugin.on_enable();
        let name = desc.name.clone();
        self.descriptions.write().insert(name.clone(), desc);
        self.plugins.write().insert(name, plugin);
    }

    pub fn disable(&self, name: &str) {
        let mut plugins = self.plugins.write();
        if let Some(mut plugin) = plugins.remove(name) {
            plugin.on_disable();
        }
        self.descriptions.write().remove(name);
    }

    pub fn count(&self) -> usize {
        self.plugins.read().len()
    }

    pub fn list(&self) -> Vec<String> {
        self.plugins.read().keys().cloned().collect()
    }

    pub fn load_directory(&self, path: &str) -> usize {
        let p = std::path::Path::new(path);
        if !p.exists() {
            let _ = std::fs::create_dir_all(p);
            return 0;
        }

        let mut count = 0;
        if let Ok(entries) = std::fs::read_dir(p) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    let json_manifest = entry_path.join("plugin.json");
                    let yml_manifest = entry_path.join("plugin.yml");
                    let desc_opt = if json_manifest.exists() {
                        if let Ok(content) = std::fs::read_to_string(&json_manifest) {
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                                let name = val.get("name").and_then(|v| v.as_str()).unwrap_or("Unnamed").to_string();
                                let version = val.get("version").and_then(|v| v.as_str()).unwrap_or("1.0.0").to_string();
                                let author = val.get("author").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
                                let description = val.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                let main = val.get("main").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                Some(PluginDescription {
                                    name,
                                    version,
                                    author,
                                    description,
                                    main,
                                    api_version: "0.1.0".to_string(),
                                    plugin_type: PluginType::Lua,
                                    depends: Vec::new(),
                                })
                            } else { None }
                        } else { None }
                    } else if yml_manifest.exists() {
                        if let Ok(content) = std::fs::read_to_string(&yml_manifest) {
                            let mut name = "Unnamed".to_string();
                            let mut version = "1.0.0".to_string();
                            let mut author = "Unknown".to_string();
                            let mut description = String::new();
                            let mut main = String::new();
                            for line in content.lines() {
                                let line = line.trim();
                                if let Some((k, v)) = line.split_once(':') {
                                    let k = k.trim();
                                    let v = v.trim().trim_matches('"').trim_matches('\'');
                                    match k {
                                        "name" => name = v.to_string(),
                                        "version" => version = v.to_string(),
                                        "author" => author = v.to_string(),
                                        "description" => description = v.to_string(),
                                        "main" => main = v.to_string(),
                                        _ => {}
                                    }
                                }
                            }
                            Some(PluginDescription {
                                name,
                                version,
                                author,
                                description,
                                main,
                                api_version: "0.1.0".to_string(),
                                plugin_type: PluginType::Lua,
                                depends: Vec::new(),
                            })
                        } else { None }
                    } else { None };

                    if let Some(desc) = desc_opt {
                        let plugin_name = desc.name.clone();
                        self.enable(Box::new(SimplePlugin { desc }));
                        tracing::info!("Loaded plugin: {}", plugin_name);
                        count += 1;
                    }
                }
            }
        }
        count
    }
}

impl Default for PluginManager {
    fn default() -> Self { Self::new() }
}
