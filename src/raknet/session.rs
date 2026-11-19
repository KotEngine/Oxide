use crate::raknet::protocol::{PacketReliability, constants};
use crate::raknet::protocol::encapsulated::EncapsulatedPacket;
use crate::raknet::protocol::datagram::{Datagram, AcknowledgePacket};
use crate::raknet::protocol::online::*;
use crate::raknet::InternetAddress;

pub const STATE_CONNECTING: u8 = 0;
pub const STATE_CONNECTED: u8 = 1;
pub const STATE_DISCONNECTING: u8 = 2;
pub const STATE_DISCONNECTED: u8 = 3;

pub const CHANNEL_COUNT: usize = 32;
pub const WINDOW_SIZE: u32 = 2048;
pub const MIN_MTU_SIZE: usize = 400;
pub const MAX_SPLIT_SIZE: i32 = 128;
pub const MAX_SPLIT_COUNT: usize = 4;

#[derive(Clone)]
#[allow(dead_code)]
struct StoredDatagram {
    seq: u32,
    packets: Vec<EncapsulatedPacket>,
    raw_packets: Vec<Vec<u8>>,
    send_time: f64,
}

pub struct Session {
    pub address: InternetAddress,
    pub client_id: i64,
    pub mtu_size: usize,
    pub state: u8,
    pub is_temporal: bool,
    pub last_update: f64,
    pub last_ping_measure: i64,

    message_index: u32,
    send_ordered_index: [u32; CHANNEL_COUNT],
    send_sequenced_index: [u32; CHANNEL_COUNT],
    receive_ordered_index: [u32; CHANNEL_COUNT],
    receive_sequenced_highest: [u32; CHANNEL_COUNT],
    receive_ordered_packets: Vec<std::collections::HashMap<u32, EncapsulatedPacket>>,

    send_seq_number: u32,
    send_queue: Datagram,
    is_active: bool,

    ack_queue: Vec<u32>,
    nack_queue: Vec<u32>,
    recovery_queue: std::collections::HashMap<u32, StoredDatagram>,
    packet_to_send: Vec<StoredDatagram>,
    split_packets: std::collections::HashMap<u16, Vec<Option<EncapsulatedPacket>>>,
    split_id: u16,

    window_start: u32,
    window_end: u32,
    highest_seq_this_tick: i64,
    received_datagram_seq: std::collections::HashSet<u32>,

    reliable_window_start: u32,
    reliable_window_end: u32,
    reliable_window: std::collections::HashSet<u32>,

    last_ping_time: f64,
    disconnection_time: Option<f64>,
    outgoing: Vec<Vec<u8>>,
    pub incoming: Vec<Vec<u8>>,
}

impl Session {
    pub fn new(address: InternetAddress, client_id: i64, mtu_size: usize, now: f64) -> Self {
        let mut receive_ordered_packets = Vec::with_capacity(CHANNEL_COUNT);
        for _ in 0..CHANNEL_COUNT {
            receive_ordered_packets.push(std::collections::HashMap::new());
        }
        Self {
            address,
            client_id,
            mtu_size,
            state: STATE_CONNECTING,
            is_temporal: true,
            last_update: now,
            last_ping_measure: 1,
            message_index: 0,
            send_ordered_index: [0; CHANNEL_COUNT],
            send_sequenced_index: [0; CHANNEL_COUNT],
            receive_ordered_index: [0; CHANNEL_COUNT],
            receive_sequenced_highest: [0; CHANNEL_COUNT],
            receive_ordered_packets,
            send_seq_number: 0,
            send_queue: Datagram::new(),
            is_active: false,
            ack_queue: Vec::new(),
            nack_queue: Vec::new(),
            recovery_queue: std::collections::HashMap::new(),
            packet_to_send: Vec::new(),
            split_packets: std::collections::HashMap::new(),
            split_id: 0,
            window_start: 0,
            window_end: WINDOW_SIZE,
            highest_seq_this_tick: -1,
            received_datagram_seq: std::collections::HashSet::new(),
            reliable_window_start: 0,
            reliable_window_end: WINDOW_SIZE,
            reliable_window: std::collections::HashSet::new(),
            last_ping_time: -1.0,
            disconnection_time: None,
            outgoing: Vec::new(),
            incoming: Vec::new(),
        }
    }

