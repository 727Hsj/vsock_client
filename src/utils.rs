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
    // 主动关闭连接
    if let Err(e) = stream.shutdown(std::net::Shutdown::Both) {
        eprintln!("{} ✗ 关闭连接失败: {:?}", log_prefix, e);
    }
}

pub fn get_command_code(command: &str) -> u8 {
    match command {
        constants::SAVE => constants::SAVE_COMMAND,
        constants::DUMP => constants::DUMP_COMMAND,
        constants::SAVE_PROCESS => constants::SAVE_PROCESS_COMMAND,
        constants::DUMP_PROCESS => constants::DUMP_PROCESS_COMMAND,
        _ => 0,
    }
}
