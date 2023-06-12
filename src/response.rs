use crate::{
    param::*,
    request::Request,
    cache::FileCache,
    util::HtmlBuilder,
};

use chrono::prelude::*;
use bytes::Bytes;
use flate2::{
    write::{DeflateEncoder, GzEncoder},
    Compression,
};
use brotli::enc::{self, backward_references::BrotliEncoderParams};
use log::{error, info};

use std::{
    io::{self, Read, Write},
    sync::{Arc, Mutex},
    fs::File,
};

/// HTTP 响应
/// 
/// - `version`: 使用的HTTP版本。目前仅支持1.1
/// - `status_code`: HTTP状态码
/// - `information`: 对状态码的说明文字
/// - `content_type`: MIME
/// - `content_length`: 响应**体**的长度。不包含响应头。
/// - `date`: 发送响应时的时间
/// - `content_encoding`: 指定响应体应当以何种算法进行压缩
/// - `server_name`: 服务器名
/// - `content`: 响应体本身
#[derive(Debug, Clone)]
pub struct Response {
    version: HttpVersion,
    status_code: u16,
    information: String,
    content_type: String,
    content_length: usize,
    date: DateTime<Utc>,
    content_encoding: HttpEncoding,     // 响应仅指定一种压缩编码即可。若浏览器支持多种，则具体采用哪种由Config决定
    server_name: String,
    content: Bytes,
}

impl Response {
    /// 生成一个空的Response对象，各成员默认值为：
    /// 
    /// - HTTP版本：`1.1`
    /// - 状态码：200
    /// - 响应信息：`OK`
    /// - Content-Type：`text/plain;charset=utf-8`
    /// - Content-Length：`0`
    /// - Date：当前的UTC时间
    /// - Content-Encoding：明文（无压缩）
    /// - Content：留空
    pub fn new() -> Self {
        Self {
            version: HttpVersion::V1_1,
            status_code: 200,
            information: "OK".to_string(),
            content_type: "text/plain;charset=utf-8".to_string(),
            content_length: 0,
            date: Utc::now(),
            content_encoding: HttpEncoding::None,
            server_name: SERVER_NAME.to_string(),
            content: Bytes::new(),
        }
    }

    /// 通过指定的文件构建content域，文件内容是以无压缩字节流的形式写入的
    /// 
    /// ## 参数：
    /// - path: 文件的完整路径
    fn from_file(path: &str, accept_encoding: Vec<HttpEncoding>, id: u128, cache: &Arc<Mutex<FileCache>>) -> Self {
        let mut response = Self::new();

        let content_encoding = decide_encoding(&accept_encoding);
        match content_encoding {
            HttpEncoding::Gzip => info!("[ID{}]使用Gzip压缩编码", id),
            HttpEncoding::Br => info!("[ID{}]使用Brotli压缩编码", id),
            HttpEncoding::Deflate => info!("[ID{}]使用Deflate压缩编码", id),
            HttpEncoding::None => info!("[ID{}]不进行压缩", id),
        };
        
        // 查找缓存
        let mut cache_lock = cache.lock().unwrap();
        match cache_lock.find(path) {
            Some(bytes) => {
                info!("[ID{}]缓存命中", id);
                response.content = bytes.clone();
                // 这里其实是有个潜在问题的。理论上不同客户端要求的encoding可能会不同，但是缓存却是共享的，导致encoding是相同的。
                // 但是单客户端情况下可以忽略。而且目前所有主流浏览器也都支持gzip了。
                response.content_encoding = content_encoding;
            },
            None => {
                info!("[ID{}]缓存未命中", id);
                let mut file = match File::open(path) {
                    Ok(f) => f,
                    Err(e) => {
                        error!("[ID{}]无法打开路径{}指定的文件。错误：{}", id, path, e);
                        panic!();
                    },
                };
                let mut contents = Vec::new();
                match file.read_to_end(&mut contents) {
                    Ok(_) => {},
                    Err(e) => {
                        error!("[ID{}]无法读取文件{}。错误：{}", id, path, e);
                        panic!();
                    }
                }
                response.content_encoding = content_encoding;
                contents = compress(contents, response.content_encoding).unwrap();
                response.content = Bytes::from(contents.clone()); 
                cache_lock.push(path, Bytes::from(contents));
            }
        }
        response.content_length = response.content.len();
        response
    }

    fn from_status_code(code: u16, accept_encoding: Vec<HttpEncoding>) -> Self {
        let mut response = Self::new();
        response.content_encoding = decide_encoding(&accept_encoding);
        let content = match code {
            404 => HtmlBuilder::from_status_code(405, Some(
                r"<h1>噢！</h1><p>你指定的网页无法找到。</p>"
            )),
            405 => HtmlBuilder::from_status_code(405, Some(
                r"<h1>噢！</h1><p>你的浏览器发出了一个非GET方法的HTTP请求。本服务器目前仅支持GET方法。</p>"
            )),
            _ => HtmlBuilder::from_status_code(405, None),
        }.build();
        response.content = Bytes::from(content);
        response.content_length = response.content.len();
        response.status_code = code;
        response
    }

    /// 设定时间为当前时刻
    fn set_date(&mut self) -> &mut Self {
        self.date = Utc::now();
        self
    }

    /// 设置content_type即mime
    fn set_content_type(&mut self, mime: &str) -> &mut Self {
        self.content_type = mime.to_string();
        self
    }

