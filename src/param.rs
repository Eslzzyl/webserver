pub const HTML_INDEX: &str = r"files/html/index.html";
pub const HTML_404: &str = r"files/html/404.html";

pub const SERVER_NAME: &str = "eslzzyl-webserver";

pub const CRLF: &str = "\r\n";

#[derive(Debug, Clone, Copy)]
pub enum HttpVersion {
    V1_1,
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