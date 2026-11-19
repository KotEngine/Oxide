use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;

pub struct RconServer {
    password: String,
}

impl RconServer {
    pub fn new(password: String) -> Self {
        Self { password }
    }

    pub async fn start(self, addr: SocketAddr) -> anyhow::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        let password = Arc::new(self.password);
        loop {
            if let Ok((mut socket, _peer)) = listener.accept().await {
                let pwd = password.clone();
                tokio::spawn(async move {
                    let mut len_buf = [0u8; 4];
                    loop {
                        if socket.read_exact(&mut len_buf).await.is_err() { break; }
                        let len = i32::from_le_bytes(len_buf) as usize;
                        if len < 4 || len > 4096 { break; }
                        let mut buf = vec![0u8; len];
                        if socket.read_exact(&mut buf).await.is_err() { break; }
                        let id = i32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                        let _type = i32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
                        let body = &buf[8..len.saturating_sub(2)];
                        let body_str = String::from_utf8_lossy(body).to_string();

                        if _type == 3 {
                            if body_str == *pwd {
                                let response = Self::encode_packet(id, 2, "");
                                let _ = socket.write_all(&response).await;
                            } else {
                                let response = Self::encode_packet(-1, 2, "");
                                let _ = socket.write_all(&response).await;
                                break;
                            }
                        } else if _type == 2 {
                            let response = Self::encode_packet(id, 0, "Command executed");
                            let _ = socket.write_all(&response).await;
                        }
                    }
                });
            }
        }
    }

    fn encode_packet(id: i32, packet_type: i32, body: &str) -> Vec<u8> {
        let body_bytes = body.as_bytes();
        let len = (10 + body_bytes.len()) as i32;
        let mut out = Vec::with_capacity(4 + len as usize);
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&id.to_le_bytes());
        out.extend_from_slice(&packet_type.to_le_bytes());
        out.extend_from_slice(body_bytes);
        out.push(0);
        out.push(0);
        out
    }
}
