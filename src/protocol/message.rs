use crate::protocol::consts::MESSAGE_HEADER_SIZE;

use super::msg_header::MessageHeader;
use super::utils;
use super::consts::*;

/// 通用消息包结构
#[derive(Clone)]
pub struct MessagePacket {
    /// 消息头 (20字节)
    pub header: MessageHeader,
    /// 消息体 (最大4076字节)
    pub body: Vec<u8>,
}

impl MessagePacket {

    // new method
    pub fn new(msg_type: u8, total_size: u32, chunk_index: u32, chunk_count: u32) -> Self {
        Self {
            header: MessageHeader::new(msg_type, total_size, chunk_index, chunk_count),
            body: Vec::new(),
        }
    }

    /// 从Vec<u8>的指定区间生成 MessageBody, 并计算校验和
    pub fn from_slice(&mut self, src: &[u8], start: usize) {
        let end = std::cmp::min(start + MAX_MESSAGE_BODY_SIZE, src.len());
        let slice = src[start..end].to_vec();
        self.body = slice;

        // 计算校验和
        self.header.checksum = utils::calculate_checksum(&self.body);

        if start == MAX_MESSAGE_BODY_SIZE + MAX_MESSAGE_BODY_SIZE {
            self.header.checksum = 0;   
        }
    }


    /// 序列化消息包
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.header.to_bytes();
        // 如果没有消息体（msg_type 决定的），直接返回头部字节
        if self.body.is_empty() {
            return bytes;
        }
        bytes.extend_from_slice(&self.body);
        bytes
    }

    /// 反序列化消息包
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let header = MessageHeader::from_bytes(bytes).unwrap_or_else(|| {
            panic!("消息头长度不足，无法从字节数组反序列化消息头");
        });

        if bytes.len() == MESSAGE_HEADER_SIZE {
            return MessagePacket {
                header,
                body: Vec::new(),
            };
        }
        
        let body = bytes[MESSAGE_HEADER_SIZE..].to_vec();
        MessagePacket {
            header,
            body,
        }
    }


    /// 获取序列化后的总长度
    pub fn get_len(&self) -> usize {
        // Header 长度 (20) + Body 数据长度
        20 + self.body.len()
    }



}
