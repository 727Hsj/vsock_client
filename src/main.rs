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
use anyhow::Result;

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
        eprintln!("使用格式： <program> --save <json_file_path>");
        eprintln!("使用格式： <program> --dump <json_file_path>");
        return Err("参数错误".into());
    }

    let command: String = args[1].clone();                      // 命令
    let json_file_path: String = args[2].clone();               // 读取 或者 存储 的 JSON 文件路径
 


    println!("[Main] 正在启动 vsock 线程...");

    match command.as_str() {
        "--save" | constants::SAVE_PROCESS => {
            let message_str = std::fs::read_to_string(&json_file_path)?;
            send_process(message_str)?;
        },

        "--dump" | constants::DUMP_PROCESS => {
            let dumped_data = dump_process()?;

            // 处理转储的数据
            if !dumped_data.is_empty() {
                // 反序列化为 JSON Value
                let json_value: serde_json::Value = serde_json::from_slice(&dumped_data)
                    .map_err(|e| anyhow::anyhow!("反序列化失败: {:?}", e))?;
                
                // 格式化为漂亮的 JSON 字符串
                let pretty_json = serde_json::to_string_pretty(&json_value)?;
                
                // 写入文件 (这里使用命令行参数传入的路径，如果需要固定为 test.json，可以改为 "test.json")
                std::fs::write(&json_file_path, pretty_json)?;
                println!("已将数据保存到 {}", json_file_path);
            } else {
                println!("未接收到数据");
            }
        },
        _ => {
            println!("未知参数，请使用 --save 或 --dump");
        }
    } 

    println!("\n[Main] vsock线程已启动，等待完成...\n");

    
    println!("\n[Main] ✓ 所有线程已结束。");
    println!("[Main] 客户端退出。");

    Ok(())
}


pub fn send_process(message_str: String) -> Result<()> {

    // 压缩字符串
    let (compressed_data, compressed_len) = data_process::compress_string(&message_str)?;
    println!("字符串压缩前后长度是：{}-->{}", message_str.len(), compressed_len);

    // 将压缩字符串包装成 message_packet 数组
    let mut msg_packets: Vec<MessagePacket>  = data_process::wrap_message_packets(compressed_data);

    for packet in &mut msg_packets {
            packet.header.set_message_id(1);
    }

    client_thread_save::client_thread(msg_packets, DEFAULT_SERVER_CID, DEFAULT_SERVER_PORT, constants::SAVE_PROCESS_COMMAND).unwrap_or_else(|e| {
            eprintln!("Save 线程 出现错误: {:?}", e);
    });
   
    Ok(())
}

pub fn dump_process() -> Result<Vec<u8>> {

    let ret = client_thread_dump::client_thread(DEFAULT_SERVER_CID, DEFAULT_SERVER_PORT, constants::DUMP_PROCESS_COMMAND);

    // 错开启动时间
    thread::sleep(Duration::from_millis(300));

    ret
}