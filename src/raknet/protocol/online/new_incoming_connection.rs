use crate::utils::BinaryStream;
use crate::raknet::protocol::{constants::SYSTEM_ADDRESS_COUNT, message_ids::MessageIdentifiers, packet::{write_address, read_address}};
use crate::raknet::InternetAddress;
use std::net::Ipv4Addr;

pub struct NewIncomingConnection {
    pub address: Option<InternetAddress>,
    pub system_addresses: Vec<InternetAddress>,
    pub send_ping_time: i64,
    pub send_pong_time: i64,
}

impl NewIncomingConnection {
    pub const ID: u8 = MessageIdentifiers::ID_NEW_INCOMING_CONNECTION;

    pub fn new(address: InternetAddress, send_ping_time: i64, send_pong_time: i64) -> Self {
        Self {
            address: Some(address),
            system_addresses: Vec::new(),
            send_ping_time,
            send_pong_time,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut stream = BinaryStream::new();
        stream.put_byte(Self::ID);
        if let Some(addr) = &self.address {
            write_address(&mut stream, addr);
        }
        for addr in &self.system_addresses {
            write_address(&mut stream, addr);
        }
        stream.put_long(self.send_ping_time);
        stream.put_long(self.send_pong_time);
        stream.into_inner()
    }

    pub fn decode(buffer: Vec<u8>) -> Self {
        let mut stream = BinaryStream::from_bytes(buffer);
        let _ = stream.get_byte();
        let address = read_address(&mut stream);
        let mut system_addresses = Vec::new();
        let stop_offset = stream.len().saturating_sub(16);
        let dummy = InternetAddress::new(std::net::IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0, 4);
        for _ in 0..SYSTEM_ADDRESS_COUNT {
            if stream.offset >= stop_offset {
                system_addresses.push(dummy.clone());
            } else if let Some(addr) = read_address(&mut stream) {
                system_addresses.push(addr);
            }
        }
        let send_ping_time = stream.get_long().unwrap_or(0);
        let send_pong_time = stream.get_long().unwrap_or(0);
        Self { address, system_addresses, send_ping_time, send_pong_time }
    }
}
