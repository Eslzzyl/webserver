use crate::param::*;

use chrono::prelude::*;
use bytes::Bytes;

use std::fs::File;

#[derive(Debug, Clone)]
pub struct Response {
    version: HttpVersion,
    status_code: u16,
    information: String,
    content_type: String,
    content_length: usize,
    date: DateTime<Utc>,
    content_encoding: HttpEncoding,     // 响应仅指定一种压缩编码即可。若浏览器支持多种，则具体采用哪种由Config决定
    content: Bytes,
}

impl Response {
    /// 生成一个空的Response对象，状态码为200 OK
    pub fn new() -> Self {
        Self {
            version: HttpVersion::V1_1,
            status_code: 200,
            information: "OK".to_string(),
            content_type: "text/plain;charset=utf-8".to_string(),
            content_length: 0,
            date: Utc::now(),
            content_encoding: HttpEncoding::Gzip,
            content: Bytes::new(),
        }
    }

    /// 通过指定的文件构建content域，文件内容是以无压缩字节流的形式写入的
    /// 
    /// 参数：
    /// 
    /// - path: 文件的完整路径
    fn from_file(path: String) -> Self {
        let file = File::open(path).unwrap();

    }

    /// 设定时间为当前时刻
    fn set_date(&mut self) -> &mut Self {
        self.date = Utc::now();
        self
    }

    /// 设置content_type即mime
    fn set_content_type(&mut self, mime: String) -> &mut Self {
        self.content_type = mime;
        self
    }

    /// 设置响应协议版本，当前固定为HTTP1.1
    fn set_version(&mut self) -> &mut Self {
        self.version = HttpVersion::V1_1;
        self
    }

    fn compress(&mut self) -> &mut Self {
        todo!();
    }

    pub fn response_404() -> Self {
        let mime = "text/html;charset=utf-8".to_string();
        let response
            = Self::from_file(HTML_404.to_string())
                .set_content_type(mime)
                .set_date()
                .set_code(404);

        return response;
    }

    pub fn from() -> Self {
        todo!()
    }

    // 注意：首部总是以一个空行（仅包含一个CRLF）结束，即使没有主体部分也是如此。
    pub fn as_bytes(&self) -> &[u8] {
        let version: &str = match self.version {
            HttpVersion::V1_1 => "HTTP/1.1",
        };
        let status_code: &str = &self.status_code.to_string();
        let information: &str = &self.information;
        let content_type: &str = &self.content_type;
        let content_length: &str = &self.content_length.to_string();
        let date: &str = &format_date(&self.date);
        let content_encoding: &str = &match self.content_encoding {
            HttpEncoding::Gzip => "gzip",
            HttpEncoding::Deflate => "deflate",
            HttpEncoding::Br => "br"
        }.to_string();
        let b = [
            version, " ", status_code, " ", information, CRLF,
            "Content-Type: ", content_type, CRLF,
            "Content-Length: ", content_length, CRLF,
            "Date: ", date, CRLF,
            "Content-Encoding: ", content_encoding, CRLF,
            CRLF,
        ].concat().as_bytes();
        &[b, &self.content].concat()
    }
}

fn format_date(date: &DateTime<Utc>) -> String {
    date.to_rfc2822()
}

impl Response {
    /// 设置状态码(Status Code)和状态短语
    /// 
    /// 本函数提供了RFC9110中定义的所有状态码，尽管大部分可能不会用到。见 [RFC9110#15](https://www.rfc-editor.org/rfc/rfc9110#section-15)
    fn set_code(&mut self, code: u16) -> &mut Self {
        self.status_code = code;
        self.information = match code {
            // 1xx: Informational
            100 => "Continue",
            101 => "Switching Protocols",
            // 2xx: Successful
            200 => "OK",
            201 => "Created",
            202 => "Accepted",
            203 => "Non-Authoritative Information",
            204 => "No Content",
            205 => "Reset Content",
            206 => "Partial Content",
            // 3xx: Redirection
            300 => "Multiple Choices",
            301 => "Moved Permanently",
            302 => "Found",
            303 => "See Other",
            304 => "Not Modified",
            305 => "Use Proxy",
            // 306 已弃用
            307 => "Temporary Redirect",
            308 => "Permanent Redirect",
            // 4xx: Client Error
            400 => "Bad Request",
            401 => "Unauthorized",
            402 => "Payment Required",  // 保留，当前不使用
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            406 => "Not Acceptable",
            407 => "Proxy Authentication Required",
            408 => "Request Timeout",
            409 => "Conflict",
            410 => "Gone",
            411 => "Length Required",
            412 => "Precondition Failed",
            413 => "Content Too Large",
            414 => "URI Too Long",
            415 => "Unsupported Media Type",
            416 => "Range Not Satisfiable",
            417 => "Expectation Failed",
            408 => "I'm a teapot",      // 愚人节玩笑，见RFC2324，该状态码不应被使用
            421 => "Misdirected Request",
            422 => "Unprocessable Content",
            426 => "Upgrade Required",
            // 5xx: Server Error
            500 => "Internal Server Error",
            501 => "Not Implemented",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            504 => "Gateway Timeout",
            505 => "HTTP Version Not Supported",
            _ => panic!("Invalid status code: {}", code),
        }.to_string();
        self
    }
}