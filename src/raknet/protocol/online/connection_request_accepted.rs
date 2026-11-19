use crate::utils::BinaryStream;
use crate::raknet::protocol::{constants::SYSTEM_ADDRESS_COUNT, message_ids::MessageIdentifiers, packet::{write_address, read_address}};
use crate::raknet::InternetAddress;
use std::net::Ipv4Addr;

pub struct ConnectionRequestAccepted {
    pub address: Option<InternetAddress>,
    pub system_addresses: Vec<InternetAddress>,
    pub send_ping_time: i64,
    pub send_pong_time: i64,
}

impl ConnectionRequestAccepted {
    pub const ID: u8 = MessageIdentifiers::ID_CONNECTION_REQUEST_ACCEPTED;

    pub fn new(address: InternetAddress, send_ping_time: i64, send_pong_time: i64) -> Self {
        Self {
            address: Some(address),
            system_addresses: vec![InternetAddress::new(
                std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0, 4,
            )],
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
        stream.put_short(0); // system_index (big-endian short)
        let dummy = InternetAddress::new(std::net::IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0, 4);
        for i in 0..SYSTEM_ADDRESS_COUNT {
            let addr = self.system_addresses.get(i).unwrap_or(&dummy);
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
        let _ = stream.get_short();
        let mut system_addresses = Vec::new();
        let len = stream.len();
        for _ in 0..SYSTEM_ADDRESS_COUNT {
            if stream.offset + 16 < len {
                if let Some(addr) = read_address(&mut stream) {
                    system_addresses.push(addr);
                }
            }
        }
        let send_ping_time = stream.get_long().unwrap_or(0);
        let send_pong_time = stream.get_long().unwrap_or(0);
        Self { address, system_addresses, send_ping_time, send_pong_time }
    }
}
