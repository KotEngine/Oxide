use std::net::SocketAddr;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use parking_lot::RwLock;
use tracing::{info, warn, error, debug};

use crate::raknet::protocol::constants::{MCPE_RAKNET_PROTOCOL_VERSION, MIN_MTU_SIZE};
use crate::raknet::protocol::message_ids::MessageIdentifiers;
use crate::raknet::protocol::datagram::{Datagram, AcknowledgePacket, BITFLAG_VALID, BITFLAG_ACK, BITFLAG_NAK};
use crate::raknet::protocol::offline::*;
use crate::raknet::session::{Session, current_time};
use crate::raknet::InternetAddress;

pub enum RakNetEvent {
    OpenSession(InternetAddress, i64),
    CloseSession(InternetAddress, String),
    Encapsulated(InternetAddress, Vec<u8>),
    Raw(InternetAddress, Vec<u8>),
}

pub struct RakNetServer {
    socket: Arc<UdpSocket>,
    sessions: Arc<RwLock<HashMap<SocketAddr, Session>>>,
    server_id: i64,
    pub server_name: String,
    pub events: Arc<Mutex<Vec<RakNetEvent>>>,
    shutdown: Arc<RwLock<bool>>,
}

impl RakNetServer {
    pub async fn new(addr: SocketAddr, server_name: String) -> anyhow::Result<Self> {
        let socket = UdpSocket::bind(addr).await?;
        info!("RakNet bound to {}", addr);
        let server_id: i64 = {
            use rand::Rng;
            rand::thread_rng().gen::<u64>() as i64
        };
        Ok(Self {
            socket: socket.into(),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            server_id,
            server_name,
            events: Arc::new(Mutex::new(Vec::new())),
            shutdown: Arc::new(RwLock::new(false)),
        })
    }

    pub fn server_id(&self) -> i64 { self.server_id }

    pub async fn start(self: Arc<Self>) -> anyhow::Result<()> {
        let recv_self = self.clone();
        tokio::spawn(async move {
            recv_self.recv_loop().await;
        });
        let tick_self = self.clone();
        tokio::spawn(async move {
            tick_self.tick_loop().await;
        });
        Ok(())
    }

    async fn recv_loop(&self) {
        let mut buf = vec![0u8; 65535];
        loop {
            if *self.shutdown.read() { return; }
            match self.socket.recv_from(&mut buf).await {
                Ok((len, addr)) => {
                    let data = buf[..len].to_vec();
                    self.handle_packet(data, addr).await;
                }
                Err(e) => {
                    // On Windows UDP socket returns WSAECONNRESET (10054) when previous
                    // sendto received ICMP port unreachable from a disconnected client.
                    // Not a fatal socket error; the session will be closed by timeout.
                    if e.kind() == std::io::ErrorKind::ConnectionReset {
                        debug!("recv: connection reset by remote host (ignored)");
                    } else {
                        error!("recv error: {}", e);
                    }
                }
            }
        }
    }

