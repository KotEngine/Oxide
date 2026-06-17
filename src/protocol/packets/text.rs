use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

pub const TYPE_RAW: u8 = 0;
pub const TYPE_CHAT: u8 = 1;
pub const TYPE_TRANSLATION: u8 = 2;
pub const TYPE_POPUP: u8 = 3;
pub const TYPE_TIP: u8 = 4;
pub const TYPE_SYSTEM: u8 = 5;
pub const TYPE_WHISPER: u8 = 6;

pub struct TextPacket {
    pub text_type: u8,
    pub source: String,
    pub message: String,
    pub parameters: Vec<String>,
}

impl TextPacket {
    pub fn raw(message: impl Into<String>) -> Self {
        Self { text_type: TYPE_RAW, source: String::new(), message: message.into(), parameters: Vec::new() }
    }
    pub fn chat(source: impl Into<String>, message: impl Into<String>) -> Self {
        Self { text_type: TYPE_CHAT, source: source.into(), message: message.into(), parameters: Vec::new() }
    }
    pub fn system(message: impl Into<String>) -> Self {
        Self { text_type: TYPE_SYSTEM, source: String::new(), message: message.into(), parameters: Vec::new() }
    }
    pub fn tip(message: impl Into<String>) -> Self {
        Self { text_type: TYPE_TIP, source: String::new(), message: message.into(), parameters: Vec::new() }
    }
    pub fn popup(message: impl Into<String>) -> Self {
        Self { text_type: TYPE_POPUP, source: String::new(), message: message.into(), parameters: Vec::new() }
    }
}

impl DataPacket for TextPacket {
    const NETWORK_ID: u8 = ProtocolInfo::TEXT_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        stream.put_byte(self.text_type);
        match self.text_type {
            TYPE_POPUP | TYPE_CHAT | TYPE_WHISPER => {
                stream.put_string(&self.source);
                stream.put_string(&self.message);
            }
            TYPE_RAW | TYPE_TIP | TYPE_SYSTEM => {
                stream.put_string(&self.message);
            }
            TYPE_TRANSLATION => {
                stream.put_string(&self.message);
                stream.put_unsigned_var_int(self.parameters.len() as u32);
                for p in &self.parameters {
                    stream.put_string(p);
                }
            }
            _ => {}
        }
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let text_type = stream.get_byte().unwrap_or(0);
        let mut source = String::new();
        let mut message = String::new();
        let mut parameters = Vec::new();
        match text_type {
            TYPE_POPUP | TYPE_CHAT | TYPE_WHISPER => {
                source = stream.get_string().unwrap_or_default();
                message = stream.get_string().unwrap_or_default();
            }
            TYPE_RAW | TYPE_TIP | TYPE_SYSTEM => {
                message = stream.get_string().unwrap_or_default();
            }
            TYPE_TRANSLATION => {
                message = stream.get_string().unwrap_or_default();
                let count = stream.get_unsigned_var_int().unwrap_or(0);
                for _ in 0..count {
                    if stream.eof() { break; }
                    parameters.push(stream.get_string().unwrap_or_default());
                }
            }
            _ => {}
        }
        Self { text_type, source, message, parameters }
    }

    fn name(&self) -> &'static str { "TextPacket" }
}
