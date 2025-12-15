
use std::convert::TryInto;
use super::consts::*;



#[derive(Debug, Clone)]
/// 协议头结构体
pub struct MessageHeader {
    // 协议版本 (1字节)
    pub version: u8,
    // 消息类型 (1字节) (START=0x01, DATA=0x02, END=0x03, ACK=0x04, ERROR=0x05)
    pub msg_type: u8,
    // 消息标识符 (4字节)，用于区分并发传输
    pub message_id: u32,
    // 数据总大小 (4字节)
    pub total_size: u32,
    // 当前分片索引 (4字节，从0开始)
    pub chunk_index: u32,
    // 总分片数 (4字节)
    pub chunk_count: u32,
    // 保留字段 (1字节)
    pub reserved: u8,
    // 数据校验和 (1字节，简单累加和) 
    pub checksum: u8,
}

impl MessageHeader {
    pub fn new(msg_type: u8, total_size: u32, chunk_index: u32, chunk_count: u32) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            msg_type,
            message_id: 0,   // 后期设置唯一ID
            total_size,
            chunk_index,
            chunk_count,
            reserved: 0,   // 保留字段置0
            checksum: 0,   // 后期完善校验算法
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(MESSAGE_HEADER_SIZE);
        bytes.push(self.version);
        bytes.push(self.msg_type);
        bytes.extend_from_slice(&self.message_id.to_be_bytes());
        bytes.extend_from_slice(&self.total_size.to_be_bytes());
        bytes.extend_from_slice(&self.chunk_index.to_be_bytes());
        bytes.extend_from_slice(&self.chunk_count.to_be_bytes());
        bytes.push(self.reserved);
        bytes.push(self.checksum);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < MESSAGE_HEADER_SIZE {
            return None; 
        }
        Some(MessageHeader {
            version: bytes[0],
            msg_type: bytes[1],
            message_id: u32::from_be_bytes(bytes[2..6].try_into().ok()?),
            total_size: u32::from_be_bytes(bytes[6..10].try_into().ok()?),
            chunk_index: u32::from_be_bytes(bytes[10..14].try_into().ok()?),
            chunk_count: u32::from_be_bytes(bytes[14..18].try_into().ok()?),
            reserved: bytes[18],
            checksum: bytes[19],
        })
    }

    pub fn set_message_id(&mut self, msg_id: u32) {
        self.message_id = msg_id;
    }

    pub fn set_reserved(&mut self, reserved: u8) {
        self.reserved = reserved;
    }
}
