use std::collections::HashMap;

use lazy_static::lazy_static;

pub const HTML_INDEX: &str = r"files/html/index.html";

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

lazy_static! {
    pub static ref MIME_TYPES: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("aac", "audio/aac");
        map.insert("abw", "application/x-abiword");
        map.insert("apk", "application/vnd.android.package-archive");
        map.insert("arc", "application/x-freearc");
        map.insert("avi", "video/x-msvideo");
        map.insert("avif", "image/avif");
        map.insert("azw", "application/vnd.amazon.ebook");
        map.insert("bin", "application/octet-stream");
        map.insert("bmp", "image/bmp");
        map.insert("bz", "application/x-bzip");
        map.insert("bz2", "application/x-bzip2");
        map.insert("cab", "application/vnd.ms-cab-compressed");
        map.insert("cda", "application/x-cdf");
        map.insert("csh", "application/x-csh");
        map.insert("css", "text/css;charset=utf-8");
        map.insert("csv", "text/csv");
        map.insert("crx", "application/x-chrome-extension");
        map.insert("deb", "application/x-deb");
        map.insert("doc", "application/msword");
        map.insert("docx", "application/vnd.openxmlformats-officedocument.wordprocessingml.document");
        map.insert("eot", "application/vnd.ms-fontobject");
        map.insert("epub", "application/epub+zip");
        map.insert("exe", "application/x-msdownload");
        map.insert("gif", "image/gif");
        map.insert("gz", "application/gzip");
        map.insert("htm", "text/html;charset=utf-8");
        map.insert("html", "text/html;charset=utf-8");
        map.insert("img", "application/x-iso9660-image");
        map.insert("ico", "image/x-icon");
        map.insert("ics", "text/calendar");
        map.insert("iso", "application/x-iso9660-image");
        map.insert("jar", "application/java-archive");
        map.insert("js", "text/javascript;charset=utf-8");
        map.insert("json", "application/json");
        map.insert("jsonld", "application/ld+json");
        map.insert("jpg", "image/jpeg");
        map.insert("jpeg", "image/jpeg");
        map.insert("mid", "audio/x-midi");
        map.insert("midi", "audio/x-midi");
        map.insert("mjs", "text/javascript");
        map.insert("mkv", "video/x-matroska");
        map.insert("mp3", "audio/mpeg");
        map.insert("mp4", "video/mp4");
        map.insert("mpeg", "video/mpeg");
        map.insert("mpkg", "application/vnd.apple.installer+xml");
        map.insert("msi", "application/x-msdownload");
        map.insert("odp", "application/vnd.oasis.opendocument.presentation");
        map.insert("ods", "application/vnd.oasis.opendocument.spreadsheet");
        map.insert("odt", "application/vnd.oasis.opendocument.text");
        map.insert("oga", "audio/ogg");
        map.insert("ogv", "video/ogg");
        map.insert("ogx", "application/ogg");
        map.insert("opus", "audio/opus");
        map.insert("otf", "font/otf");
        map.insert("pdf", "application/pdf");
        map.insert("png", "image/png");
        map.insert("php", "application/x-httpd-php");
        map.insert("ppt", "application/vnd.ms-powerpoint");
        map.insert("pptx", "application/vnd.openxmlformats-officedocument.presentationml.presentation");
        map.insert("rar", "application/x-rar-compressed");
        map.insert("rtf", "application/rtf");
        map.insert("rpm", "application/x-rpm");
        map.insert("sh", "application/x-sh");
        map.insert("svg", "image/svg+xml");
        map.insert("swf", "application/x-shockwave-flash");
        map.insert("tar", "application/x-tar");
        map.insert("tif", "image/tiff");
        map.insert("tiff", "image/tiff");
        map.insert("ts", "video/mp2t");
        map.insert("txt", "text/plain");
        map.insert("ttf", "font/ttf");
        map.insert("vsd", "application/vnd.visio");
        map.insert("wav", "audio/wav");
        map.insert("wasm", "application/wasm");
        map.insert("weba", "audio/webm");
        map.insert("webm", "video/webm");
        map.insert("webp", "image/webp");
        map.insert("woff", "font/woff");
        map.insert("woff2", "font/woff2");
        map.insert("xhtml", "application/xhtml+xml");
        map.insert("xls", "application/vnd.ms-excel");
        map.insert("xlsx", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
        map.insert("xml", "text/xml");
        map.insert("xpi", "application/x-xpinstall");
        map.insert("xul", "application/vnd.mozilla.xul+xml");
        map.insert("zip", "application/zip");
        map.insert("7z", "application/x-7z-compressed");
        map.insert("_", "application/octet-stream");
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