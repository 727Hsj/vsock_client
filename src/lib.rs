pub mod client_thread_save;
pub mod client_thread_dump;
pub mod utils;
pub mod constants;
pub mod protocol;
pub mod data_process;

use std::sync::Mutex;
use anyhow::Result;
use crate::protocol::MessagePacket;

// 全局互斥锁，用于保护 Vsock 通信不被并发竞争
static VSOCK_MUTEX: Mutex<()> = Mutex::new(());

pub const DEFAULT_SERVER_CID: u32 = 103;  // 默认连接 Host (CID=3)
pub const DEFAULT_SERVER_PORT: u32 = 1234;

pub fn send_process(message_str: String) -> Result<()> {

    // 压缩字符串
    //let (compressed_data, compressed_len) = data_process::compress_string(&message_str)?;
    let compressed_data = vec![0xAB; 1 * 1024 * 1024];
    let compressed_len = compressed_data.len();
    println!("字符串压缩前后长度是：{}-->{}", message_str.len(), compressed_len);
    
    // 将压缩字符串包装成 message_packet 数组
    let mut msg_packets: Vec<MessagePacket>  = data_process::wrap_message_packets(compressed_data);

    for packet in &mut msg_packets {
            packet.header.set_message_id(1);
    }

    // 获取锁，保护通信过程
    {
        let _guard = VSOCK_MUTEX.lock().map_err(|e| anyhow::anyhow!("获取锁失败: {:?}", e))?;
        client_thread_save::client_thread(msg_packets, DEFAULT_SERVER_CID, DEFAULT_SERVER_PORT, constants::SAVE_PROCESS_COMMAND).unwrap_or_else(|e| {
                eprintln!("Save 线程 出现错误: {:?}", e);
        });
    }
   
    Ok(())
}

pub fn dump_process() -> Result<Vec<u8>> {
    // 获取锁，保护通信过程
    let _guard = VSOCK_MUTEX.lock().map_err(|e| anyhow::anyhow!("获取锁失败: {:?}", e))?;

    let ret = client_thread_dump::client_thread(DEFAULT_SERVER_CID, DEFAULT_SERVER_PORT, constants::DUMP_PROCESS_COMMAND).unwrap_or_else(|e| {
        Vec::new()
    });
    Ok(ret)

}

// save crash字段
pub fn send_crash_log(message_str: String) -> Result<()> {

    // 压缩字符串
    let (compressed_data, compressed_len) = data_process::compress_string(&message_str)?;
    println!("字符串压缩前后长度是：{}-->{}", message_str.len(), compressed_len);

    // 将压缩字符串包装成 message_packet 数组
    let mut msg_packets: Vec<MessagePacket>  = data_process::wrap_message_packets(compressed_data);

    for packet in &mut msg_packets {
            packet.header.set_message_id(1);
    }

    // 获取锁，保护通信过程
    {
        let _guard = VSOCK_MUTEX.lock().map_err(|e| anyhow::anyhow!("获取锁失败: {:?}", e))?;
        client_thread_save::client_thread(msg_packets, DEFAULT_SERVER_CID, DEFAULT_SERVER_PORT, constants::SAVE_CRASH_LOG).unwrap_or_else(|e| {
                eprintln!("Save 线程 出现错误: {:?}", e);
        });
    }
   
    Ok(())
}


pub fn dump_crash_log() -> Result<Vec<u8>> {
    // 获取锁，保护通信过程
    let _guard = VSOCK_MUTEX.lock().map_err(|e| anyhow::anyhow!("获取锁失败: {:?}", e))?;

    let ret = client_thread_dump::client_thread(DEFAULT_SERVER_CID, DEFAULT_SERVER_PORT, constants::DUMP_CRASH_LOG).unwrap_or_else(|e| {
        Vec::new()
    });
    Ok(ret)

}