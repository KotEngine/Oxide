use std::net::SocketAddr;
use crate::server::OxideServer;

pub trait CommandSender: Send + Sync {
    fn name(&self) -> &str;
    fn send_message(&mut self, message: &str);
    fn is_op(&self) -> bool { false }
    fn has_permission(&self, _perm: &str) -> bool { false }
    fn player_addr(&self) -> Option<SocketAddr> { None }
}

pub struct ConsoleSender {
    pub name: String,
}

impl ConsoleSender {
    pub fn new() -> Self {
        Self { name: "Console".to_string() }
    }
}

impl Default for ConsoleSender {
    fn default() -> Self { Self::new() }
}

impl CommandSender for ConsoleSender {
    fn name(&self) -> &str { &self.name }
    fn send_message(&mut self, message: &str) {
        tracing::info!("[CONSOLE] {}", message);
    }
    fn is_op(&self) -> bool { true }
    fn has_permission(&self, _perm: &str) -> bool { true }
    fn player_addr(&self) -> Option<SocketAddr> { None }
}

pub struct PlayerSender {
    pub name: String,
    pub addr: SocketAddr,
    pub is_op: bool,
    server: OxideServer,
}

impl PlayerSender {
    pub fn new(name: String, addr: SocketAddr, is_op: bool, server: OxideServer) -> Self {
        Self { name, addr, is_op, server }
    }
}

impl CommandSender for PlayerSender {
    fn name(&self) -> &str { &self.name }
    fn send_message(&mut self, message: &str) {
        self.server.send_message_to(self.addr, message);
    }
    fn is_op(&self) -> bool { self.is_op }
    fn has_permission(&self, _perm: &str) -> bool { true }
    fn player_addr(&self) -> Option<SocketAddr> { Some(self.addr) }
}
