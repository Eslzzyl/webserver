use crate::exception::Exception;
use crate::param::*;

#[derive(Debug, Clone)]
pub struct Request {
    method: HttpRequestMethod,
    path: String,
    version: HttpVersion,
    user_agent: String,
    accept_encoding: Vec<HttpEncoding>,  // 压缩编码，可以支持多种编码，如果该vec为空说明不支持压缩
}

impl Request {

    /// 生成一个空的Request对象，各成员默认值为：
    /// 
    /// - 请求方法：`Get`
    /// - 路径：`""`（空字符串）
    /// - HTTP版本：`1.1`
    /// - User-Agent：`""`（空字符串）
    /// - Accept-Encoding：空
    pub fn new() -> Self {
        Self {
            method: HttpRequestMethod::Get,
            path: "".to_string(),
            version: HttpVersion::V1_1,
            user_agent: "".to_string(),
            accept_encoding: Vec::<HttpEncoding>::new(),
        }
    }

    /// 尝试通过字节流解析Request
    /// 
    /// 参数：
    /// - `buffer`: 来自客户浏览器的请求报文，用字节流表示
    pub fn try_from(buffer: Vec<u8>) -> Result<Self, Exception> {
        let request_string = match String::from_utf8(buffer) {
            Ok(string) => string,
            Err(_) => {
                println!("Error when parsing request!");
                return Err(Exception::RequestIsNotUtf8);
            }
        };

        // GET只有请求头，POST还有请求体，当前只考虑GET

        // 以CRLF为边界分割字符串
        let request_lines: Vec<&str> = request_string.split(CRLF).collect();

        // 然后再以空格分割首行
        let first_line: Vec<&str> = request_lines[0].split(" ").collect();
        let method_str = first_line[0].to_uppercase();
        let method = match method_str.as_str() {
            "GET" => HttpRequestMethod::Get,
            "POST" => HttpRequestMethod::Post,
            _ => {
                println!("Unsupported method!");
                return Err(Exception::UnSupportedRequestMethod);
            }
        };
        let path = first_line[1].to_string();
        let version_str = first_line[2].to_uppercase();
        let version = match version_str.as_str() {
            // 当前只支持1.1
            r"HTTP/1.1" => HttpVersion::V1_1,
            _ => {
                println!("Unsupported HTTP version!");
                return Err(Exception::UnsupportedHttpVersion);
            }
        };

        // 确定剩余字段
        let mut user_agent = "".to_string();
        let mut accept_encoding = vec!();
        for line in &request_lines {
            // 确定user-agent，注意 HTTP请求头大小写不敏感
            if line.starts_with("user-agent") || line.starts_with("User-Agent") {
                user_agent = line.split(": ").collect::<Vec<&str>>()[1].to_string();
                break;
            }
        }

        for line in &request_lines {
            // 确定accept-encoding，即浏览器能接受的压缩编码
            if line.starts_with("accept-encoding") || line.starts_with("Accept-Encoding") {
                let encoding = line.split(": ").collect::<Vec<&str>>()[1];
                if encoding.contains("gzip") {
                    accept_encoding.push(HttpEncoding::Gzip);
                }
                if encoding.contains("deflate") {
                    accept_encoding.push(HttpEncoding::Deflate);
                }
                if encoding.contains("br") {
                    accept_encoding.push(HttpEncoding::Br);
                }
                break;
            }
        }

        Ok(Self {
            method,
            path,
            version,
            user_agent,
            accept_encoding,
        })
    }
}

impl Request {
    /// 返回当前Request的请求路径
    pub fn path(&self) -> &str {
        &self.path
    }

    /// 返回请求的方法
    pub fn method(&self) -> HttpRequestMethod {
        self.method
    }

    /// 返回当前Request的User-Agent
    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    /// 返回当前浏览器接受的压缩编码
    pub fn accept_encoding(&self) -> &Vec<HttpEncoding> {
        &self.accept_encoding
    }
}