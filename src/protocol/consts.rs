#[allow(dead_code)]
pub const MAX_MESSAGE_PACKET_SIZE: usize = 4096;
pub const MESSAGE_HEADER_SIZE: usize = 20;
pub const MAX_MESSAGE_BODY_SIZE: usize = 4076;

pub const PROTOCOL_VERSION: u8 = 1;

pub const MSG_TYPE_START: u8 = 0x01;
pub const MSG_TYPE_DATA: u8 = 0x02;
pub const MSG_TYPE_END: u8 = 0x03;
pub const MSG_TYPE_ACK: u8 = 0x04;
#[allow(dead_code)]
pub const MSG_TYPE_ERROR: u8 = 0x05;
pub const MSG_TYPE_ALL_END: u8 = 0x06;