    pub fn is_connected(&self) -> bool {
        self.state != STATE_DISCONNECTING && self.state != STATE_DISCONNECTED
    }

    pub fn take_outgoing(&mut self) -> Vec<Vec<u8>> {
        std::mem::take(&mut self.outgoing)
    }

    pub fn take_incoming(&mut self) -> Vec<Vec<u8>> {
        std::mem::take(&mut self.incoming)
    }

    pub fn send_payload(&mut self, payload: Vec<u8>, reliability: u8, order_channel: u8) {
        let mut enc = EncapsulatedPacket::new();
        enc.reliability = reliability;
        enc.order_channel = order_channel;
        enc.buffer = payload;
        self.add_encapsulated_to_queue(enc, constants::PRIORITY_NORMAL);
    }

    pub fn send_immediate(&mut self, payload: Vec<u8>, reliability: u8, order_channel: u8) {
        let mut enc = EncapsulatedPacket::new();
        enc.reliability = reliability;
        enc.order_channel = order_channel;
        enc.buffer = payload;
        self.add_encapsulated_to_queue(enc, constants::PRIORITY_IMMEDIATE);
    }

    fn send_datagram(&mut self, mut datagram: Datagram, now: f64) {
        if let Some(seq) = datagram.seq_number {
            self.recovery_queue.remove(&seq);
        }
        let seq = self.send_seq_number;
        datagram.seq_number = Some(seq);
        self.send_seq_number += 1;
        let stored = StoredDatagram {
            seq,
            packets: datagram.packets.clone(),
            raw_packets: datagram.raw_packets.clone(),
            send_time: now,
        };
        self.recovery_queue.insert(seq, stored);
        let bytes = datagram.encode();
        tracing::debug!(
            "DATAGRAM OUT seq={} packets={} bytes={} (mtu {})",
            seq, datagram.raw_packets.len() + datagram.packets.len(), bytes.len(), self.mtu_size
        );
        self.outgoing.push(bytes);
    }

    fn send_packet_bytes(&mut self, bytes: Vec<u8>) {
        self.outgoing.push(bytes);
    }

    pub fn send_queue(&mut self, now: f64) {
        if !self.send_queue.packets.is_empty() || !self.send_queue.raw_packets.is_empty() {
            let q = std::mem::take(&mut self.send_queue);
            self.send_datagram(q, now);
            self.send_queue = Datagram::new();
        }
    }

    fn add_to_queue(&mut self, pk: EncapsulatedPacket, flags: u8, now: f64) {
        let priority = flags & 0b00000111;
        let max_len = self.mtu_size.saturating_sub(36);
        if self.send_queue.length() + pk.get_total_length() > max_len {
            self.send_queue(now);
        }
        self.send_queue.raw_packets.push(pk.to_binary());
        if priority == constants::PRIORITY_IMMEDIATE {
            self.send_queue(now);
        }
    }

    fn add_encapsulated_to_queue(&mut self, mut packet: EncapsulatedPacket, flags: u8) {
        use PacketReliability as R;
        if R::is_ordered(packet.reliability) {
            packet.order_index = Some(self.send_ordered_index[packet.order_channel as usize]);
            self.send_ordered_index[packet.order_channel as usize] += 1;
        } else if R::is_sequenced(packet.reliability) {
            packet.order_index = Some(self.send_ordered_index[packet.order_channel as usize]);
            packet.sequence_index = Some(self.send_sequenced_index[packet.order_channel as usize]);
            self.send_sequenced_index[packet.order_channel as usize] += 1;
        }

        let now = current_time();
        let max_size = self.mtu_size.saturating_sub(60);
        if packet.buffer.len() > max_size {
            let buffer_count;
            let buffers: Vec<Vec<u8>> = {
                let mut chunks = Vec::new();
                let mut iter = packet.buffer.chunks(max_size);
                while let Some(c) = iter.next() {
                    chunks.push(c.to_vec());
                }
                buffer_count = chunks.len() as i32;
                chunks
            }
            .into_iter().collect();
            self.split_id = ((self.split_id as u32 + 1) % 65536) as u16;
            let split_id = self.split_id;
            for (i, buf) in buffers.into_iter().enumerate() {
                let mut pk = EncapsulatedPacket::new();
                pk.split_id = split_id;
                pk.has_split = true;
                pk.split_count = buffer_count;
                pk.reliability = packet.reliability;
                pk.split_index = i as i32;
                pk.buffer = buf;
                if R::is_reliable(pk.reliability) {
                    pk.message_index = Some(self.message_index);
                    self.message_index += 1;
                }
                pk.sequence_index = packet.sequence_index;
                pk.order_channel = packet.order_channel;
                pk.order_index = packet.order_index;
                self.add_to_queue(pk, flags | constants::PRIORITY_IMMEDIATE, now);
            }
        } else {
            if R::is_reliable(packet.reliability) {
                packet.message_index = Some(self.message_index);
                self.message_index += 1;
            }
            self.add_to_queue(packet, flags, now);
        }
    }

