use std::collections::HashMap;

use lazy_static::lazy_static;

pub const HTML_INDEX: &str = r"files/html/index.html";
pub const HTML_404: &str = r"files/html/404.html";

pub const SERVER_NAME: &str = "eslzzyl-webserver";

pub const CRLF: &str = "\r\n";

lazy_static! {
    pub static ref STATUS_CODES: HashMap<u16, &'static str> = {
        let mut map = HashMap::new();
        map.insert(100, "Continue");
        map.insert(101, "Switching Protocols");
        // 2xx: Successful
        map.insert(200, "OK");
        map.insert(201, "Created");
        map.insert(202, "Accepted");
        map.insert(203, "Non-Authoritative Information");
        map.insert(204, "No Content");
        map.insert(205, "Reset Content");
        map.insert(206, "Partial Content");
        // 3xx: Redirection
        map.insert(300, "Multiple Choices");
        map.insert(301, "Moved Permanently");
        map.insert(302, "Found");
        map.insert(303, "See Other");
        map.insert(304, "Not Modified");
        map.insert(305, "Use Proxy");
        // 306 已弃用
        map.insert(307, "Temporary Redirect");
        map.insert(308, "Permanent Redirect");
        // 4xx: Client Error
        map.insert(400, "Bad Request");
        map.insert(401, "Unauthorized");
        map.insert(402, "Payment Required");  // 保留，当前不使用
        map.insert(403, "Forbidden");
        map.insert(404, "Not Found");
        map.insert(405, "Method Not Allowed");
        map.insert(406, "Not Acceptable");
        map.insert(407, "Proxy Authentication Required");
        map.insert(408, "Request Timeout");
        map.insert(409, "Conflict");
        map.insert(410, "Gone");
        map.insert(411, "Length Required");
        map.insert(412, "Precondition Failed");
        map.insert(413, "Content Too Large");
        map.insert(414, "URI Too Long");
        map.insert(415, "Unsupported Media Type");
        map.insert(416, "Range Not Satisfiable");
        map.insert(417, "Expectation Failed");
        map.insert(418, "I'm a teapot");      // 愚人节玩笑，见RFC2324，该状态码不应被使用
        map.insert(421, "Misdirected Request");
        map.insert(422, "Unprocessable Content");
        map.insert(426, "Upgrade Required");
        // 5xx: Server Error
        map.insert(500, "Internal Server Error");
        map.insert(501, "Not Implemented");
        map.insert(502, "Bad Gateway");
        map.insert(503, "Service Unavailable");
        map.insert(504, "Gateway Timeout");
        map.insert(505, "HTTP Version Not Supported");
        map
    };
}

#[derive(Debug, Clone, Copy)]
pub enum HttpVersion {
    V1_1,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HttpRequestMethod {
    Post,
    Get,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HttpEncoding {
    Gzip,
    Deflate,
    Br,
    None,
}

use std::fmt;

impl fmt::Display for HttpVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            HttpVersion::V1_1 => write!(f, "1.1"),
        }
    }
}

impl fmt::Display for HttpEncoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            HttpEncoding::Gzip => write!(f, "gzip"),
            HttpEncoding::Deflate => write!(f, "deflate"),
            HttpEncoding::Br => write!(f, "br"),
            HttpEncoding::None => write!(f, "None"),
        }
    }
}