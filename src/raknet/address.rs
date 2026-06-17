use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InternetAddress {
    pub ip: IpAddr,
    pub port: u16,
    pub version: u8,
}

impl InternetAddress {
    pub fn new(ip: IpAddr, port: u16, version: u8) -> Self {
        Self { ip, port, version }
    }

    pub fn from_v4(ip: &str, port: u16) -> Self {
        Self {
            ip: IpAddr::V4(ip.parse().unwrap_or(Ipv4Addr::UNSPECIFIED)),
            port,
            version: 4,
        }
    }

    pub fn to_socket_addr(&self) -> std::net::SocketAddr {
        std::net::SocketAddr::new(self.ip, self.port)
    }

    pub fn get_ip(&self) -> String {
        self.ip.to_string()
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub fn get_version(&self) -> u8 {
        self.version
    }

    pub fn equals(&self, other: &Self) -> bool {
        self.ip == other.ip && self.port == other.port && self.version == other.version
    }
}

impl std::fmt::Display for InternetAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.ip, self.port)
    }
}
