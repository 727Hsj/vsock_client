mod client_thread_save;
mod client_thread_dump;
mod utils;
mod constants;
mod protocol;
mod data_process;

use std::env;
use std::thread;
use std::time::Duration;
use std::error::Error;

use crate::protocol::MessagePacket;
 

const DEFAULT_SERVER_CID: u32 = 3;  // 默认连接 Host (CID=3)
const DEFAULT_SERVER_PORT: u32 = 1234;
const NUM_CONNECTIONS: usize = 1;


fn main() -> Result<(), Box<dyn Error>>{
    // 假设使用客户端的命令格式是  .target/debug/vsock_client -f path/to/file.json 103 1234
    let args: Vec<String> = env::args().collect();

    // 判断参数数量
    if args.len() < 3 {
        eprintln!("错误：请提供 必要的参数。");
        eprintln!("使用格式： <program> --save <json_file_path> [server_cid] [server_port]");
        eprintln!("使用格式： <program> --dump <json_file_path> [server_cid] [server_port]");
        return Err("参数错误".into());
    }

    let command: String = args[1].clone();                      // 命令
    let json_file_path: String = args[2].clone();               // 读取 或者 存储 的 JSON 文件路径
    let server_cid: u32 = if args.len() > 3 {                   // server cid
        args[3].parse::<u32>().unwrap_or(DEFAULT_SERVER_CID)
    } else {
        DEFAULT_SERVER_CID
    };
    let server_port: u32 = if args.len() > 4 {                 // server port   
        args[4].parse::<u32>().unwrap_or(DEFAULT_SERVER_PORT)
    } else {
        DEFAULT_SERVER_PORT
    };
 
    println!("╔════════════════════════════════════╗");
    println!("║    VSocket Test Client v1.0        ║");
    println!("╚════════════════════════════════════╝");
    println!();
    println!();
    println!("Configuration:");
    println!("  Server CID:  {}", server_cid);
    println!("  Server Port: {}", server_port);
    println!();
    println!();

    // 启动客户端线程来处理 vsock 连接
    let mut handles = Vec::new();
    println!("[Main] 正在启动 vsock 线程...");

    match command.as_str() {
        "--save" => {
            // 读取 解析 序列化 JSON 文件
            let message_str = data_process::read_json_compact(&json_file_path)?;

            // 压缩字符串
            let (compressed_data, compressed_len) = data_process::compress_string(&message_str)?;
            println!("字符串压缩前后长度是：{}-->{}", message_str.len(), compressed_len);

            // 将压缩字符串包装成 message_packet 数组
            let msg_packets: Vec<MessagePacket>  = data_process::wrap_message_packets(compressed_data);

            // 计算每个线程处理的 消息包 数量
            let packet_chunk_size: usize = (msg_packets.len() + NUM_CONNECTIONS  - 1) / NUM_CONNECTIONS;

            // 多线程传输 -----先单的把
            for i in 0..NUM_CONNECTIONS {
                let start = i * packet_chunk_size;
                let end = std::cmp::min(start + packet_chunk_size, msg_packets.len());
                println!("共{}个消息包，线程{}传输第{}-{}个数据包", msg_packets.len(), i+1, start, end);
                // 并发传输修改 message_id
                let mut my_chunk = msg_packets[start..end].to_vec();
                let message_id = i as u32; // 简单生成一个唯一ID
                for packet in &mut my_chunk {
                    packet.header.set_message_id(message_id);
                }

                let handle = thread::spawn(move || {
                    client_thread_save::client_thread(i, my_chunk, server_cid, server_port).unwrap_or_else(|e| {
                        eprintln!("Save 线程 {} 出现错误: {:?}", i, e);
                    });
                });
                handles.push(handle);
                thread::sleep(Duration::from_millis(300));
            }
        },

        "--dump" => {
            let handle = thread::spawn(move || {
                client_thread_dump::client_thread(0, &json_file_path, server_cid, server_port).unwrap_or_else(|e| {
                    eprintln!("Dump 线程 出现错误: {:?}", e);
                });
            });
            handles.push(handle);
            // 错开启动时间
            thread::sleep(Duration::from_millis(300));
        },
        _ => {
            println!("未知参数，请使用 --save 或 --dump");
        }
    } 

    println!("\n[Main] vsock线程已启动，等待完成...\n");

    // 等待所有线程完成
    for (i, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok(_) => println!("[Main] 线程-{} 已完成", i),
            Err(e) => eprintln!("[Main] 线程-{} 异常退出: {:?}", i, e),
        }
    }
    println!("\n[Main] ✓ 所有线程已结束。");
    println!("[Main] 客户端退出。");

    Ok(())
}
