use std::net::SocketAddr;
use tokio::net::UdpSocket;
use std::sync::Arc;
use parking_lot::RwLock;

pub struct QueryHandler {
    socket: Option<Arc<UdpSocket>>,
    server_name: String,
    max_players: u32,
    players: Arc<RwLock<Vec<String>>>,
}

const HANDSHAKE_MAGIC: u16 = 0xff00;
const STAT_MAGIC: u16 = 0x0001;

impl QueryHandler {
    pub fn new(server_name: String, max_players: u32) -> Self {
        Self {
            socket: None,
            server_name,
            max_players,
            players: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn start(&mut self, addr: SocketAddr) -> anyhow::Result<()> {
        let socket = Arc::new(UdpSocket::bind(addr).await?);
        self.socket = Some(socket.clone());
        let server_name = self.server_name.clone();
        let max_players = self.max_players;
        let players = self.players.clone();
        tokio::spawn(async move {
            let mut buf = vec![0u8; 65535];
            loop {
                if let Ok((len, peer)) = socket.recv_from(&mut buf).await {
                    if len < 7 { continue; }
                    let data = &buf[..len];
                    let magic = u16::from_be_bytes([data[2], data[3]]);
                    match magic {
                        HANDSHAKE_MAGIC => {
                            let mut reply = Vec::new();
                            reply.extend_from_slice(b"handshake_token_oxide");
                            reply.push(0);
                            let _ = socket.send_to(&reply, peer).await;
                        }
                        STAT_MAGIC => {
                            let mut stat = Vec::new();
                            stat.extend_from_slice(b"Oxide\0");
                            stat.push(0);
                            stat.extend_from_slice(format!("{}\0", server_name).as_bytes());
                            stat.push(0);
                            stat.extend_from_slice(format!("{}\0", server_name).as_bytes());
                            stat.push(0);
                            stat.extend_from_slice(b"MCPE\0");
                            stat.extend_from_slice(format!("{}\0", players.read().len()).as_bytes());
                            stat.extend_from_slice(format!("{}\0", max_players).as_bytes());
                            let _ = socket.send_to(&stat, peer).await;
                        }
                        _ => {}
                    }
                }
            }
        });
        Ok(())
    }

    pub fn add_player(&self, name: String) {
        self.players.write().push(name);
    }

    pub fn remove_player(&self, name: &str) {
        self.players.write().retain(|n| n != name);
    }
}
