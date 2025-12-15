// src/constants.rs

// 协议相关常量
// pub const SHUTDOWN_COMMAND: &[u8] = b"shutdown\n";
pub const SHUTDOWN_RESPONSE: &str = "shutdowned";
pub const NO_FIND_DUMP_RESPONSE: &str = "nofind";
pub const FIND_DUMP_RESPONSE: &str = "find";
pub const ECHO_DUMP_RESPONSE_SUCCESS: &[u8] = b"ok and success\n";
pub const ECHO_DUMP_RESPONSE_FAIL: &[u8] = b"ok but fail\n";

pub const SAVE_COMMAND: u8 = 0x01;
pub const DUMP_COMMAND: u8 = 0x02;
// pub const SHUTDOWN_COMMAND: u8 = 0x03;

