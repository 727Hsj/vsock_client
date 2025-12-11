use std::env;
use std::thread;
use std::time::Duration;
 
mod client_thread_save;
mod client_thread_dump;

const DEFAULT_SERVER_CID: u32 = 103;  // 默认连接 Host (CID=2)
const DEFAULT_SERVER_PORT: u32 = 1234;
 
 


fn main() {
    //read args from command line as server cid, port, num connections, messages per connection
    // If args are not provided, use default values


    // 假设使用客户端的命令格式是  .target/debug/vsock_client -f path/to/file.json 103 1234
    let args: Vec<String> = env::args().collect();

    // 判断参数数量
    if args.len() < 3 {
        eprintln!("错误：请提供 必要的参数。");
        eprintln!("使用格式： <program> --save <json_file_path> [server_cid] [server_port]");
        eprintln!("使用格式： <program> --dump <json_file_path> [server_cid] [server_port]");
        return;
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
    println!("[Main] Spawning a thread for vsock...");

    match command.as_str() {
        "--save" => {
            let handle = thread::spawn(move || {
                client_thread_save::client_thread(json_file_path, server_cid, server_port);
            });
            handles.push(handle);
            // 错开启动时间
            thread::sleep(Duration::from_millis(300));
        },

        "--dump" => {
            let handle = thread::spawn(move || {
                client_thread_dump::client_thread(json_file_path, server_cid, server_port);
            });
            handles.push(handle);
            // 错开启动时间
            thread::sleep(Duration::from_millis(300));
        },
        _ => {
            println!("未知参数，请使用 --save 或 --dump");
        }
    } 

    println!("\n[Main] All threads spawned. Waiting for completion...\n");

    // 等待所有线程完成
    for (i, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok(_) => println!("[Main] Thread-{} completed", i),
            Err(e) => eprintln!("[Main] Thread-{} panicked: {:?}", i, e),
        }
    }
    println!("\n[Main] ✓ All threads finished.");
    println!("[Main] Client exiting.");
}
