use crate::utils::{BinaryStream, Binary};
use super::constants::MAGIC;

pub trait RakNetPacket: Send + Sync {
    const ID: u8;

    fn encode(&self) -> Vec<u8>;
    fn decode(buffer: Vec<u8>) -> Self where Self: Sized;
}

pub struct OfflineMessage;

impl OfflineMessage {
    pub fn write_magic(stream: &mut BinaryStream) {
        stream.put(&MAGIC);
    }

    pub fn read_magic(stream: &mut BinaryStream) -> [u8; 16] {
        let mut magic = [0u8; 16];
        if let Ok(bytes) = stream.get(16) {
            magic.copy_from_slice(&bytes);
        }
        magic
    }

    pub fn is_valid_magic(magic: &[u8; 16]) -> bool {
        magic == &MAGIC
    }
}

pub fn write_address(stream: &mut BinaryStream, addr: &crate::raknet::InternetAddress) {
    match addr.ip {
        std::net::IpAddr::V4(v4) => {
            stream.put_byte(4);
            let octets = v4.octets();
            for o in octets {
                stream.put_byte(!o);
            }
            stream.put_lshort(addr.port);
        }
        std::net::IpAddr::V6(_v6) => {
            stream.put_byte(6);
            stream.put_byte(0);
            stream.put_lshort(addr.port);
            stream.put_byte(0);
        }
    }
}

pub fn read_address(stream: &mut BinaryStream) -> Option<crate::raknet::InternetAddress> {
    let version = stream.get_byte().ok()?;
    match version {
        4 => {
            let mut octets = [0u8; 4];
            for i in 0..4 {
                octets[i] = !stream.get_byte().ok()?;
            }
            let port = stream.get_lshort().ok()?;
            let ip = std::net::Ipv4Addr::from(octets);
            Some(crate::raknet::InternetAddress::new(
                std::net::IpAddr::V4(ip), port, 4,
            ))
        }
        6 => {
            let _ = stream.get_lshort();
            let port = stream.get_lshort().ok()?;
            let _ = stream.get_lshort();
            let _ = stream.get_byte();
            let _ = stream.get(16);
            let _ = stream.get_lshort();
            Some(crate::raknet::InternetAddress::new(
                std::net::IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED), port, 6,
            ))
        }
        _ => None,
    }
}
