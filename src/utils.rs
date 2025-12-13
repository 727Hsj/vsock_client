// src/utils.rs
use vsock::VsockStream;
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;
use crate::constants;

const MAX_BUFFER_SIZE: usize = 64;
const MAX_RETRIES: usize = 3;

// 发送 shutdown 请求
pub fn graceful_shutdown(stream: &mut VsockStream, log_prefix: &str) {

    if let Err(e) = stream.write_all(constants::SHUTDOWN_COMMAND) {
        eprintln!("{} ✗ 发送 shutdown 请求失败: {:?}", log_prefix, e);
    }

    // 等待服务端回复
    let mut buffer = vec![0u8; MAX_BUFFER_SIZE];
    let mut should_shutdown = false;

    for i in 0..MAX_RETRIES {
        match stream.read(&mut buffer) {
            Ok(0) => {
                 eprintln!("{} ✗ 服务端在发送确认前关闭了连接", log_prefix);
                 should_shutdown = true;
                 break;
            }
            Ok(n) => {
                let response = String::from_utf8_lossy(&buffer[..n]);
                if response.trim() == constants::SHUTDOWN_RESPONSE {
                    println!("{} ← 收到服务端确认: {}", log_prefix, constants::SHUTDOWN_RESPONSE);
                }else {
                    eprintln!("{} ✗ 收到未知响应: {}", log_prefix, response);
                }
                should_shutdown = true;
                break;
            }
            Err(e) => {
                eprintln!("{} ✗ 等待服务端确认失败 (尝试 {}/{}): {:?}", log_prefix, i + 1, MAX_RETRIES, e);
                if i < MAX_RETRIES - 1 {
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }

    // 主动关闭连接
    if should_shutdown {
        if let Err(e) = stream.shutdown(std::net::Shutdown::Both) {
            eprintln!("{} ✗ 关闭连接失败: {:?}", log_prefix, e);
        }
    }
}
