use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

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

    fn decode(mut buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let protocol = stream.get_int().unwrap_or(0);
        if !ProtocolInfo::ACCEPTED_PROTOCOLS.contains(&(protocol as u32)) {
            return Self::empty(protocol);
        }
        let game_edition = stream.get_byte().unwrap_or(0);
        let chain_len = stream.get_unsigned_var_int().unwrap_or(0) as usize;
        let _chain_str = stream.get_string().unwrap_or_default();
        let _ = chain_len;

        let chain_blob_len = stream.get_lint().unwrap_or(0) as usize;
        let chain_blob = stream.get(chain_blob_len).unwrap_or_default();
        let chain_json: serde_json::Value = serde_json::from_slice(&chain_blob).unwrap_or_default();
        let chain = chain_json.get("chain").and_then(|c| c.as_array()).cloned().unwrap_or_default();

        let mut username = String::new();
        let mut client_uuid = String::new();
        let mut identity_public_key: Option<String> = None;
        let mut verified = false;

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
                        verified = true;
                    }
                }
            }
        }

        let client_data_len = stream.get_lint().unwrap_or(0) as usize;
        let client_data_blob = stream.get(client_data_len).unwrap_or_default();
        let client_data = decode_jwt_payload_from_b64(&client_data_blob).unwrap_or_default();

        let client_id = client_data.get("ClientRandomId").and_then(|v| v.as_i64()).unwrap_or(0);
        let server_address = client_data.get("ServerAddress").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let skin_id = client_data.get("SkinId").and_then(|v| v.as_str()).map(|s| s.to_string());
        let skin = client_data.get("SkinData").and_then(|v| v.as_str()).map(|s| decode_base64(s));
        let language_code = client_data.get("LanguageCode").and_then(|v| v.as_str()).map(|s| s.to_string());
        let device_model = client_data.get("DeviceModel").and_then(|v| v.as_str()).map(|s| s.to_string());
        let device_os = client_data.get("DeviceOS").and_then(|v| v.as_i64());

        Self {
            username,
            protocol,
            game_edition,
            client_uuid,
            client_id,
            identity_public_key: if verified { identity_public_key } else { None },
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
    fn empty(protocol: i32) -> Self {
        Self {
            username: String::new(),
            protocol,
            game_edition: 0,
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
    if parts.len() != 3 { return None; }
    decode_b64_json(parts[1])
}

fn decode_jwt_payload_from_b64(blob: &[u8]) -> Option<serde_json::Value> {
    let s = std::str::from_utf8(blob).ok()?;
    decode_jwt_payload(s)
}

fn decode_b64_json(input: &str) -> Option<serde_json::Value> {
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    let bytes = URL_SAFE_NO_PAD.decode(input).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn decode_base64(input: &str) -> Vec<u8> {
    use base64::{Engine, engine::general_purpose::STANDARD};
    STANDARD.decode(input).unwrap_or_default()
}
