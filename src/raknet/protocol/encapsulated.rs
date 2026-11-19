use crate::utils::Binary;

#[derive(Clone)]
pub struct EncapsulatedPacket {
    pub reliability: u8,
    pub has_split: bool,
    pub message_index: Option<u32>,
    pub sequence_index: Option<u32>,
    pub order_index: Option<u32>,
    pub order_channel: u8,
    pub split_count: i32,
    pub split_id: u16,
    pub split_index: i32,
    pub buffer: Vec<u8>,
    pub need_ack: bool,
    pub identifier_ack: Option<i32>,
}

impl Default for EncapsulatedPacket {
    fn default() -> Self { Self::new() }
}

impl EncapsulatedPacket {
    const RELIABILITY_SHIFT: u8 = 5;
    const RELIABILITY_MASK: u8 = 0b111 << Self::RELIABILITY_SHIFT;
    const SPLIT_FLAG: u8 = 0b00010000;

    pub fn new() -> Self {
        Self {
            reliability: 0,
            has_split: false,
            message_index: None,
            sequence_index: None,
            order_index: None,
            order_channel: 0,
            split_count: 0,
            split_id: 0,
            split_index: 0,
            buffer: Vec::new(),
            need_ack: false,
            identifier_ack: None,
        }
    }

    pub fn from_binary(binary: &[u8]) -> (Self, usize) {
        if binary.is_empty() {
            return (Self::new(), 0);
        }
        let flags = binary[0];
        let reliability = (flags & Self::RELIABILITY_MASK) >> Self::RELIABILITY_SHIFT;
        let has_split = (flags & Self::SPLIT_FLAG) != 0;

        let length = ((Binary::read_short(&binary[1..3]) as usize) + 7) / 8;
        let mut offset = 3usize;

        let mut message_index = None;
        let mut sequence_index = None;
        let mut order_index = None;
        let mut order_channel = 0u8;

        use crate::raknet::protocol::PacketReliability as R;
        if R::is_reliable(reliability) {
            message_index = Some(Binary::read_ltriad(&binary[offset..offset + 3]));
            offset += 3;
        }
        if R::is_sequenced(reliability) {
            sequence_index = Some(Binary::read_ltriad(&binary[offset..offset + 3]));
            offset += 3;
        }
        if R::is_sequenced_or_ordered(reliability) {
            order_index = Some(Binary::read_ltriad(&binary[offset..offset + 3]));
            offset += 3;
            order_channel = binary[offset];
            offset += 1;
        }

        let mut split_count = 0;
        let mut split_id = 0;
        let mut split_index = 0;

        if has_split {
            split_count = Binary::read_signed_int(&binary[offset..offset + 4]);
            offset += 4;
            split_id = Binary::read_short(&binary[offset..offset + 2]);
            offset += 2;
            split_index = Binary::read_signed_int(&binary[offset..offset + 4]);
            offset += 4;
        }

        let buffer = if offset + length <= binary.len() {
            binary[offset..offset + length].to_vec()
        } else {
            binary[offset..].to_vec()
        };
        offset += length;

        (
            Self {
                reliability,
                has_split,
                message_index,
                sequence_index,
                order_index,
                order_channel,
                split_count,
                split_id,
                split_index,
                buffer,
                need_ack: false,
                identifier_ack: None,
            },
            offset,
        )
    }

    pub fn to_binary(&self) -> Vec<u8> {
        use crate::raknet::protocol::PacketReliability as R;
        let mut out = Vec::new();
        out.push((self.reliability << Self::RELIABILITY_SHIFT) | (if self.has_split { Self::SPLIT_FLAG } else { 0 }));
        out.extend_from_slice(&Binary::write_short((self.buffer.len() << 3) as u16));

        if R::is_reliable(self.reliability) {
            if let Some(mi) = self.message_index {
                out.extend_from_slice(&Binary::write_ltriad(mi));
            } else {
                out.extend_from_slice(&Binary::write_ltriad(0));
            }
        }
        if R::is_sequenced(self.reliability) {
            if let Some(si) = self.sequence_index {
                out.extend_from_slice(&Binary::write_ltriad(si));
            } else {
                out.extend_from_slice(&Binary::write_ltriad(0));
            }
        }
        if R::is_sequenced_or_ordered(self.reliability) {
            if let Some(oi) = self.order_index {
                out.extend_from_slice(&Binary::write_ltriad(oi));
            } else {
                out.extend_from_slice(&Binary::write_ltriad(0));
            }
            out.push(self.order_channel);
        }

        if self.has_split {
            out.extend_from_slice(&Binary::write_int(self.split_count as u32));
            out.extend_from_slice(&Binary::write_short(self.split_id));
            out.extend_from_slice(&Binary::write_int(self.split_index as u32));
        }

        out.extend_from_slice(&self.buffer);
        out
    }

    pub fn get_total_length(&self) -> usize {
        use crate::raknet::protocol::PacketReliability as R;
        let mut len = 1 + 2;
        if R::is_reliable(self.reliability) { len += 3; }
        if R::is_sequenced(self.reliability) { len += 3; }
        if R::is_sequenced_or_ordered(self.reliability) { len += 3 + 1; }
        if self.has_split { len += 4 + 2 + 4; }
        len + self.buffer.len()
    }
}
