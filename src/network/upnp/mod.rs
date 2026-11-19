use std::net::SocketAddr;

pub struct UpnpManager;

impl UpnpManager {
    pub fn new() -> Self { Self }

    pub async fn add_port_mapping(&self, _internal_addr: SocketAddr, _external_port: u16, _protocol: &str) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn remove_port_mapping(&self, _external_port: u16, _protocol: &str) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn discover_external_ip(&self) -> Option<String> { None }
}

impl Default for UpnpManager {
    fn default() -> Self { Self::new() }
}
