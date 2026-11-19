use std::collections::HashMap;
use parking_lot::RwLock;
use super::Permission;

pub struct PermissionManager {
    permissions: RwLock<HashMap<String, Permission>>,
}

impl PermissionManager {
    pub fn new() -> Self {
        Self { permissions: RwLock::new(HashMap::new()) }
    }

    pub fn add(&self, perm: Permission) {
        self.permissions.write().insert(perm.name.clone(), perm);
    }

    pub fn get(&self, name: &str) -> Option<Permission> {
        self.permissions.read().get(name).cloned()
    }

    pub fn count(&self) -> usize {
        self.permissions.read().len()
    }
}

impl Default for PermissionManager {
    fn default() -> Self { Self::new() }
}
