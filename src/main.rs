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

#[tokio::main]
async fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    let config = Config::from_toml("files/config.toml");
    info!("配置文件已载入");

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

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        info!("新的TCP连接已建立，ID为{}", id);

        // 这种写法太野了，之后尽量改一下。一定要避免clone
        let config_clone = config.clone();
        tokio::spawn(async move{
            handle_connection(stream, &config_clone, id).await;
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
    info!("[ID{}]TCPStream可读，即将读入HTTP请求", id);

    match stream.try_read(&mut buffer) {
        Err(e) => {
            error!("[ID{}]读取TCPStream时遇到错误：{}", id, e);
            panic!();
        },
        _ => {},
    }
    info!("[ID{}]HTTP请求接收完毕", id);

    // println!("{}", String::from_utf8_lossy(&buffer));

    let request = Request::try_from(buffer).unwrap();
    info!("[ID{}]成功解析HTTP请求", id);
    // dbg!(&request);

    let (code, path, mime) = route(&request.path(), config, id);
    // dbg!(&code);
    // dbg!(&path);
    // dbg!(&mime);
    info!("[ID{}]HTTP路由解析完毕", id);


    // 如果path不存在，就返回404。使用Response::response_404
    let response = if code == 1 {
        warn!("[ID{}]请求的路径：{:?} 不存在，返回404响应", id, path);
        Response::response_404()
    } else {
        let path_str = match path.to_str() {
            Some(s) => s,
            None => {
                error!("[ID{}]无法将路径{:?}转换为str", id, path);
                return;
            },
        };
        Response::from(path_str, &mime)
    };
    info!("[ID{}]HTTP响应构建完成", id);
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
    // 将path转换为绝对路径
    // dbg!(&path);
    let path = Path::new(path).canonicalize().unwrap();
    // 将路径和config.wwwroot拼接
    let binding = config.www_root();
    let root = Path::new(&binding);
    let path = root.join(path);
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

/// MIME
/// 
/// 全是Copilot生成的，没仔细检查过
fn get_mime(extension: &OsStr) -> &str {
    match extension.to_str().unwrap() {
        "html" => "text/html;charset=utf-8",
        "css" => "text/css;charset=utf-8",
        "js" => "text/javascript;charset=utf-8",
        "jpg" => "image/jpeg",
        "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "ico" => "image/x-icon",
        "svg" => "image/svg+xml",
        "ttf" => "font/ttf",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "eot" => "application/vnd.ms-fontobject",
        "otf" => "font/otf",
        "json" => "application/json",
        "pdf" => "application/pdf",
        "mp4" => "video/mp4",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "txt" => "text/plain",
        "xml" => "text/xml",
        "zip" => "application/zip",
        "rar" => "application/x-rar-compressed",
        "7z" => "application/x-7z-compressed",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",
        "exe" => "application/x-msdownload",
        "msi" => "application/x-msdownload",
        "cab" => "application/vnd.ms-cab-compressed",
        "iso" => "application/x-iso9660-image",
        "img" => "application/x-iso9660-image",
        "apk" => "application/vnd.android.package-archive",
        "crx" => "application/x-chrome-extension",
        "xpi" => "application/x-xpinstall",
        "deb" => "application/x-deb",
        "rpm" => "application/x-rpm",
        "bin" => "application/octet-stream",
        "swf" => "application/x-shockwave-flash",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    }
}