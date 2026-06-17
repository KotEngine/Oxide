use crate::utils::BinaryStream;
use super::encapsulated::EncapsulatedPacket;
use super::message_ids::MessageIdentifiers;

pub const BITFLAG_VALID: u8 = 0x80;
pub const BITFLAG_ACK: u8 = 0x40;
pub const BITFLAG_NAK: u8 = 0x20;
pub const BITFLAG_PACKET_PAIR: u8 = 0x10;
pub const BITFLAG_CONTINUOUS_SEND: u8 = 0x08;
pub const BITFLAG_NEEDS_B_AND_AS: u8 = 0x04;

pub struct Datagram {
    pub header_flags: u8,
    pub packets: Vec<EncapsulatedPacket>,
    pub seq_number: Option<u32>,
    pub raw_packets: Vec<Vec<u8>>,
}

impl Default for Datagram {
    fn default() -> Self { Self::new() }
}

impl Datagram {
    pub fn new() -> Self {
        Self { header_flags: 0, packets: Vec::new(), seq_number: None, raw_packets: Vec::new() }
    }

    pub fn is_datagram(first_byte: u8) -> bool {
        (first_byte & BITFLAG_VALID) != 0 && (first_byte & (BITFLAG_ACK | BITFLAG_NAK)) == 0
    }

    pub fn is_ack(first_byte: u8) -> bool {
        (first_byte & BITFLAG_VALID) != 0 && (first_byte & BITFLAG_ACK) != 0
    }

    pub fn is_nak(first_byte: u8) -> bool {
        (first_byte & BITFLAG_VALID) != 0 && (first_byte & BITFLAG_NAK) != 0
    }

    pub fn is_online_packet(first_byte: u8) -> bool {
        first_byte < MessageIdentifiers::ID_USER_PACKET_ENUM
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut stream = BinaryStream::new();
        stream.put_byte(BITFLAG_VALID | self.header_flags);
        if let Some(seq) = self.seq_number {
            stream.put_ltriad(seq);
        } else {
            stream.put_ltriad(0);
        }
        for pkt in &self.packets {
            let bin = pkt.to_binary();
            stream.put(&bin);
        }
        for raw in &self.raw_packets {
            stream.put(raw);
        }
        stream.into_inner()
    }

    pub fn decode(buffer: Vec<u8>) -> Self {
        let mut stream = BinaryStream::from_bytes(buffer);
        let header_flags = stream.get_byte().unwrap_or(BITFLAG_VALID);
        let seq_number = stream.get_ltriad().ok();

        let mut packets = Vec::new();
        while !stream.eof() {
            let remaining = &stream.buffer[stream.offset..];
            if remaining.is_empty() { break; }
            let (pkt, consumed) = EncapsulatedPacket::from_binary(remaining);
            if consumed == 0 || pkt.buffer.is_empty() { break; }
            stream.offset += consumed;
            packets.push(pkt);
        }

        Self { header_flags, packets, seq_number, raw_packets: Vec::new() }
    }

    pub fn length(&self) -> usize {
        let mut len = 4;
        for pkt in &self.packets {
            len += pkt.get_total_length();
        }
        for raw in &self.raw_packets {
            len += raw.len();
        }
        len
    }
}

pub struct AcknowledgePacket {
    pub flag: u8,
    pub packets: Vec<u32>,
}

impl AcknowledgePacket {
    pub fn ack() -> Self { Self { flag: BITFLAG_ACK, packets: Vec::new() } }
    pub fn nak() -> Self { Self { flag: BITFLAG_NAK, packets: Vec::new() } }

    pub fn encode(&self) -> Vec<u8> {
        let mut stream = BinaryStream::new();
        stream.put_byte(BITFLAG_VALID | self.flag);
        let mut records: Vec<(bool, u32, u32)> = Vec::new();
        if !self.packets.is_empty() {
            let mut sorted = self.packets.clone();
            sorted.sort_unstable();
            sorted.dedup();
            let mut start = sorted[0];
            let mut last = sorted[0];
            for &current in sorted.iter().skip(1) {
                if current == last + 1 {
                    last = current;
                } else {
                    if start == last {
                        records.push((true, start, 0));
                    } else {
                        records.push((false, start, last));
                    }
                    start = current;
                    last = current;
                }
            }
            if start == last {
                records.push((true, start, 0));
            } else {
                records.push((false, start, last));
            }
        }
        stream.put_lshort(records.len() as u16);
        for (single, f, s) in records {
            stream.put_bool(single);
            stream.put_ltriad(f);
            if !single {
                stream.put_ltriad(s);
            }
        }
        stream.into_inner()
    }

    pub fn decode(buffer: Vec<u8>) -> Self {
        let mut stream = BinaryStream::from_bytes(buffer);
        let flag = stream.get_byte().unwrap_or(0) & 0x3f;
        let count = stream.get_lshort().unwrap_or(0);
        let mut packets = Vec::new();
        for _ in 0..count {
            let single = stream.get_bool().unwrap_or(false);
            let f = stream.get_ltriad().unwrap_or(0);
            if single {
                packets.push(f);
            } else {
                let s = stream.get_ltriad().unwrap_or(0);
                for v in f..=s {
                    packets.push(v);
                }
            }
        }
        Self { flag, packets }
    }
}