    fn handle_split(&mut self, packet: EncapsulatedPacket) -> Option<EncapsulatedPacket> {
        if packet.split_count >= MAX_SPLIT_SIZE || packet.split_count < 0
            || packet.split_index >= packet.split_count || packet.split_index < 0 {
            return None;
        }
        let entry = self.split_packets.entry(packet.split_id).or_insert_with(|| {
            let mut v = Vec::with_capacity(packet.split_count as usize);
            v.resize(packet.split_count as usize, None);
            v
        });
        if entry.len() != packet.split_count as usize {
            *entry = Vec::new();
            entry.resize(packet.split_count as usize, None);
        }
        entry[packet.split_index as usize] = Some(packet.clone());
        let have = entry.iter().filter(|p| p.is_some()).count();
        tracing::debug!(
            "SPLIT id={} part {}/{} (assembled {})",
            packet.split_id, packet.split_index + 1, packet.split_count, have
        );
        for p in entry.iter() {
            if p.is_none() { return None; }
        }
        let parts = std::mem::take(entry);
        self.split_packets.remove(&packet.split_id);
        let mut result = EncapsulatedPacket::new();
        result.reliability = packet.reliability;
        result.message_index = packet.message_index;
        result.sequence_index = packet.sequence_index;
        result.order_index = packet.order_index;
        result.order_channel = packet.order_channel;
        for p in parts {
            if let Some(p) = p {
                result.buffer.extend(p.buffer);
            }
        }
        Some(result)
    }

    fn handle_encapsulated(&mut self, mut packet: EncapsulatedPacket) {
        tracing::debug!(
            "ENCAP rel={} mi={:?} oi={:?} ch={} split={} ({}/{}) len={}",
            packet.reliability, packet.message_index, packet.order_index,
            packet.order_channel, packet.has_split,
            packet.split_index, packet.split_count, packet.buffer.len()
        );
        if let Some(mi) = packet.message_index {
            if mi < self.reliable_window_start || mi > self.reliable_window_end || self.reliable_window.contains(&mi) {
                tracing::warn!(
                    "DROPPED by reliable window: mi={} (window {}..{}, duplicate={})",
                    mi, self.reliable_window_start, self.reliable_window_end,
                    self.reliable_window.contains(&mi)
                );
                return;
            }
            self.reliable_window.insert(mi);
            if mi == self.reliable_window_start {
                while self.reliable_window.remove(&self.reliable_window_start) {
                    self.reliable_window_start += 1;
                    self.reliable_window_end += 1;
                }
            }
        }

        if packet.has_split {
            match self.handle_split(packet) {
                Some(p) => packet = p,
                None => return,
            }
        }

        use PacketReliability as R;
        if R::is_sequenced_or_ordered(packet.reliability) && packet.order_channel as usize >= CHANNEL_COUNT {
            tracing::warn!("DROPPED: channel {} out of range", packet.order_channel);
            return;
        }
        if R::is_sequenced(packet.reliability) {
            let ch = packet.order_channel as usize;
            let si = packet.sequence_index.unwrap_or(0);
            let oi = packet.order_index.unwrap_or(0);
            if si < self.receive_sequenced_highest[ch] || oi < self.receive_ordered_index[ch] {
                tracing::warn!("DROPPED stale sequenced packet: si={} oi={}", si, oi);
                return;
            }
            self.receive_sequenced_highest[ch] = si + 1;
            self.route_encapsulated(packet);
        } else if R::is_ordered(packet.reliability) {
            let ch = packet.order_channel as usize;
            let oi = packet.order_index.unwrap_or(0);
            if oi == self.receive_ordered_index[ch] {
                self.receive_sequenced_highest[ch] = 0;
                self.receive_ordered_index[ch] = oi + 1;
                self.route_encapsulated(packet);
                let mut i = self.receive_ordered_index[ch];
                while let Some(p) = self.receive_ordered_packets[ch].remove(&i) {
                    self.route_encapsulated(p);
                    i += 1;
                }
                self.receive_ordered_index[ch] = i;
            } else if oi > self.receive_ordered_index[ch] {
                tracing::warn!(
                    "BUFFERED ordered packet: oi={}, expecting {} (buffered {})",
                    oi, self.receive_ordered_index[ch], self.receive_ordered_packets[ch].len() + 1
                );
                self.receive_ordered_packets[ch].insert(oi, packet);
            } else {
                tracing::warn!("DROPPED stale ordered packet: oi={}, expecting {}", oi, self.receive_ordered_index[ch]);
            }
        } else {
            self.route_encapsulated(packet);
        }
    }

