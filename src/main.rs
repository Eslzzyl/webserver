#![allow(clippy::unused_io_amount)]

mod exception;
mod param;
mod config;
mod request;
mod response;
mod cache;
mod util;

use request::Request;
use config::Config;
use response::Response;
use cache::FileCache;

use tokio::{
    net::{TcpListener, TcpStream},
    io::{
        AsyncWriteExt,
        AsyncBufReadExt,
        BufReader
    },
    runtime::Builder,
};
use log::{error, warn, info, debug};
use log4rs;

use std::{
    net::{Ipv4Addr, SocketAddrV4},
    path::{Path, PathBuf},
    time::Instant,
    sync::{Arc, Mutex},
};

use crate::{
    param::HTML_INDEX,
    exception::Exception,
};

#[tokio::main]
async fn main() {
    // 初始化日志系统
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    // 加载配置文件
    let config = Config::from_toml("files/config.toml");
    info!("配置文件已载入");
    let root = config.www_root().to_string();
    info!("www root: {}", &root);

    // 设置工作线程数量
    let worker_threads = config.worker_threads();
    let runtime = Builder::new_multi_thread()
        .worker_threads(worker_threads)
        .build()
        .unwrap();

    // 初始化文件缓存
    let cache_size = config.cache_size();
    let cache = Arc::new(
        Mutex::new(FileCache::from_capacity(cache_size))
    );

    // 监听端口
    let port: u16 = config.port();
    info!("服务端将在{}端口上监听Socket连接", port);
    // 地址，本地调试用127.0.0.1
    let address = match config.local() {
        true => Ipv4Addr::new(127, 0, 0, 1),
        false => Ipv4Addr::new(0, 0, 0, 0)
    };
    info!("服务端将在{}地址上监听Socket连接", address);
    // 拼接socket
    let socket = SocketAddrV4::new(address, port);

    // 执行bind
    let listener = match TcpListener::bind(socket).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("无法绑定端口：{}，错误：{}", port, e);
            panic!("无法绑定端口：{}，错误：{}", port, e);
        }
    };
    info!("端口{}绑定完成", port);

    // 停机命令标志
    let shutdown_flag = Arc::new(Mutex::new(false));
    // 活跃连接计数
    let active_connection = Arc::new(Mutex::new(0u32));

    // 启动异步命令处理任务
    runtime.spawn({
        let shutdown_flag = Arc::clone(&shutdown_flag);
        let active_connection = Arc::clone(&active_connection);
        async move {
            let stdin = tokio::io::stdin();
            let mut reader = BufReader::new(stdin);
            let mut input = String::new();
            loop {
                input.clear();
                // 在这里处理命令，可以调用服务器的相关函数或执行其他操作
                if let Ok(_) = reader.read_line(&mut input).await {
                    let cmd = input.trim();
                    match cmd {
                        "stop" => {
                            // 如果收到 "stop" 命令，则设置停机标志
                            let mut flag = shutdown_flag.lock().unwrap();
                            *flag = true;
                            break;
                        },
                        "help" => {
                            println!("== Webserver Help ==");
                            println!("输入stop并再发出一次连接请求以停机");
                            println!("输入status以查看当前服务器状态");
                            println!("====================");
                        },
                        "status" => {
                            let active_count = *active_connection.lock().unwrap();
                            println!("== Webserver 状态 ===");
                            println!("当前连接数: {}", active_count);
                            println!("====================");
                        },
                        _ => {
                            println!("无效的命令：{}", cmd);
                        }
                    }
                } else {
                    break;
                }
            }
        }
    });

    let mut id: u128 = 0;

    loop {
        // 检查停机标志，如果设置了停机标志，退出循环
        if *shutdown_flag.lock().unwrap() {
            break;
        }
        let (mut stream, addr) = listener.accept().await.unwrap();
        debug!("新的连接：{}", addr);

        let active_connection_arc = Arc::clone(&active_connection);
        let root_clone = root.clone();
        let cache_arc = Arc::clone(&cache);
        debug!("[ID{}]TCP连接已建立", id);
        tokio::spawn(async move {
            {
                let mut lock = active_connection_arc.lock().unwrap();
                *lock += 1;
            }
            handle_connection(&mut stream, id, &root_clone, cache_arc).await;
            {
                let mut lock = active_connection_arc.lock().unwrap();
                *lock -= 1;
            }
        });
        id += 1;
    }
}

/// 处理TCP连接
/// 
/// 参数：
/// - `stream`: 建立好的`TcpStream`
/// - `config`: Web服务器配置类型，在当前子线程建立时使用`Arc<T>`共享
/// - `id`: 当前TCP连接的ID
async fn handle_connection(stream: &mut TcpStream, id: u128, root: &str, cache: Arc<Mutex<FileCache>>) {
    let mut buffer = vec![0; 1024];

    // 等待tcpstream变得可读
    stream.readable().await.unwrap();

    match stream.try_read(&mut buffer) {
        Ok(0) => return,
        Err(e) => {
            error!("[ID{}]读取TCPStream时遇到错误: {}", id, e);
            panic!();
        },
        _ => {},
    }
    debug!("[ID{}]HTTP请求接收完毕", id);

    // 启动timer
    let start_time = Instant::now();

    let request = Request::try_from(&buffer, id).unwrap();
    debug!("[ID{}]成功解析HTTP请求", id);

    let result = route(&request.path(), id, root).await;
    debug!("[ID{}]HTTP路由解析完毕", id);


    // 如果path不存在，就返回404。使用Response::response_404
    let response = match result {
        Ok(path) => {
            let path_str = match path.to_str() {
                Some(s) => s,
                None => {
                    error!("[ID{}]无法将路径{:?}转换为str", id, path);
                    return;
                },
            };
            Response::from(path_str, &request, id, &cache)
        },
        Err(Exception::FileNotFound) => {
            warn!("[ID{}]请求的路径：{} 不存在，返回404响应", id, &request.path());
            Response::response_404(&request, id)
        },
        Err(e) => {
            panic!("非法的错误类型：{}", e);
        }
    };

    debug!("[ID{}]HTTP响应构建完成，服务端用时{}ms。",
        id,
        start_time.elapsed().as_millis()
    );

    info!("[ID{}] {}, {}, {}, {}, {}, {}, ", id,
        request.version(),
        request.path(),
        request.method(),
        response.status_code(),
        response.information(),
        request.user_agent(),
    );

    stream.write(&response.as_bytes()).await.unwrap();
    stream.flush().await.unwrap();
    debug!("[ID{}]HTTP响应已写回", id);
}

/// 路由解析函数
/// 
/// ## 参数：
/// - `path`：请求路径
/// - `config`：Web服务器配置类型
/// - `id`: 当前TCP连接的ID
/// 
/// ## 返回：
/// - `u8`: 状态码。0为正常，1为404
/// - `PathBuf`: 文件的完整路径
/// - `String`: MIME类型
async fn route(path: &str, id: u128, root: &str) -> Result<PathBuf, Exception> {
    if path == "/" {
        debug!("[ID{}]请求路径为根目录，返回index", id);
        let path = PathBuf::from(HTML_INDEX);
        return Ok(path)
    }
    let mut path_str = path.to_string();
    path_str.remove(0);
    let path = Path::new(&path_str);
    // 将路径和config.wwwroot拼接
    let root = Path::new(root);
    let path = root.join(path);
    debug!("[ID{}]请求文件路径：{}", id, path.to_str().unwrap());
    match path.exists() {
        true => Ok(path),
        false => Err(Exception::FileNotFound),
    }
}
