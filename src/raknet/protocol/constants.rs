pub const RAKNET_VERSION: &str = "0.12.0";

pub const DEFAULT_PROTOCOL_VERSION: u8 = 6;

pub const MAGIC: [u8; 16] = [
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe,
    0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];

pub const PRIORITY_NORMAL: u8 = 0;
pub const PRIORITY_IMMEDIATE: u8 = 1;

pub const FLAG_NEED_ACK: u8 = 0b00001000;

pub const SYSTEM_ADDRESS_COUNT: usize = 20;

pub const MIN_MTU_SIZE: usize = 400;
pub const MAX_MTU_SIZE: usize = 1492;

pub const PACKET_ENCAPSULATED: u8 = 0x01;
pub const PACKET_OPEN_SESSION: u8 = 0x02;
pub const PACKET_CLOSE_SESSION: u8 = 0x03;
pub const PACKET_INVALID_SESSION: u8 = 0x04;
pub const PACKET_SEND_QUEUE: u8 = 0x05;
pub const PACKET_ACK_NOTIFICATION: u8 = 0x06;
pub const PACKET_SET_OPTION: u8 = 0x07;
pub const PACKET_RAW: u8 = 0x08;
pub const PACKET_BLOCK_ADDRESS: u8 = 0x09;
pub const PACKET_UNBLOCK_ADDRESS: u8 = 0x10;
pub const PACKET_REPORT_PING: u8 = 0x11;
pub const PACKET_SHUTDOWN: u8 = 0x7e;
pub const PACKET_EMERGENCY_SHUTDOWN: u8 = 0x7f;