    fn route_encapsulated(&mut self, packet: EncapsulatedPacket) {
        if packet.buffer.is_empty() { return; }
        let id = packet.buffer[0];
        tracing::debug!(
            "ROUTE id=0x{:02x} rel={} len={} state={}",
            id, packet.reliability, packet.buffer.len(), self.state
        );
        use crate::raknet::protocol::message_ids::MessageIdentifiers as M;
        if id == M::ID_CONNECTION_REQUEST && self.state == STATE_CONNECTING {
            let req = ConnectionRequest::decode(packet.buffer.clone());
            let reply = ConnectionRequestAccepted::new(
                self.address.clone(),
                req.send_ping_time,
                current_time_millis(),
            );
            // UNRELIABLE, channel 0, PRIORITY_IMMEDIATE
            self.send_immediate(reply.encode(), PacketReliability::UNRELIABLE, 0);
        } else if id == M::ID_NEW_INCOMING_CONNECTION && self.state == STATE_CONNECTING {
            self.state = STATE_CONNECTED;
            self.is_temporal = false;
            let _nic = NewIncomingConnection::decode(packet.buffer.clone());
        } else if id == M::ID_DISCONNECTION_NOTIFICATION {
            self.state = STATE_DISCONNECTED;
        } else if id == M::ID_CONNECTED_PING {
            let ping = ConnectedPing::decode(packet.buffer.clone());
            let pong = ConnectedPong::new(ping.send_ping_time, current_time_millis());
            // UNRELIABLE, channel 0, normal priority
            self.send_payload(pong.encode(), PacketReliability::UNRELIABLE, 0);
        } else if id == M::ID_CONNECTED_PONG {
            let pong = ConnectedPong::decode(packet.buffer.clone());
            self.last_ping_measure = current_time_millis() - pong.send_ping_time;
        } else if id >= M::ID_USER_PACKET_ENUM {
            if self.state == STATE_CONNECTED {
                self.incoming.push(packet.buffer);
            } else {
                tracing::warn!(
                    "Game packet 0x{:02x} dropped: session is not CONNECTED yet (state={})",
                    id, self.state
                );
            }
        } else {
            tracing::debug!("ROUTE: unhandled packet 0x{:02x} (state={})", id, self.state);
        }
    }

