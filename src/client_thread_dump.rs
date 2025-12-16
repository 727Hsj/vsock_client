// src/client_thread_dump.rs
use vsock::{VsockStream, VsockAddr};
use std::thread;
use std::time::Duration;
use crate::utils;
use crate::protocol::{MessagePacket, utils as protocol_utils};
use anyhow::Result;
use crate::protocol::consts;
use crate::data_process;

const MESSAGE_INTERVAL_MS: u64 = 100;

pub fn client_thread(server_cid: u32, server_port: u32, command: u8) -> Result<Vec<u8>> {
    let client_id = 1; // 简单起见，使用固定的 client_id
    println!("[Client-{}] 黑匣子客户端线程正在启动...", client_id);
 
    // 连接到服务器
    let addr = VsockAddr::new(server_cid, server_port);
    let mut stream = match VsockStream::connect(&addr) {
        Ok(s) => {
            println!("[Client-{}] ✓ 已连接到服务端 CID:{} Port:{}", client_id, server_cid, server_port);
            s
        }
        Err(e) => {
            return Err(anyhow::anyhow!("[Client-{}] ✗ 连接失败: {:?}", client_id, e));
        }
    };

    // 1. 发送开始消息, 同时携带 client_id 作为 message_id， 命令编号 作为 reserved
    protocol_utils::send_start_message(&mut stream, client_id as u32, command)?;

    // 2. 等待 ACK
    if !protocol_utils::wait_for_ack(&mut stream, client_id as u32) {
        return Err(anyhow::anyhow!("Server not ready"));
    }

    // 3. 发送 ACK
    protocol_utils::send_ack_message(&mut stream, client_id as u32)?;

    /*
     * 接收 echo
     * 
     * 有消息 和 无消息 两种情况
     * 
     * 有消息: 
     * 接收消息暂存4096字节的buffer中，有可能会多个消息。
     * 如果全部接收，会不会超过4096？会不会有可能传输失败？
     * 我们这样：
     * 服务端传输一个消息，客户端接收一个消息，处理成功后返回给服务端一个标识，然后循环；
     * 最后，服务端传输完毕，发送 "find" 标识，关闭；客户端收到 "find" 后，关闭连接。
     * 
     * 无消息:
     * 服务端，发送 "nofind" 标识，关闭；客户端收到 "nofind" 后，关闭连接。
     */

    let mut received_items: Vec<serde_json::Value> = Vec::new();

    loop {
        let (new_packets, all_end_received) = get_one_report(&mut stream, client_id)?;
        // 没有记录需要dump
        if new_packets.len() == 0 && all_end_received == true{
            println!("[Client-{}] 没有需要 dump 的记录", client_id);
            protocol_utils::send_ack_message(&mut stream, client_id as u32)?;
            protocol_utils::wait_for_ack(&mut stream, client_id as u32);
            break;
        }

        println!("[Client-{}] 成功接收 {} 个数据包", client_id, new_packets.len());

        // 将数据包的 body 合并成一个 Vec<u8>
        let combined_data: Vec<u8> = data_process::combine_message_bodies(&new_packets);

        // 字符串解压成 紧凑的 JSON 字符串
        let (decompressed_str, _len) = data_process::decompress_to_string(&combined_data)?;

        // 尝试解析为 JSON
        match serde_json::from_str::<serde_json::Value>(&decompressed_str) {
            Ok(val) => {
                received_items.push(val);
            },
            Err(e) => {
                eprintln!("[Client-{}] ✗ 解析 JSON 失败: {:?}.", client_id, e);
            }
        }
        if all_end_received {
            println!("[Client-{}] 收到 ALL_END 消息，所有传输结束", client_id);
            protocol_utils::send_ack_message(&mut stream, client_id as u32)?;
            protocol_utils::wait_for_ack(&mut stream, client_id as u32);
            break;
        }
    }

    let mut json_bytes: Vec<u8> = Vec::new();
    if !received_items.is_empty() {
        // 导出为 Vec<u8>
        json_bytes = serde_json::to_vec(&received_items)?;
    } else {
        println!("[Client-{}] 暂无需要接收的 dump 信息", client_id);
    }

    // 间隔
    thread::sleep(Duration::from_millis(MESSAGE_INTERVAL_MS));

    // 优雅关闭连接
    utils::graceful_shutdown(&mut stream, "[Client-Thread-For-Dump]");

    println!("[Client-Thread-For-Dump] 完成。正在关闭连接。");

    Ok(json_bytes)
}



fn get_one_report(stream: &mut VsockStream, client_id: usize) -> Result<(Vec<MessagePacket>, bool)> {

    let mut msg_packets: Vec<MessagePacket> = Vec::new();
    let mut received_data_size: u32 = 0;
    let mut is_all_reports_have_been_received = false;

    loop {

        // 1. 读取消息包
        let mut buf = [0u8; consts::MAX_MESSAGE_PACKET_SIZE];
        let packet = protocol_utils::receive_data_message(stream, &mut buf)?;

        // 2. 检查消息类型, 以及消息完整性
        if packet.header.msg_type == consts::MSG_TYPE_END 
            && msg_packets.len() == msg_packets[0].header.chunk_count as usize
            && msg_packets[0].header.total_size == received_data_size {
            println!("收到 END 消息，本次传输结束");
            
            protocol_utils::send_ack_message(stream, client_id as u32)?;

            protocol_utils::wait_for_ack(stream, client_id as u32);

            break;
        }

        if packet.header.msg_type == consts::MSG_TYPE_ALL_END {
            println!("收到 ALL_END 消息，所有传输结束");
            is_all_reports_have_been_received = true;
            break;
        }

        if packet.header.msg_type != consts::MSG_TYPE_DATA {
            println!("收到非 DATA/END 消息: type={}, 忽略", packet.header.msg_type);
            continue;
        }

        // 3. 验证消息ID
        if packet.header.message_id != client_id as u32 {
            return Err(anyhow::anyhow!("Unexpected message ID: {}, expected: {}", packet.header.message_id, client_id));
        }
        println!("收到分片: ID={}, Index={}/{}", packet.header.message_id, packet.header.chunk_index, packet.header.chunk_count);

        // 4. 统计消息体累计大小
        received_data_size += packet.body.len() as u32;

        // 5. 验证校验和
        let calculated_checksum = protocol_utils::calculate_checksum(&packet.body);
        if calculated_checksum != packet.header.checksum {
            return Err(anyhow::anyhow!("Checksum mismatch for chunk {}", packet.header.chunk_index));
        }

        // 6. 每收到5个分片发送一次ACK
        if packet.header.chunk_index % 5 == 4 {
            protocol_utils::send_ack_message(stream, client_id as u32)?;
        }

        // 7. 构造 MessagePacket 并存储
        msg_packets.push(packet);
    }


    Ok((msg_packets, is_all_reports_have_been_received))
}