#![allow(clippy::unused_io_amount)]

mod exception;
mod param;
mod config;
mod request;
mod response;

use request::Request;
use config::Config;

use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

use std::{net::{Ipv4Addr, SocketAddrV4}};
use std::path::{Path, PathBuf};
use std::ffi::OsStr;


#[tokio::main]
async fn main() {
    let config = Config::from_toml("files/config.toml");

    // 监听端口
    let port: u16 = config.port();
    // 地址，本地调试用127.0.0.1
    let address = Ipv4Addr::new(127, 0, 0, 1);
    // 拼接socket
    let socket = SocketAddrV4::new(address, port);

    // 执行bind
    let listener = match TcpListener::bind(socket).await {
        Ok(listener) => listener,
        Err(e) => {
            println!("Failed to bind port: {}", port);
            panic!("{}", e);
        }
    };

    loop {
        let (mut stream, _) = listener.accept().await.unwrap();

        // 这种写法太野了，之后尽量改一下。一定要避免clone
        let config_clone = config.clone();
        tokio::spawn(async move{
            handle_connection(stream, &config_clone).await;
        });
    }
}

async fn handle_connection(mut stream: TcpStream, config: &Config) {
    let mut buffer = vec![0; 1024];
    let mut bytes_read: usize = 0;

    // 等待tcpstream变得可读
    stream.readable().await.unwrap();

    match stream.try_read(&mut buffer) {
        Ok(n) => bytes_read = n,
        Err(e) => {
            println!("Error when reading from TCP stream!");
            panic!("{}", e);
        }
    }

    // println!("{}", String::from_utf8_lossy(&buffer));

    let request = Request::try_from(buffer).unwrap();
    dbg!(&request);

    let (is_valid, path, mime) = route(&request.path(), config);


    // 如果path不存在，就返回404。使用Response::response_404
    

    // let get = b"GET / HTTP/1.1\r\n";

    // let (status_line, filename) = if buffer.starts_with(get) {
    //     ("HTTP/1.1 200 OK", HTML_INDEX)
    // } else {
    //     ("HTTP/1.1 404 NOT FOUND", HTML_404)
    // };

    // let contents = fs::read_to_string(filename).unwrap();

    // let response = format!(
    //     "{}\r\nContent-Length: {}\r\n\r\n{}",
    //     status_line,
    //     contents.len(),
    //     contents
    // );

    // stream.write(response.as_bytes()).await.unwrap();
    // stream.flush().await.unwrap();
}

/// 返回值
/// bool: 是否有效，1为有效。
/// 第一个String: 文件的完整路径
/// 第二个String: MIME类型
fn route(path: &str, config: &Config) -> (bool, PathBuf, String) {
    // 将path转换为绝对路径
    let path = Path::new(path).canonicalize().unwrap();
    // 将路径和config.wwwroot拼接
    let binding = config.www_root();
    let root = Path::new(&binding);
    let path = root.join(path);
    // 根据文件名确定MIME
    let mime = match path.extension() {
        Some(extension) => get_mime(extension),
        None => "text/plain",
    };
    // 返回
    (path.exists(), path, mime.to_string())
}

/// MIME
fn get_mime(extension: &OsStr) -> &str {
    match extension.to_str().unwrap() {
        "html" => "text/html",
        "css" => "text/css",
        "js" => "text/javascript",
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