    async fn tick_loop(&self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(10));
        loop {
            if *self.shutdown.read() { return; }
            interval.tick().await;
            let now = current_time();
            let mut to_remove = Vec::new();
            let mut outgoing: Vec<(SocketAddr, Vec<u8>)> = Vec::new();
            let mut new_events = Vec::new();

            {
                let mut sessions = self.sessions.write();
                for (addr, session) in sessions.iter_mut() {
                    session.update(now);
                    for out in session.take_outgoing() {
                        outgoing.push((*addr, out));
                    }
                    for inc in session.take_incoming() {
                        new_events.push(RakNetEvent::Encapsulated(Self::sock_to_internet(*addr, 4), inc));
                    }
                    if session.state == crate::raknet::session::STATE_DISCONNECTED {
                        new_events.push(RakNetEvent::CloseSession(
                            Self::sock_to_internet(*addr, 4),
                            "disconnected".to_string(),
                        ));
                        to_remove.push(*addr);
                    }
                }
            }

            for (addr, data) in outgoing {
                let _ = self.socket.send_to(&data, addr).await;
            }
            if !new_events.is_empty() {
                let mut ev = self.events.lock().await;
                ev.extend(new_events);
            }
            {
                let mut sessions = self.sessions.write();
                for addr in to_remove {
                    sessions.remove(&addr);
                }
            }
        }
    }

    async fn handle_packet(&self, data: Vec<u8>, addr: SocketAddr) {
        if data.is_empty() { return; }
        let id = data[0];

        if id == MessageIdentifiers::ID_UNCONNECTED_PING {
            let ping = UnconnectedPing::decode(data);
            if !ping.is_valid() { return; }
            let pong = UnconnectedPong::new(ping.ping_id, self.server_id, self.server_name.clone());
            let _ = self.socket.send_to(&pong.encode(), addr).await;
        } else if id == MessageIdentifiers::ID_UNCONNECTED_PING_OPEN_CONNECTIONS {
            let pong = UnconnectedPong::new(0, self.server_id, self.server_name.clone());
            let _ = self.socket.send_to(&pong.encode(), addr).await;
        } else if id == MessageIdentifiers::ID_OPEN_CONNECTION_REQUEST_1 {
            let req = OpenConnectionRequest1::decode(data);
            if !req.is_valid() { return; }
            if req.protocol != MCPE_RAKNET_PROTOCOL_VERSION {
                // Return server protocol version so the client knows expected version
                let reply = IncompatibleProtocolVersion::new(MCPE_RAKNET_PROTOCOL_VERSION, self.server_id);
                let _ = self.socket.send_to(&reply.encode(), addr).await;
                warn!(
                    "Rejected {} - incompatible RakNet protocol version (expected {}, received {})",
                    addr, MCPE_RAKNET_PROTOCOL_VERSION, req.protocol
                );
                return;
            }
            let mtu = req.mtu_size.clamp(MIN_MTU_SIZE, 1492) as u16;
            let reply = OpenConnectionReply1::new(self.server_id, mtu);
            let _ = self.socket.send_to(&reply.encode(), addr).await;
        } else if id == MessageIdentifiers::ID_OPEN_CONNECTION_REQUEST_2 {
            let req = OpenConnectionRequest2::decode(data);
            if !req.is_valid() { return; }
            let mtu = req.mtu_size.max(MIN_MTU_SIZE as u16) as usize;
            let internet = Self::sock_to_internet(addr, 4);
            let (session_exists,) = {
                let sessions = self.sessions.read();
                (sessions.contains_key(&addr),)
            };
            if !session_exists {
                {
                    let mut sessions = self.sessions.write();
                    let session = Session::new(internet.clone(), req.client_id, mtu, current_time());
                    sessions.insert(addr, session);
                }
                let reply = OpenConnectionReply2::new(self.server_id, internet, mtu as u16);
                let _ = self.socket.send_to(&reply.encode(), addr).await;
            } else {
                let reply = OpenConnectionReply2::new(self.server_id, internet, mtu as u16);
                let _ = self.socket.send_to(&reply.encode(), addr).await;
            }
        } else if (id & BITFLAG_VALID) != 0 {
            if (id & BITFLAG_ACK) != 0 {
                let ack = AcknowledgePacket::decode(data);
                let mut sessions = self.sessions.write();
                if let Some(session) = sessions.get_mut(&addr) {
                    session.handle_ack(ack.packets);
                }
            } else if (id & BITFLAG_NAK) != 0 {
                let nak = AcknowledgePacket::decode(data);
                let mut sessions = self.sessions.write();
                if let Some(session) = sessions.get_mut(&addr) {
                    session.handle_nack(nak.packets);
                }
            } else {
                let datagram = Datagram::decode(data);
                let is_new = !self.sessions.read().contains_key(&addr);
                if is_new && datagram.packets.is_empty() { return; }
                let (was_connecting, now_connected, cid, inc_packets, out_packets) = {
                    let mut sessions = self.sessions.write();
                    let session = sessions.entry(addr).or_insert_with(|| {
                        Session::new(Self::sock_to_internet(addr, 4), 0, 1492, current_time())
                    });
                    let was_connecting = session.state == crate::raknet::session::STATE_CONNECTING;
                    session.handle_datagram(datagram);
                    session.flush_acks();
                    let now_connected = session.state == crate::raknet::session::STATE_CONNECTED;
                    let inc = session.take_incoming();
                    let out = session.take_outgoing();
                    (was_connecting, now_connected, session.client_id, inc, out)
                };
                for out_bytes in out_packets {
                    let _ = self.socket.send_to(&out_bytes, addr).await;
                }
                if !inc_packets.is_empty() {
                    let mut ev = self.events.lock().await;
                    let internet = Self::sock_to_internet(addr, 4);
                    for inc in inc_packets {
                        ev.push(RakNetEvent::Encapsulated(internet.clone(), inc));
                    }
                }
                if was_connecting && now_connected {
                    let ev = self.events.clone();
                    let internet = Self::sock_to_internet(addr, 4);
                    tokio::spawn(async move {
                        let mut e = ev.lock().await;
                        e.push(RakNetEvent::OpenSession(internet, cid));
                    });
                }
            }
        }
    }

    pub async fn send_encapsulated(&self, addr: SocketAddr, payload: Vec<u8>, reliable_ordered: bool) {
        let reliability = if reliable_ordered { crate::raknet::protocol::PacketReliability::RELIABLE_ORDERED } else { crate::raknet::protocol::PacketReliability::UNRELIABLE };
        let outgoing = {
            let mut sessions = self.sessions.write();
            if let Some(session) = sessions.get_mut(&addr) {
                session.send_payload(payload, reliability, 0);
                session.send_queue(current_time());
                session.take_outgoing()
            } else {
                Vec::new()
            }
        };
        for data in outgoing {
            let _ = self.socket.send_to(&data, addr).await;
        }
    }

    pub async fn close_session(&self, addr: SocketAddr, reason: String) {
        {
            let mut sessions = self.sessions.write();
            if let Some(session) = sessions.get_mut(&addr) {
                session.disconnect();
            }
        }
        let mut ev = self.events.lock().await;
        ev.push(RakNetEvent::CloseSession(Self::sock_to_internet(addr, 4), reason));
    }

    pub async fn drain_events(&self) -> Vec<RakNetEvent> {
        let mut ev = self.events.lock().await;
        std::mem::take(&mut *ev)
    }

    pub fn sock_to_internet(addr: SocketAddr, version: u8) -> InternetAddress {
        InternetAddress::new(addr.ip(), addr.port(), version)
    }

    pub async fn shutdown(&self) {
        *self.shutdown.write() = true;
    }

    pub async fn get_session_addrs(&self) -> Vec<SocketAddr> {
        self.sessions.read().keys().copied().collect()
    }
}
