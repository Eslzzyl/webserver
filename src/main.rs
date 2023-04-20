mod exception;
mod param;
mod config;

use param::*;

// use std::net::Ipv4Addr;
// use std::net::SocketAddrV4;
// use std::net::TcpListener;
// use std::net::TcpStream;
use std::io;
use std::fs;

use async_std::task::spawn;
use async_std::prelude::*;
use async_std::net::Ipv4Addr;
use async_std::net::SocketAddrV4;
use async_std::net::TcpListener;
use async_std::net::TcpStream;

use futures::stream::StreamExt;

// const CRLF: &str = "\r\n";

#[async_std::main]
async fn main() -> io::Result<()> {
    // 监听端口
    let port: u16 = 7878;
    // 地址，本地调试用127.0.0.1
    let address = Ipv4Addr::new(127, 0, 0, 1);
    // 拼接socket
    let socket = SocketAddrV4::new(address, port);

    let listener = match TcpListener::bind(socket).await {
        Ok(listener) => listener,
        Err(e) => {
            println!("Failed to bind port: {}", port);
            panic!("{}", e);
        }
    };

    // TcpListener::incoming函数返回迭代器，等价于无限循环地调用TcpListener::accept
    listener
        .incoming()
        .for_each_concurrent(/* limit */ None, |tcpstream| async move {
            let tcpstream = tcpstream.unwrap();
            spawn(handle_connection(tcpstream));
        }).await;

    Ok(())
}

async fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).await.unwrap();

    let get = b"GET / HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", HTML_INDEX)
    } else {
        ("HTTP/1.1 404 NOT FOUND", HTML_404)
    };

    let contents = fs::read_to_string(filename).unwrap();

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write(response.as_bytes()).await.unwrap();
    stream.flush().await.unwrap();
}