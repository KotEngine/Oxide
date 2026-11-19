use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

#[derive(Debug)]
pub struct LoginPacket {
    pub username: String,
    pub protocol: i32,
    pub game_edition: u8,
    pub client_uuid: String,
    pub client_id: i64,
    pub identity_public_key: Option<String>,
    pub server_address: String,
    pub skin_id: Option<String>,
    pub skin: Option<Vec<u8>>,
    pub language_code: Option<String>,
    pub device_model: Option<String>,
    pub device_os: Option<i64>,
}

impl DataPacket for LoginPacket {
    const NETWORK_ID: u8 = ProtocolInfo::LOGIN_PACKET;

    fn encode_payload(&self, _stream: &mut BinaryStream) {}

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let buf_len = buf.len();
        tracing::debug!("LoginPacket decode: {} bytes, first 12 = {:02x?}",
            buf_len, buf.iter().take(12).copied().collect::<Vec<_>>());
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let protocol = match stream.get_int() {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("LoginPacket: protocol read failed: {}", e);
                return Self::empty(0, 0);
            }
        };
        let game_edition = stream.get_byte().unwrap_or(0);
        let total_len = stream.get_unsigned_var_int().unwrap_or(0) as usize;
        let remaining = stream.remaining();
        tracing::debug!("LoginPacket: protocol={}, edition={}, blob_len={}, remaining={}",
            protocol, game_edition, total_len, remaining);
        if total_len == 0 || total_len > remaining {
            tracing::warn!("LoginPacket: bad blob_len {} (remaining {})", total_len, remaining);
            return Self::empty(protocol, game_edition);
        }
        let blob = match stream.get(total_len) {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!("LoginPacket: blob read failed: {}", e);
                return Self::empty(protocol, game_edition);
            }
        };

        if blob.len() < 4 {
            tracing::warn!("LoginPacket: blob too short ({})", blob.len());
            return Self::empty(protocol, game_edition);
        }
        let chain_len = u32::from_le_bytes([blob[0], blob[1], blob[2], blob[3]]) as usize;
        if chain_len == 0 || 4 + chain_len > blob.len() {
            tracing::warn!("LoginPacket: bad chain_len {} (blob {})", chain_len, blob.len());
            return Self::empty(protocol, game_edition);
        }
        let chain_json: serde_json::Value = serde_json::from_slice(&blob[4..4 + chain_len]).unwrap_or_default();
        let chain = chain_json.get("chain").and_then(|c| c.as_array()).cloned().unwrap_or_default();

        let mut username = String::new();
        let mut client_uuid = String::new();
        let mut identity_public_key: Option<String> = None;

        for token in &chain {
            if let Some(s) = token.as_str() {
                if let Some(payload) = decode_jwt_payload(s) {
                    if let Some(extra) = payload.get("extraData") {
                        if let Some(dn) = extra.get("displayName").and_then(|v| v.as_str()) {
                            username = dn.to_string();
                        }
                        if let Some(id) = extra.get("identity").and_then(|v| v.as_str()) {
                            client_uuid = id.to_string();
                        }
                    }
                    if let Some(key) = payload.get("identityPublicKey").and_then(|v| v.as_str()) {
                        identity_public_key = Some(key.to_string());
                    }
                }
            }
        }

        let mut offset = 4 + chain_len;
        let client_data = if offset + 4 <= blob.len() {
            let client_len = u32::from_le_bytes([blob[offset], blob[offset+1], blob[offset+2], blob[offset+3]]) as usize;
            offset += 4;
            if client_len > 0 && offset + client_len <= blob.len() {
                let raw = &blob[offset..offset + client_len];
                let s = std::str::from_utf8(raw).ok();
                s.and_then(decode_jwt_payload).unwrap_or_default()
            } else {
                serde_json::Value::Null
            }
        } else {
            serde_json::Value::Null
        };

        let client_id = client_data.get("ClientRandomId").and_then(|v| v.as_i64()).unwrap_or(0);
        let server_address = client_data.get("ServerAddress").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let skin_id = client_data.get("SkinId").and_then(|v| v.as_str()).map(|s| s.to_string());
        let skin = client_data.get("SkinData").and_then(|v| v.as_str()).map(decode_base64);
        let language_code = client_data.get("LanguageCode").and_then(|v| v.as_str()).map(|s| s.to_string());
        let device_model = client_data.get("DeviceModel").and_then(|v| v.as_str()).map(|s| s.to_string());
        let device_os = client_data.get("DeviceOS").and_then(|v| v.as_i64());

        if username.is_empty() {
            username = "Player".to_string();
        }

        Self {
            username,
            protocol,
            game_edition,
            client_uuid,
            client_id,
            identity_public_key,
            server_address,
            skin_id,
            skin,
            language_code,
            device_model,
            device_os,
        }
    }

    fn name(&self) -> &'static str { "LoginPacket" }
    fn can_be_sent_before_login(&self) -> bool { true }
}

impl LoginPacket {
    pub const EDITION_POCKET: u8 = 0;

    fn empty(protocol: i32, game_edition: u8) -> Self {
        Self {
            username: "Player".to_string(),
            protocol,
            game_edition,
            client_uuid: String::new(),
            client_id: 0,
            identity_public_key: None,
            server_address: String::new(),
            skin_id: None,
            skin: None,
            language_code: None,
            device_model: None,
            device_os: None,
        }
    }
}

fn decode_jwt_payload(token: &str) -> Option<serde_json::Value> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() < 2 { return None; }
    decode_b64_json(parts[1])
}

fn decode_b64_json(input: &str) -> Option<serde_json::Value> {
    use base64::{Engine, engine::general_purpose};
    if let Ok(bytes) = general_purpose::URL_SAFE_NO_PAD.decode(input) {
        return serde_json::from_slice(&bytes).ok();
    }
    let padded = match input.len() % 4 {
        2 => format!("{}==", input),
        3 => format!("{}=", input),
        _ => input.to_string(),
    };
    if let Ok(bytes) = general_purpose::URL_SAFE.decode(&padded) {
        return serde_json::from_slice(&bytes).ok();
    }
    if let Ok(bytes) = general_purpose::STANDARD.decode(input) {
        return serde_json::from_slice(&bytes).ok();
    }
    None
}

fn decode_base64(input: &str) -> Vec<u8> {
    use base64::{Engine, engine::general_purpose};
    if let Ok(v) = general_purpose::STANDARD.decode(input) { return v; }
    if let Ok(v) = general_purpose::URL_SAFE_NO_PAD.decode(input) { return v; }
    Vec::new()
}
