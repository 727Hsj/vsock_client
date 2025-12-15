use vsock::VsockStream;
use std::io::{Read, Write};
use anyhow::Result;
use crate::protocol::message::MessagePacket;
use crate::protocol::consts::*;

pub fn calculate_checksum(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &x| acc.wrapping_add(x))
}

pub fn send_start_message(stream: &mut VsockStream, msg_id: u32, command: u8) -> Result<()> {
    let mut startmsg = MessagePacket::new(MSG_TYPE_START, 0, 0, 0);
    startmsg.header.set_message_id(msg_id);
    startmsg.header.set_reserved(command);  // 在待定字段设置命令编号
    println!("发送开始消息，长度是 20 吗？ ? {}", startmsg.get_len() == startmsg.to_bytes().len());

    stream.write_all(&startmsg.to_bytes())?;
    stream.flush()?;
    Ok(())
}

pub fn send_end_message(stream: &mut VsockStream, msg_id: u32) -> Result<()> {
    let mut endmsg = MessagePacket::new(MSG_TYPE_END, 0, 0, 0);
    endmsg.header.set_message_id(msg_id);
    println!("发送结束消息，长度是 20 吗？ ? {}", endmsg.get_len() == endmsg.to_bytes().len());

    stream.write_all(&endmsg.to_bytes())?;
    stream.flush()?;
    Ok(())
}

#[allow(dead_code)]
pub fn send_ack_message(stream: &mut VsockStream, msg_id: u32) -> Result<()> {
    let mut ackmsg = MessagePacket::new(MSG_TYPE_ACK, 0, 0, 0);
    ackmsg.header.set_message_id(msg_id);
    println!("发送ACK消息，长度是 20 吗？ ? {}", ackmsg.get_len() == ackmsg.to_bytes().len());

    stream.write_all(&ackmsg.to_bytes())?;
    stream.flush()?;
    Ok(())
}


pub fn send_data_message(stream: &mut VsockStream, datamsg: &MessagePacket) -> Result<()> {    
    // 写入消息头和数据
    stream.write_all(&datamsg.to_bytes())?;
    stream.flush()?;
    Ok(())
}

#[allow(dead_code)]
pub fn receive_data_message(stream: &mut VsockStream, buf: &mut [u8]) -> Result<MessagePacket> {
    let n = stream.read(buf)?;
    let packet = MessagePacket::from_bytes(&buf[..n]);
    Ok(packet)
}

pub fn wait_for_ack(stream: &mut VsockStream, expected_msg_id: u32) -> bool {
    let mut packet_buf = [0u8; MESSAGE_HEADER_SIZE];

    match stream.read_exact(&mut packet_buf) {
        Ok(_) => {
            let packet = MessagePacket::from_bytes(&packet_buf);
            if packet.header.msg_type == MSG_TYPE_ACK && packet.header.message_id == expected_msg_id {
                true
            } else {
                eprintln!("收到非预期ACK: type={}, id={}", packet.header.msg_type, packet.header.message_id);
                false
            }
        }
        Err(e) => {
            eprintln!("等待ACK错误: {:?}", e);
            false
        }
    }
}

// pub fn wait_final_response(stream: &mut VsockStream, expected_msg_id: u32) -> Result<String> {
//     let mut buffer = Vec::new();
//     let mut header_buf = [0u8; HEADER_SIZE];
    
//     // 读取响应消息头
//     self.stream.read_exact(&mut header_buf)?;
//     let header = Self::bytes_to_header(&header_buf);
    
//     if header.msg_type == MessageType::Data as u8 {
//         // 读取响应数据
//         let data_size = header.total_size as usize;
//         buffer.resize(data_size, 0);
//         self.stream.read_exact(&mut buffer)?;
        
//         String::from_utf8(buffer).map_err(|e| e.into())
//     } else {
//         Err(anyhow::anyhow!("Unexpected message type: {}", header.msg_type))
//     }
// }

