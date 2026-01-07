// src/data_process.rs
// 数据处理相关函数
use std::fs;
use std::io::{Read, Write};
use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use flate2::Compression;
use anyhow::Result;

use crate::protocol::MessagePacket;
use crate::protocol::consts::{MSG_TYPE_DATA, MAX_MESSAGE_BODY_SIZE};


/// 读取json文件并序列化为紧凑字符串
pub fn read_json_compact(json_file_path: &str) -> Result<String> {
    let file_content = fs::read_to_string(json_file_path)
        .map_err(|e| anyhow::anyhow!("读取文件失败: {:?}", e))?;
    let json_value: serde_json::Value = serde_json::from_str(&file_content)
        .map_err(|e| anyhow::anyhow!("解析JSON失败: {:?}", e))?;
    serde_json::to_string(&json_value).map_err(|e| anyhow::anyhow!("序列化JSON失败: {:?}", e))
}

/// 对长字符串进行压缩，返回Vec<u8>和压缩后长度
pub fn compress_string(data: &str) -> Result<(Vec<u8>, usize)> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data.as_bytes()).map_err(|e| anyhow::anyhow!("压缩写入失败: {:?}", e))?;
    let compressed = encoder.finish().map_err(|e| anyhow::anyhow!("压缩完成失败: {:?}", e))?;
    let len = compressed.len();
    Ok((compressed, len))
}

/// 对Vec<u8>数据解压，返回字符串和长度
#[allow(dead_code)]
pub fn decompress_to_string(data: &[u8]) -> Result<(String, usize)> {
    let mut decoder = ZlibDecoder::new(data);
    let mut s = String::new();
    decoder.read_to_string(&mut s).map_err(|e| anyhow::anyhow!("解压失败: {:?}", e))?;
    let len = s.len();  
    Ok((s, len))
}


/// 将压缩后的字节数组分片并包装为 MessagePacket 数组
pub fn wrap_message_packets(data: Vec<u8>) -> Vec<MessagePacket> {
    let total_len: usize = data.len();
    let mut packets: Vec<MessagePacket> = Vec::new();
    
    // 计算 字节数组 分片数量
    let chunk_count = (total_len + MAX_MESSAGE_BODY_SIZE - 1) / MAX_MESSAGE_BODY_SIZE;
    for i in 0..chunk_count {
        let start = i * MAX_MESSAGE_BODY_SIZE;
        let mut packet = MessagePacket::new(
            MSG_TYPE_DATA,
            total_len as u32,
            i as u32,
            chunk_count as u32,
        );
        packet.from_slice(&data, start);
        packets.push(packet);
    }
    packets
}

/// 将 MessagePacket 数组的 body 合并为一个 Vec<u8>
pub fn combine_message_bodies(packets: &[MessagePacket]) -> Vec<u8> {
    let mut combined: Vec<u8> = Vec::new();
    for packet in packets {
        combined.extend_from_slice(&packet.body);
    }
    combined
}