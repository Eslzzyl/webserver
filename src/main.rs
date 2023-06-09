#![allow(clippy::unused_io_amount)]

mod exception;
mod param;
mod config;
mod request;
mod response;

use request::Request;
use config::Config;
use response::Response;

use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use log4rs;
use log::{error, warn, info};

use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use std::time::Instant;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    let config = Config::from_toml("files/config.toml");
    info!("配置文件已载入");
    info!("www root：{}", config.www_root());

    // 监听端口
    let port: u16 = config.port();
    info!("服务端将在{}端口上监听Socket连接", port);
    // 地址，本地调试用127.0.0.1
    let address = Ipv4Addr::new(127, 0, 0, 1);
    info!("服务端将在{}地址上监听Socket连接", address);
    // 拼接socket
    let socket = SocketAddrV4::new(address, port);

    // 执行bind
    let listener = match TcpListener::bind(socket).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("无法绑定端口：{}，错误：{}", port, e);
            panic!();
        }
    };
    info!("端口{}绑定完成", port);

    let mut id: u128 = 0;

    let arc_config = Arc::new(config);

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        info!("新的TCP连接已建立，ID为{}", id);

        let arc_config_clone = Arc::clone(&arc_config);
        tokio::spawn(async move {
            handle_connection(stream, &arc_config_clone, id).await;
        });
        id += 1;
    }
}

/// 处理TCP连接
/// 
/// 参数：
/// - `stream`: 建立好的`TcpStream`
/// - `config`: Web服务器配置类型，在当前子线程建立时拷贝
/// - `id`: 当前TCP连接的ID
async fn handle_connection(mut stream: TcpStream, config: &Config, id: u128) {
    let mut buffer = vec![0; 1024];

    // 等待tcpstream变得可读
    stream.readable().await.unwrap();

    match stream.try_read(&mut buffer) {
        Err(e) => {
            error!("[ID{}]读取TCPStream时遇到错误：{}", id, e);
            panic!();
        },
        _ => {},
    }
    info!("[ID{}]HTTP请求接收完毕", id);

    // 启动timer
    let start_time = Instant::now();

    let request = Request::try_from(buffer).unwrap();
    info!("[ID{}]成功解析HTTP请求", id);

    let (code, path, mime) = route(&request.path(), config, id);
    info!("[ID{}]HTTP路由解析完毕", id);


    // 如果path不存在，就返回404。使用Response::response_404
    let response = if code == 1 {
        warn!("[ID{}]请求的路径：{:?} 不存在，返回404响应", id, path);
        Response::response_404(&request, id)
    } else {
        let path_str = match path.to_str() {
            Some(s) => s,
            None => {
                error!("[ID{}]无法将路径{:?}转换为str", id, path);
                return;
            },
        };
        Response::from(path_str, &mime, &request, id)
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
fn route(path: &str, config: &Config, id: u128) -> (u8, PathBuf, String) {
    if path == "/" {
        info!("[ID{}]请求路径为根目录，返回index", id);
        let path = PathBuf::from("./files/html/index.html");
        return (0, path, "text/html".to_string());
    }
    let mut path_str = path.to_string();
    path_str.remove(0);
    let path = Path::new(&path_str);
    // 将路径和config.wwwroot拼接
    let binding = config.www_root();
    let root = Path::new(&binding);
    let path = root.join(path);
    info!("[ID{}]请求文件路径：{}", id, path.to_str().unwrap());
    // 根据文件名确定MIME
    let binding = path.clone();    // 很野的写法，extension会borrow path，导致没法正常返回。尝试寻找解决方法？
    let mime = match binding.extension() {
        Some(extension) => get_mime(extension),
        None => "text/plain",
    };
    info!("[ID{}]MIME类型：{}", id, mime);
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