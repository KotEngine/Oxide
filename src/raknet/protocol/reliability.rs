pub struct PacketReliability;

impl PacketReliability {
    pub const UNRELIABLE: u8 = 0;
    pub const UNRELIABLE_SEQUENCED: u8 = 1;
    pub const RELIABLE: u8 = 2;
    pub const RELIABLE_ORDERED: u8 = 3;
    pub const RELIABLE_SEQUENCED: u8 = 4;
    pub const UNRELIABLE_WITH_ACK_RECEIPT: u8 = 5;
    pub const RELIABLE_WITH_ACK_RECEIPT: u8 = 6;
    pub const RELIABLE_ORDERED_WITH_ACK_RECEIPT: u8 = 7;

    pub fn is_reliable(reliability: u8) -> bool {
        matches!(
            reliability,
            Self::RELIABLE | Self::RELIABLE_ORDERED | Self::RELIABLE_SEQUENCED
                | Self::RELIABLE_WITH_ACK_RECEIPT | Self::RELIABLE_ORDERED_WITH_ACK_RECEIPT
        )
    }

    pub fn is_sequenced(reliability: u8) -> bool {
        matches!(reliability, Self::UNRELIABLE_SEQUENCED | Self::RELIABLE_SEQUENCED)
    }

    pub fn is_ordered(reliability: u8) -> bool {
        matches!(
            reliability,
            Self::RELIABLE_ORDERED | Self::RELIABLE_ORDERED_WITH_ACK_RECEIPT
        )
    }

    pub fn is_sequenced_or_ordered(reliability: u8) -> bool {
        matches!(
            reliability,
            Self::UNRELIABLE_SEQUENCED | Self::RELIABLE_ORDERED
                | Self::RELIABLE_SEQUENCED | Self::RELIABLE_ORDERED_WITH_ACK_RECEIPT
        )
    }
}
