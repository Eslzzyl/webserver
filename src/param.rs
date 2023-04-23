pub const HTML_INDEX: &str = r"files/html/index.html";
pub const HTML_404: &str = r"files/html/404.html";

pub const CRLF: &str = "\r\n";

#[derive(Debug, Clone, Copy)]
pub enum HttpVersion {
    V1_1,
}

#[derive(Debug, Clone, Copy)]
pub enum HttpEncoding {
    Gzip,
    Deflate,
    Br,
}