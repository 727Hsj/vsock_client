// src/client_thread_dump.rs
use vsock::{VsockStream, VsockAddr};
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;
use std::fs;
use crate::{constants, utils};

const MESSAGE_INTERVAL_MS: u64 = 100;

pub fn client_thread(json_file_path: String, server_cid: u32, server_port: u32) {
    println!("[Client-Thread-For-Dump] 正在启动...");
 
    // 连接到服务器
    let addr = VsockAddr::new(server_cid, server_port);
    let mut stream = match VsockStream::connect(&addr) {
        Ok(s) => {
            println!("[Client-Thread-For-Dump] ✓ 已连接到 CID:{} Port:{}", server_cid, server_port);
            s
        }
        Err(e) => {
            eprintln!("[Client-Thread-For-Dump] ✗ 连接失败: {:?}", e);
            return;
        }
    };

    // 发送 send
    let message = b"dump command\n";
    match stream.write_all(message) {
        Ok(_) => {
            println!("[Client-Thread-For-Dump] → 已发送命令");
        }
        Err(e) => {
            eprintln!("[Client-Thread-For-Dump] ✗ 发送错误: {:?}", e);
            return;
        }
    }

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
    fs::write(&json_file_path, "").unwrap();    // 先清空文件内容
    let mut buffer = vec![0u8; 4096];
    let mut received_items: Vec<serde_json::Value> = Vec::new();

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                eprintln!("[Client-Thread-For-Dump] ✗ 服务器意外断开连接");
                break;
            }
            Ok(n) => {
                let recv_str = std::str::from_utf8(&buffer[..n]);
                let recv_str = recv_str.unwrap_or_else(|e| {
                    eprintln!("[Client-Thread-For-Dump] ✗ UTF-8 解码失败: {:?}. Echo bytes: {:?}", e, n);
                    ""
                }).trim();

                if recv_str == constants::NO_FIND_DUMP_RESPONSE {
                    break;
                } else if recv_str == constants::FIND_DUMP_RESPONSE {
                    break;
                } else if recv_str == "" {
                    continue;
                } else {
                    // 尝试解析为 JSON
                    match serde_json::from_str::<serde_json::Value>(recv_str) {
                        Ok(val) => {
                            received_items.push(val);
                            // 发送确认标识 "OK"
                            stream.write_all(constants::ECHO_DUMP_RESPONSE_SUCCESS).unwrap();
                        },
                        Err(e) => {
                            eprintln!("[Client-Thread-For-Dump] ✗ 解析 JSON 失败: {:?}. 接收内容: {}", e, recv_str);
                             // 即使解析失败也发送 OK 以继续
                            stream.write_all(constants::ECHO_DUMP_RESPONSE_FAIL).unwrap();
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("[Client-Thread-For-Dump] ✗ 接收错误: {:?}", e);
                return;
            }
        }
    }

    // 将所有接收到的条目写入文件
    if !received_items.is_empty() {
        let pretty_json = serde_json::to_string_pretty(&received_items).unwrap();
        fs::write(&json_file_path, pretty_json).unwrap();
        println!("[Client-Thread-For-Dump] 已保存 {} 条记录到 {}", received_items.len(), json_file_path);
    } else {
        println!("[Client-Thread-For-Dump] 暂无需要接收的 dump 信息");
    }

    // 间隔
    thread::sleep(Duration::from_millis(MESSAGE_INTERVAL_MS));

    // 优雅关闭连接
    utils::graceful_shutdown(&mut stream, "[Client-Thread-For-Dump]");

    println!("[Client-Thread-For-Dump] 完成。正在关闭连接。");
}
