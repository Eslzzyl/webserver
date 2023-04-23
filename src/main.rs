#![allow(clippy::unused_io_amount)]

mod exception;
mod param;
mod config;
mod request;
mod response;

use request::Request;
use config::Config;

use tokio::io::AsyncWriteExt;

use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::net::{TcpListener, TcpStream};

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
    dbg!(request);
    

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
fn route(path: &str, config: &Config) -> (bool, String, String) {
    
}