    pub fn handle_datagram(&mut self, datagram: Datagram) {
        self.is_active = true;
        self.last_update = current_time();
        tracing::debug!(
            "DATAGRAM seq={:?} packets={}",
            datagram.seq_number, datagram.packets.len()
        );
        if let Some(seq) = datagram.seq_number {
            // Drop packets completely outside sliding window
            if seq < self.window_start || seq > self.window_end {
                tracing::debug!(
                    "Datagram seq={} outside window {}..{}",
                    seq, self.window_start, self.window_end
                );
                return;
            }

            // If already received, re-queue ACK so client stops resending, but don't reprocess
            if self.received_datagram_seq.contains(&seq) || self.ack_queue.contains(&seq) {
                if !self.ack_queue.contains(&seq) {
                    self.ack_queue.push(seq);
                }
                return;
            }

            self.received_datagram_seq.insert(seq);
            self.nack_queue.retain(|&s| s != seq);
            self.ack_queue.push(seq);

            if (seq as i64) > self.highest_seq_this_tick {
                self.highest_seq_this_tick = seq as i64;
            }

            if seq == self.window_start {
                while self.received_datagram_seq.contains(&self.window_start) {
                    self.received_datagram_seq.remove(&self.window_start);
                    self.window_start = self.window_start.wrapping_add(1);
                    self.window_end = self.window_end.wrapping_add(1);
                }
            } else if seq > self.window_start {
                for i in self.window_start..seq {
                    if !self.received_datagram_seq.contains(&i) && !self.ack_queue.contains(&i) && !self.nack_queue.contains(&i) {
                        self.nack_queue.push(i);
                    }
                }
            }
        }
        for pkt in datagram.packets {
            self.handle_encapsulated(pkt);
        }
    }

    pub fn handle_ack(&mut self, packets: Vec<u32>) {
        for seq in packets {
            if let Some(stored) = self.recovery_queue.remove(&seq) {
                let _ = stored;
            }
        }
    }

    pub fn handle_nack(&mut self, packets: Vec<u32>) {
        for seq in packets {
            if let Some(stored) = self.recovery_queue.get(&seq).cloned() {
                self.packet_to_send.push(stored);
                self.recovery_queue.remove(&seq);
            }
        }
    }

    pub fn update(&mut self, now: f64) {
        if !self.is_active && self.last_update + 10.0 < now {
            self.state = STATE_DISCONNECTED;
            return;
        }
        self.is_active = false;

        self.highest_seq_this_tick = -1;

        if !self.ack_queue.is_empty() {
            let ack = AcknowledgePacket { flag: crate::raknet::protocol::datagram::BITFLAG_ACK, packets: std::mem::take(&mut self.ack_queue) };
            tracing::debug!("ACK -> {:?}", ack.packets);
            self.send_packet_bytes(ack.encode());
        }
        if !self.nack_queue.is_empty() {
            let nak = AcknowledgePacket { flag: crate::raknet::protocol::datagram::BITFLAG_NAK, packets: std::mem::take(&mut self.nack_queue) };
            self.send_packet_bytes(nak.encode());
        }

        if !self.packet_to_send.is_empty() {
            let limit = 16.min(self.packet_to_send.len());
            let drained: Vec<StoredDatagram> = self.packet_to_send.drain(..limit).collect();
            for stored in drained {
                let mut dg = Datagram::new();
                dg.packets = stored.packets;
                dg.raw_packets = stored.raw_packets;
                self.send_datagram(dg, now);
            }
        }

        let cutoff = now - 8.0;
        let mut to_resend = Vec::new();
        for (&seq, stored) in self.recovery_queue.iter() {
            if stored.send_time < cutoff {
                to_resend.push(seq);
            } else {
                break;
            }
        }
        for seq in to_resend {
            if let Some(stored) = self.recovery_queue.remove(&seq) {
                self.packet_to_send.push(stored);
            }
        }

        if self.last_ping_time + 5.0 < now {
            let ping = ConnectedPing::new(current_time_millis());
            // UNRELIABLE, channel 0, PRIORITY_IMMEDIATE
            self.send_immediate(ping.encode(), PacketReliability::UNRELIABLE, 0);
            self.last_ping_time = now;
        }

        self.send_queue(now);
    }

    pub fn flag_for_disconnection(&mut self) {
        self.state = STATE_DISCONNECTING;
        self.disconnection_time = Some(current_time());
    }

    pub fn disconnect(&mut self) {
        if self.state != STATE_DISCONNECTED {
            self.state = STATE_DISCONNECTED;
            let disc = DisconnectionNotification::new();
            self.outgoing.push(disc.encode());
        }
    }
}

pub fn current_time() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0)
}

pub fn current_time_millis() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}
