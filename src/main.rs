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
use log::{error, warn, info};
use log4rs;

use std::{
    net::{Ipv4Addr, SocketAddrV4},
    path::{Path, PathBuf},
    ffi::OsStr,
    time::Instant,
    sync::{Arc, Mutex},
};

use crate::param::HTML_INDEX;

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
        info!("新的连接：{}", addr);

        let active_connection_arc = Arc::clone(&active_connection);
        let root_clone = root.clone();
        let cache_arc = Arc::clone(&cache);
        info!("[ID{}]TCP连接已建立", id);
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
/// - `config`: Web服务器配置类型，在当前子线程建立时使用Arc<T>共享
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
    info!("[ID{}]HTTP请求接收完毕", id);

    // 启动timer
    let start_time = Instant::now();

    let request = Request::try_from(&buffer).unwrap();
    info!("[ID{}]成功解析HTTP请求", id);

    let (code, path, mime) = route(&request.path(), id, root).await;
    info!("[ID{}]HTTP路由解析完毕", id);


    // 如果path不存在，就返回404。使用Response::response_404
    let response = if code == 1 {
        warn!("[ID{}]请求的路径：{:?} 不存在，返回404响应", id, path);
        Response::response_404(&request, id, &cache)
    } else {
        let path_str = match path.to_str() {
            Some(s) => s,
            None => {
                error!("[ID{}]无法将路径{:?}转换为str", id, path);
                return;
            },
        };
        Response::from(path_str, &mime, &request, id, &cache)
    };
    info!("[ID{}]HTTP响应构建完成，服务端用时{}ms。", id, start_time.elapsed().as_millis());

    stream.write(&response).await.unwrap();
    stream.flush().await.unwrap();
    info!("[ID{}]HTTP响应已写回", id);
}

/// 路由解析函数
/// 
/// 参数：
/// - `path`：请求路径
/// - `config`：Web服务器配置类型
/// - `id`: 当前TCP连接的ID
/// 
/// 返回：
/// - `u8`: 状态码。0为正常，1为404
/// - `PathBuf`: 文件的完整路径
/// - `String`: MIME类型
async fn route(path: &str, id: u128, root: &str) -> (u8, PathBuf, String) {
    if path == "/" {
        info!("[ID{}]请求路径为根目录，返回index", id);
        let path = PathBuf::from(HTML_INDEX);
        return (0, path, "text/html".to_string());
    }
    let mut path_str = path.to_string();
    path_str.remove(0);
    let path = Path::new(&path_str);
    // 将路径和config.wwwroot拼接
    let root = Path::new(root);
    let path = root.join(path);
    info!("[ID{}]请求文件路径：{}", id, path.to_str().unwrap());
    // 根据文件名确定MIME
    let binding = path.clone();    // 很野的写法，extension会borrow path，导致没法正常返回。尝试寻找解决方法？
    let mime = match binding.extension() {
        Some(extension) => get_mime(extension),
        None => "text/plain",
    };
    info!("[ID{}]MIME类型: {}", id, mime);
    // 返回
    (!path.exists() as u8, path, mime.to_string())
}

/// MIME 保存了常见文件类型的映射关系
/// 
/// https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types/Common_types
fn get_mime(extension: &OsStr) -> &str {
    match extension.to_str().unwrap() {
        "aac" => "audio/aac",
        "abw" => "application/x-abiword",
        "apk" => "application/vnd.android.package-archive",
        "arc" => "application/x-freearc",
        "avi" => "video/x-msvideo",
        "avif" => "image/avif",
        "azw" => "application/vnd.amazon.ebook",
        "bin" => "application/octet-stream",
        "bmp" => "image/bmp",
        "bz" => "application/x-bzip",
        "bz2" => "application/x-bzip2",
        "cab" => "application/vnd.ms-cab-compressed",
        "cda" => "application/x-cdf",
        "csh" => "application/x-csh",
        "css" => "text/css;charset=utf-8",
        "csv" => "text/csv",
        "crx" => "application/x-chrome-extension",
        "deb" => "application/x-deb",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "eot" => "application/vnd.ms-fontobject",
        "epub" => "application/epub+zip",
        "exe" => "application/x-msdownload",
        "gif" => "image/gif",
        "gz" => "application/gzip",
        "htm" => "text/html;charset=utf-8",
        "html" => "text/html;charset=utf-8",
        "img" => "application/x-iso9660-image",
        "ico" => "image/x-icon",
        "ics" => "text/calendar",
        "iso" => "application/x-iso9660-image",
        "jar" => "application/java-archive",
        "js" => "text/javascript;charset=utf-8",
        "json" => "application/json",
        "jsonld" => "application/ld+json",
        "jpg" => "image/jpeg",
        "jpeg" => "image/jpeg",
        "mid" => "audio/x-midi",
        "midi" => "audio/x-midi",
        "mjs" => "text/javascript",
        "mkv" => "video/x-matroska",
        "mp3" => "audio/mpeg",
        "mp4" => "video/mp4",
        "mpeg" => "video/mpeg",
        "mpkg" => "application/vnd.apple.installer+xml",
        "msi" => "application/x-msdownload",
        "odp" => "application/vnd.oasis.opendocument.presentation",
        "ods" => "application/vnd.oasis.opendocument.spreadsheet",
        "odt" => "application/vnd.oasis.opendocument.text",
        "oga" => "audio/ogg",
        "ogv" => "video/ogg",
        "ogx" => "application/ogg",
        "opus" => "audio/opus",
        "otf" => "font/otf",
        "pdf" => "application/pdf",
        "png" => "image/png",
        "php" => "application/x-httpd-php",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "rar" => "application/x-rar-compressed",
        "rtf" => "application/rtf",
        "rpm" => "application/x-rpm",
        "sh" => "application/x-sh",
        "svg" => "image/svg+xml",
        "swf" => "application/x-shockwave-flash",
        "tar" => "application/x-tar",
        "tif" => "image/tiff",
        "tiff" => "image/tiff",
        "ts" => "video/mp2t",
        "txt" => "text/plain",
        "ttf" => "font/ttf",
        "vsd" => "application/vnd.visio",
        "wav" => "audio/wav",
        "wasm" => "application/wasm",
        "weba" => "audio/webm",
        "webm" => "video/webm",
        "webp" => "image/webp",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "xhtml" => "application/xhtml+xml",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "xml" => "text/xml",
        "xpi" => "application/x-xpinstall",
        "xul" => "application/vnd.mozilla.xul+xml",
        "zip" => "application/zip",
        "7z" => "application/x-7z-compressed",
        _ => "application/octet-stream",
    }
}