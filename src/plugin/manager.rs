use std::collections::HashMap;
use parking_lot::RwLock;
use super::{Plugin, PluginDescription};

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

    pub fn load_directory(&self, _path: &str) -> usize {
        0
    }
}

impl Default for PluginManager {
    fn default() -> Self { Self::new() }
}
