// src/client_thread_send.rs
use vsock::{VsockStream, VsockAddr};
use std::io::{Read, Write};
use std::fs;
use std::thread;
use std::time::Duration;
use crate::utils;

const MESSAGE_INTERVAL_MS: u64 = 100;

pub fn client_thread(json_file_path: String, server_cid: u32, server_port: u32) {
    println!("[Client-Thread-For-Save] 黑匣子存储正在启动...");
 
    // 连接到服务器
    let addr = VsockAddr::new(server_cid, server_port);
    let mut stream = match VsockStream::connect(&addr) {
        Ok(s) => {
            println!("[Client-Thread-For-Save] ✓ 已连接到服务端 CID:{} Port:{}", server_cid, server_port);
            s
        }
        Err(e) => {
            eprintln!("[Client-Thread-For-Save] ✗ 连接失败: {:?}", e);
            return;
        }
    };

    // 读取 JSON 文件
    let file_content = match fs::read_to_string(&json_file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("[Client-Thread-For-Save] ✗ 读取文件失败 {}: {:?}", json_file_path, e);
            return;
        }
    };

    // 解析 JSON 并序列化为紧凑字符串
    let json_value: serde_json::Value = match serde_json::from_str(&file_content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[Client-Thread-For-Save] ✗ 解析文件 JSON 失败 {}: {:?}", json_file_path, e);
            return;
        }
    };

    let mut message = match serde_json::to_string(&json_value) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[Client-Thread-For-Save] ✗ 序列化 JSON 失败: {:?}", e);
            return;
        }
    };
    message.push('\n');

    // 发送 send
    match stream.write_all(message.as_bytes()) {
        Ok(_) => {
            println!("[Client-Thread-For-Save] → 已发送 JSON 内容，来自: {}", json_file_path);
        }
        Err(e) => {
            eprintln!("[Client-Thread-For-Save] ✗ 发送错误: {:?}", e);
            return;
        }
    }

    // 接收 echo
    let mut buffer = vec![0u8; 1024];
    match stream.read(&mut buffer) {
        Ok(0) => {
            eprintln!("[Client-Thread-For-Save] ✗ 服务器意外断开连接");
        }
        Ok(n) => {
            if let Ok(echo) = std::str::from_utf8(&buffer[..n]) {
                println!("[Client-Thread-For-Save] ← 已接收内容: {}", echo);
            } else {
                eprintln!("[Client-Thread-For-Save] ✗ 接收内容 UTF-8 解码失败, Echo bytes: {:?}", n);
                
            }
        }
        Err(e) => {
            eprintln!("[Client-Thread-For-Save] ✗ 接收错误: {:?}", e);
        }
    }

    // 间隔
    thread::sleep(Duration::from_millis(MESSAGE_INTERVAL_MS));

    // 优雅关闭连接
    utils::graceful_shutdown(&mut stream, "[Client-Thread-For-Save]");

    println!("[Client-Thread-For-Save] 完成。正在关闭连接。");
}