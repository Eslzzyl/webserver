use crate::exception::Exception;
use crate::param::*;

#[derive(Debug, Clone, Copy)]
enum HttpRequestMethod {
    Post,
    Get,
}

#[derive(Debug, Clone, Copy)]
enum HttpRequestVersion {
    V1_1,
}

#[derive(Debug, Clone, Copy)]
enum HttpRequestEncoding {
    Gzip,
    Deflate,
    Br,
}

#[derive(Debug, Clone)]
pub struct Request {
    method: HttpRequestMethod,
    path: String,
    version: HttpRequestVersion,
    user_agent: String,
    accept_encoding: Vec<HttpRequestEncoding>,  // 压缩编码，可以支持多种编码，如果该vec为空说明不支持压缩
}

impl Request {
    pub fn new() -> Self {
        Self {
            method: HttpRequestMethod::Get,
            path: "".into(),
            version: HttpRequestVersion::V1_1,
            user_agent: "".into(),
            accept_encoding: Vec::<HttpRequestEncoding>::new(),
        }
    }

    pub fn try_from(buffer: Vec<u8>) -> Result<Self, Exception> {
        let request_string = match String::from_utf8(buffer) {
            Ok(string) => string,
            Err(_) => {
                println!("Error when parsing request!");
                return Err(Exception::RequestIsNotUtf8);
            }
        };

        dbg!(&request_string);

        // TODO 解析请求，然后用解析的结果替代返回值

        // GET只有请求头，POST还有请求体，当前只考虑GET

        // 以CRLF为边界分割字符串
        let request_lines: Vec<&str> = request_string.split(CRLF).collect();

        // 然后再以空格分割首行
        let first_line: Vec<&str> = request_lines[0].split(" ").collect();
        let method = match first_line[0] {
            "GET" => HttpRequestMethod::Get,
            "POST" => HttpRequestMethod::Post,
            _ => {
                println!("Unsupported method!");
                return Err(Exception::UnSupportedRequestMethod);
            }
        };
        dbg!(method);
        // 也许这里应该改写成URI？那是后话
        let path = first_line[1].to_string();
        let version = match first_line[2] {
            // 当前只支持1.1
            r"HTTP/1.1" => HttpRequestVersion::V1_1,
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
                    accept_encoding.push(HttpRequestEncoding::Gzip);
                }
                if encoding.contains("deflate") {
                    accept_encoding.push(HttpRequestEncoding::Deflate);
                }
                if encoding.contains("br") {
                    accept_encoding.push(HttpRequestEncoding::Br);
                }
                break;
            }
        }

        Ok(Self {
            method: method,
            path: path,
            version: version,
            user_agent: user_agent,
            accept_encoding: accept_encoding,
        })
    }
}