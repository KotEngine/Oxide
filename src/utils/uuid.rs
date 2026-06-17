use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UUID {
    parts: [u32; 4],
}

impl UUID {
    pub fn from_parts(part0: u32, part1: u32, part2: u32, part3: u32) -> Self {
        Self { parts: [part0, part1, part2, part3] }
    }

    pub fn nil() -> Self { Self { parts: [0; 4] } }

    pub fn random() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Self {
            parts: [
                rng.gen::<u32>(),
                rng.gen::<u32>(),
                rng.gen::<u32>(),
                rng.gen::<u32>(),
            ],
        }
    }

    fn decode_hex_nibble(c: u8) -> Option<u8> {
        match c {
            b'0'..=b'9' => Some(c - b'0'),
            b'a'..=b'f' => Some(c - b'a' + 10),
            b'A'..=b'F' => Some(c - b'A' + 10),
            _ => None,
        }
    }

    pub fn from_string(s: &str) -> Option<Self> {
        let cleaned: Vec<u8> = s.bytes().filter(|c| *c != b'-').collect();
        if cleaned.len() != 32 { return None; }
        let mut bytes = [0u8; 16];
        for i in 0..16 {
            let hi = Self::decode_hex_nibble(cleaned[i * 2])?;
            let lo = Self::decode_hex_nibble(cleaned[i * 2 + 1])?;
            bytes[i] = (hi << 4) | lo;
        }
        Some(Self::from_bytes(&bytes))
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() >= 16);
        let mut parts = [0u32; 4];
        for i in 0..4 {
            parts[i] = u32::from_be_bytes([
                bytes[i * 4],
                bytes[i * 4 + 1],
                bytes[i * 4 + 2],
                bytes[i * 4 + 3],
            ]);
        }
        Self { parts }
    }

    pub fn to_bytes(&self) -> [u8; 16] {
        let mut out = [0u8; 16];
        for i in 0..4 {
            let b = self.parts[i].to_be_bytes();
            out[i * 4..i * 4 + 4].copy_from_slice(&b);
        }
        out
    }

    pub fn parts(&self) -> (u32, u32, u32, u32) {
        (self.parts[0], self.parts[1], self.parts[2], self.parts[3])
    }

    pub fn to_string(&self) -> String {
        let bytes = self.to_bytes();
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5],
            bytes[6], bytes[7],
            bytes[8], bytes[9],
            bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
        )
    }
}

impl fmt::Display for UUID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