    /// 设置响应协议版本，当前固定为HTTP1.1
    fn set_version(&mut self) -> &mut Self {
        self.version = HttpVersion::V1_1;
        self
    }

    /// 设置服务器名
    fn set_server_name(&mut self) -> &mut Self {
        self.server_name = SERVER_NAME.to_string();
        self
    }

    /// 预设的404 Response
    pub fn response_404(request: &Request, id: u128, cache: &Arc<Mutex<FileCache>>) -> Vec<u8> {
        let accept_encoding = request.accept_encoding().to_vec();
        Self::from_file(HTML_404, accept_encoding, id, cache)
            .set_content_type("text/html;charset=utf-8")
            .set_date()
            .set_code(404)
            .set_version()
            .as_bytes()
    }

    pub fn from(path: &str, mime: &str, request: &Request, id: u128, cache: &Arc<Mutex<FileCache>>) -> Vec<u8> {
        let accept_encoding = request.accept_encoding().to_vec();
        let method = request.method();
        // 当前仅支持GET方法，其他方法一律返回405
        if method != HttpRequestMethod::Get {
            Self::from_status_code(405, accept_encoding)
            .set_content_type("text/html;charset=utf-8")
            .set_date()
            .set_version()
            .set_server_name()
            .as_bytes()
        } else {
            Self::from_file(path, accept_encoding, id, cache)
            .set_content_type(mime)
            .set_date()
            .set_code(200)
            .set_version()
            .set_server_name()
            .as_bytes()
        }
    }


    pub fn as_bytes(&self) -> Vec<u8> {
        // 获取各字段的&str
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
            HttpEncoding::Br => "br",
            HttpEncoding::None => "",   // 实际上这一条是用不到的，后面还会特别检测是不是None。如果是，就直接略去content-encoding字段了。
        }.to_string();

        // 拼接响应头
        let binding = [
            version, " ", status_code, " ", information, CRLF,
            "Content-Type: ", content_type, CRLF,
            "Content-Length: ", content_length, CRLF,
            "Date: ", date, CRLF,
        ].concat();
        // 不要把下面这行的as_str移动到上面那行，否则会有生命周期问题
        let binding = binding.as_str();
        // 如果 self.content_encoding 是None，就直接跳过这个字段，否则才进行编码
        let binding = if self.content_encoding != HttpEncoding::None {
            [
                // 注意：首部总是以一个空行（仅包含一个CRLF）结束，即使没有主体部分也是如此。
                binding,
                "Content-Encoding: ", content_encoding, CRLF,
                CRLF,
            ].concat()
        } else {
            [
                binding,
                CRLF,
            ].concat()
        };
        // 拼接响应体
        [binding.as_bytes(), &self.content].concat()
    }
}


impl Response {
    /// 本函数根据传入的`code`参数设置`self`对象的状态码字段和HTTP信息字段。状态码和信息是一一对应的。
    /// 
    /// 现行HTTP协议的状态码由[RFC9110#15](https://www.rfc-editor.org/rfc/rfc9110#section-15)规定。
    /// 
    /// ## 参数：
    /// - `code`: 状态码。实际上HTTP状态码最大的也就是500多，因此采用`u16`
    fn set_code(&mut self, code: u16) -> &mut Self {
        self.status_code = code;
        self.information = match STATUS_CODES.get(&code) {
            Some(&info) => info.to_string(),
            None => {
                error!("非法的状态码：{}。这条错误说明代码编写出现了错误。", code);
                panic!();
            }
        };
        self
    }
}

/// 格式化时间，使用`chrono` crate自带的`to_rfc2822`方法
fn format_date(date: &DateTime<Utc>) -> String {
    date.to_rfc2822()
}

/// 压缩响应体
/// 
/// ## 参数：
/// - `data`：响应体数据，以字节流形式给出
/// - `mode`：指定的压缩格式，见[HttpEncoding]
/// 
/// ## 返回：
/// - 压缩后的响应体数据，以字节流形式给出
fn compress(data: Vec<u8>, mode: HttpEncoding) -> io::Result<Vec<u8>> {
    match mode {
        HttpEncoding::Gzip => {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&data)?;
            encoder.finish()
        },
        HttpEncoding::Deflate => {
            let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&data)?;
            encoder.finish()
        },
        HttpEncoding::Br => {
            let params = BrotliEncoderParams::default();
            let mut output = Vec::new();
            enc::BrotliCompress(&mut io::Cursor::new(data), &mut output, &params)?;
            Ok(output)
        },
        HttpEncoding::None => {
            // 无压缩方式，直接返回原文
            Ok(data)
        }
    }
}

/// 确定响应体压缩编码的逻辑：
/// 1. 如果浏览器支持Brotli，则使用Brotli。
/// 2. 否则，如果浏览器支持Gzip，则使用Gzip。
/// 3. 否则，如果浏览器支持Deflate，则使用Deflate。
/// 4. 再否则，就只好不压缩了。
/// 
/// 实测Brotli太慢，因此优先用Gzip。考虑后期换一个brotli库。
fn decide_encoding(accept_encoding: &Vec<HttpEncoding>) -> HttpEncoding {
    if accept_encoding.contains(&HttpEncoding::Gzip) {
        HttpEncoding::Gzip
    } else if accept_encoding.contains(&HttpEncoding::Deflate) {
        HttpEncoding::Deflate
    } else {
        HttpEncoding::None
    }
}