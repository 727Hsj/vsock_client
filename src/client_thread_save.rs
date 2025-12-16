// src/client_thread_save.rs
use vsock::{VsockStream, VsockAddr};
use crate::utils;
use crate::protocol::MessagePacket;
use crate::protocol::utils as protocol_utils;
use anyhow::Result;
use crate::constants;
use std::time::Duration;


pub fn client_thread(msg_packets: Vec<MessagePacket>, server_cid: u32, server_port: u32, command: u8) -> Result<()> {
    let msg_id = 1; // 简单起见，使用固定的 msg_id
    println!("[Client-{}] 黑匣子客户端线程正在启动...", msg_id);
 
    // 连接到服务器
    let addr = VsockAddr::new(server_cid, server_port);
    let mut stream = match VsockStream::connect(&addr) {
        Ok(s) => {
            println!("[Client-{}] ✓ 已连接到服务端 CID:{} Port:{}", msg_id, server_cid, server_port);
            s
        }
        Err(e) => {
            return Err(anyhow::anyhow!("[Client-{}] ✗ 连接失败: {:?}", msg_id, e));
        }
    };

    println!("[Client-{}] 准备发送数据，负责 {} 个消息包", msg_id, msg_packets.len());

    // 1. 发送开始消息, 同时携带 msg_id 作为 message_id， 命令编号 作为 reserved
    protocol_utils::send_start_message(&mut stream, msg_id as u32, command)?;

    // 2. 等待ACK
    if !protocol_utils::wait_for_ack(&mut stream, msg_id as u32) {
        return Err(anyhow::anyhow!("Server not ready"));
    }

    // 3. 分片发送数据
    for (index, datamsg) in msg_packets.iter().enumerate() {
        println!("[Client-{}] 发送第 {} 数据包 ", msg_id, index);
        protocol_utils::send_data_message(&mut stream, datamsg)?;
    
        if !protocol_utils::wait_for_ack(&mut stream, msg_id as u32) {
            return Err(anyhow::anyhow!("Transmission interrupted"));
        }
    }

    // 4. 发送结束消息
    protocol_utils::send_end_message(&mut stream, msg_id as u32)?;

    // 5. 等待ACK
    if !protocol_utils::wait_for_ack(&mut stream, msg_id as u32) {
        return Err(anyhow::anyhow!("Server not ready"));
    }
    println!("[Client-{}] ✓ 传输完成，服务器已确认", msg_id);

    // 6. 优雅关闭连接
    utils::graceful_shutdown(&mut stream, "[Client-{}]");

    Ok(())
}