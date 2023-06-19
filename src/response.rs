use crate::{
    param::*,
    request::Request,
    cache::FileCache,
    util::{HtmlBuilder, handle_php},
};

use chrono::prelude::*;
use bytes::Bytes;
use flate2::{
    write::{DeflateEncoder, GzEncoder},
    Compression,
};
use brotli::enc::{self, backward_references::BrotliEncoderParams};
use log::{error, warn, debug};

use std::{
    io::{self, Read, Write},
    sync::{Arc, Mutex},
    fs::{self, File, metadata},
    ffi::OsStr,
    path::{Path, PathBuf},
    os::unix::prelude::MetadataExt,
    str,
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
/// - `allow`: 服务器允许的HTTP请求方法
/// - `content`: 响应体本身
#[derive(Debug, Clone)]
pub struct Response {
    version: HttpVersion,
    status_code: u16,
    information: String,
    content_type: Option<String>,
    content_length: u64,
    date: DateTime<Utc>,
    content_encoding: Option<HttpEncoding>,
    server_name: String,
    allow: Option<Vec<HttpRequestMethod>>,
    content: Option<Bytes>,
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
        let allow = vec![
            HttpRequestMethod::Get,
            HttpRequestMethod::Head,
            HttpRequestMethod::Options,
        ];
        Self {
            version: HttpVersion::V1_1,
            status_code: 200,
            information: "OK".to_string(),
            content_type: None,
            content_length: 0,
            date: Utc::now(),
            content_encoding: None,
            server_name: SERVER_NAME.to_string(),
            allow: Some(allow),
            content: None,
        }
    }

    /// 通过指定的文件构建content域，文件内容是以无压缩字节流的形式写入的
    /// 
    /// ## 参数
    /// - `path`: 文件的完整路径
    /// - `accept_encoding`: 浏览器能够接受的压缩编码，需要根据该参数确定压缩编码
    /// - `id`: 用于日志的TCP连接编号
    /// - `cache`: 共享的文件缓存指针
    /// 
    /// ## 返回
    /// - 一个新的 Response 对象，不完整，还需要进一步处理才能发回浏览器
    fn from_file(path: &str, accept_encoding: Vec<HttpEncoding>, id: u128, cache: &Arc<Mutex<FileCache>>, headonly: bool, mime: &str) -> Self {
        let mut response = Self::new();
        response.content_encoding = match headonly {
            true => None,
            false => decide_encoding(&accept_encoding),
        };
        match response.content_encoding {
            Some(HttpEncoding::Gzip) => debug!("[ID{}]使用Gzip压缩编码", id),
            Some(HttpEncoding::Br) => debug!("[ID{}]使用Brotli压缩编码", id),
            Some(HttpEncoding::Deflate) => debug!("[ID{}]使用Deflate压缩编码", id),
            None => debug!("[ID{}]不进行压缩", id),
        };
        
        // 查找缓存
        let mut cache_lock = cache.lock().unwrap();
        match cache_lock.find(path) {
            Some(bytes) => {
                debug!("[ID{}]缓存命中", id);
                // 这里其实是有个潜在问题的。理论上不同客户端要求的encoding可能会不同，但是缓存却是共享的，导致encoding是相同的。
                // 但是单客户端情况下可以忽略。而且目前所有主流浏览器也都支持gzip了。
                response.content_length = bytes.len() as u64;
                response.content = match headonly {
                    // headonly时，不填入content（但正常设置length），否则填入找到的bytes
                    true => None,
                    false => Some(bytes.clone()),
                };
            },
            None => {
                debug!("[ID{}]缓存未命中", id);
                if headonly {
                    // 如果headonly为true，则通过Metadata直接获取文件大小，而不是真的读取文件，从而提高性能
                    let path = Path::new(path);
                    let metadata = metadata(path).unwrap();
                    response.content_type = None;
                    response.content = None;
                    response.content_length = metadata.size();
                } else {
                    // 如果为false，又缓存不命中，就只好读取文件
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
                    contents = compress(contents, response.content_encoding).unwrap();
                    response.content_length = contents.len() as u64;
                    response.content_type = Some(mime.to_string());
                    response.content = Some(Bytes::from(contents.clone()));
                    cache_lock.push(path, Bytes::from(contents));
                }
            }
        }
        response
    }

    /// 通过状态码创建response对象。content部分由`HtmlBuilder`生成。
    /// 
    /// ## 参数
    /// - `code`: 状态码
    /// - `accept_encoding`: 浏览器能够接受的压缩编码，需要根据该参数确定压缩编码
    /// - `id`: 用于日志的TCP连接编号
    /// 
    /// ## 返回
    /// - 一个新的 Response 对象，不完整，还需要进一步处理才能发回浏览器
    fn from_status_code(code: u16, accept_encoding: Vec<HttpEncoding>, id: u128) -> Self {
        let mut response = Self::new();
        response.content_encoding = decide_encoding(&accept_encoding);
        // 204响应不包含响应体，因此encoding和type也不需要
        if code == 204 {
            response.content = None;
            response.content_encoding = None;
            response.content_type = None;
            return response;
        }
        match response.content_encoding {
            Some(HttpEncoding::Gzip) => debug!("[ID{}]使用Gzip压缩编码", id),
            Some(HttpEncoding::Br) => debug!("[ID{}]使用Brotli压缩编码", id),
            Some(HttpEncoding::Deflate) => debug!("[ID{}]使用Deflate压缩编码", id),
            None => debug!("[ID{}]不进行压缩", id),
        };
        let content = match code {
            404 => HtmlBuilder::from_status_code(404, Some(
                r"<h2>噢！</h2><p>你指定的网页无法找到。</p>"
            )),
            405 => HtmlBuilder::from_status_code(405, Some(
                r"<h2>噢！</h2><p>你的浏览器发出了一个非GET方法的HTTP请求。本服务器目前仅支持GET方法。</p>"
            )),
            500 => HtmlBuilder::from_status_code(500, Some(
                r"<h2>噢！</h2><p>服务器出现了一个内部错误。</p>"
            )),
            _ => HtmlBuilder::from_status_code(code, None),
        }.build();
        let content_compressed = compress(content.into_bytes(), response.content_encoding).unwrap();
        let bytes = Bytes::from(content_compressed);
        response.content_length = bytes.len() as u64;
        response.content = Some(bytes);
        response.content_type = Some("text/html;charset=utf-8".to_string());
        response.status_code = code;
        response
    }

    /// 通过目录来生成一个 `Response`，该 `Response` 应当列出目录的所有文件。
    /// 
    /// ## 参数
    /// - `path`: 文件的完整路径
    /// - `accept_encoding`: 浏览器能够接受的压缩编码，需要根据该参数确定压缩编码
    /// - `id`: 用于日志的TCP连接编号
    /// - `cache`: 共享的文件缓存指针
    /// 
    /// ## 返回
    /// - 一个新的 Response 对象，不完整，还需要进一步处理才能发回浏览器
    fn from_dir(path: &str, accept_encoding: Vec<HttpEncoding>, id: u128, cache: &Arc<Mutex<FileCache>>, headonly: bool) -> Self {
        let mut response = Self::new();
        response.content_encoding = match headonly {
            true => None,
            false => decide_encoding(&accept_encoding),
        };
        match response.content_encoding {
            Some(HttpEncoding::Gzip) => debug!("[ID{}]使用Gzip压缩编码", id),
            Some(HttpEncoding::Br) => debug!("[ID{}]使用Brotli压缩编码", id),
            Some(HttpEncoding::Deflate) => debug!("[ID{}]使用Deflate压缩编码", id),
            None => debug!("[ID{}]不进行压缩", id),
        };

        // 仅在有响应体时才设置content-type
        if !headonly {
            response.content_type = Some("text/html;charset=utf-8".to_string());
        }

        // 查找缓存
        let mut cache_lock = cache.lock().unwrap();
        match cache_lock.find(path) {
            Some(bytes) => {
                debug!("[ID{}]缓存命中", id);
                // 这里其实是有个潜在问题的。理论上不同客户端要求的encoding可能会不同，但是缓存却是共享的，导致encoding是相同的。
                // 但是单客户端情况下可以忽略。而且目前所有主流浏览器也都支持gzip了。
                response.content = match headonly {
                    // headonly时，填入一个空字符串，否则填入找到的bytes
                    true => None,
                    false => Some(bytes.clone()),
                };
                response.content_length = bytes.len() as u64;
            },
            None => {   // 缓存未命中，生成目录列表
                debug!("[ID{}]缓存未命中", id);
                let mut dir_vec = Vec::<PathBuf>::new();
                let entries = fs::read_dir(path).unwrap();
                for entry in entries.into_iter() {
                    dir_vec.push(entry.unwrap().path());
                }
                let content = HtmlBuilder::from_dir(path, &mut dir_vec).build();
                let content_compressed = compress(content.into_bytes(), response.content_encoding).unwrap();
                response.content_length = content_compressed.len() as u64;
                // headonly时，填入一个空字符串，否则填入压缩好的content
                response.content = match headonly {
                    true => None,
                    false => Some(Bytes::from(content_compressed)),
                };
                cache_lock.push(path, response.content.clone().unwrap());
            }
        }
        response
    }

    /// 通过HTML代码生成一个`Response`
    /// 
    /// ## 参数
    /// - `html`: HTML代码
    /// - `accept_encoding`: 浏览器能够接受的压缩编码，需要根据该参数确定压缩编码
    /// - `id`: 用于日志的TCP连接编号
    /// 
    /// ## 返回
    /// - 一个新的 Response 对象，不完整，还需要进一步处理才能发回浏览器
    /// 
    /// 本函数不涉及对文件缓存的访问，因为本函数被设计用来进行PHP的处理，而PHP往往是动态页面。
    fn from_html(html: &str, accept_encoding: Vec<HttpEncoding>, id: u128, headonly: bool) -> Response {
        let mut response = Self::new();
        if headonly {
            response.content_encoding = None;
            response.content_type = None;
            response.content = None;
            return response;
        }
        response.content_encoding = decide_encoding(&accept_encoding);
        match response.content_encoding {
            Some(HttpEncoding::Gzip) => debug!("[ID{}]使用Gzip压缩编码", id),
            Some(HttpEncoding::Br) => debug!("[ID{}]使用Brotli压缩编码", id),
            Some(HttpEncoding::Deflate) => debug!("[ID{}]使用Deflate压缩编码", id),
            None => debug!("[ID{}]不进行压缩", id),
        };
        let content_compressed = compress(Vec::from(html), response.content_encoding).unwrap();
        response.content_length = content_compressed.len() as u64;
        response.content_type = Some("text/html;charset=utf-8".to_string());
        response.content = Some(Bytes::from(content_compressed));
        response
    }

    /// 设定时间为当前时刻
    fn set_date(&mut self) -> &mut Self {
        self.date = Utc::now();
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

    /// 本函数根据传入的`code`参数设置`self`对象的状态码字段和HTTP信息字段。状态码和信息是一一对应的。
    /// 
    /// 现行HTTP协议的状态码由[RFC9110#15](https://www.rfc-editor.org/rfc/rfc9110#section-15)规定。
    /// 
    /// ## 参数：
    /// - `code`: 状态码。实际上HTTP状态码最大的也就是500多，因此采用`u16`
    fn set_code(&mut self, code: u16) -> &mut Self {
        self.status_code = code;
        self.information = match STATUS_CODES.get(&code) {
            Some(&debug) => debug.to_string(),
            None => {
                error!("非法的状态码：{}。这条错误说明代码编写出现了错误。", code);
                panic!();
            }
        };
        self
    }

    /// 预设的404 Response
    pub fn response_404(request: &Request, id: u128) -> Self {
        let accept_encoding = request.accept_encoding().to_vec();
        Self::from_status_code(404, accept_encoding, id)
            .set_date()
            .set_code(404)
            .set_version()
            .to_owned()
    }

    /// 预设的500 Response
    pub fn response_500(request: &Request, id: u128) -> Self {
        let accept_encoding = request.accept_encoding().to_vec();
        Self::from_status_code(500, accept_encoding, id)
            .set_date()
            .set_code(500)
            .set_version()
            .to_owned()
    }

    /// 通过指定的路径创建一个`response`对象
    /// 
    /// ## 参数
    /// - `path`: 文件的完整路径
    /// - `request`: 来自浏览器的`request`
    /// - `id`: 用于日志的TCP连接编号
    /// - `cache`: 共享的文件缓存指针
    /// 
    /// ## 返回
    /// - HTTP响应
    pub fn from(path: &str, request: &Request, id: u128, cache: &Arc<Mutex<FileCache>>) -> Response {
        let accept_encoding = request.accept_encoding().to_vec();
        let method = request.method();
        let metadata_result = fs::metadata(path);

        // 仅有下列方法得到支持，其他方法一律返回405
        if method != HttpRequestMethod::Get
            && method != HttpRequestMethod::Head
            && method != HttpRequestMethod::Options {
            return Self::from_status_code(405, accept_encoding, id)
                .set_date()
                .set_version()
                .set_server_name()
                .to_owned();
        }

        // 对于OPTIONS方法的处理
        // OPTIONS允许指定明确的请求路径，或者请求*。服务器目前对所有的请求资源均使用相同的请求方法，因此无需特别处理路径问题。
        if method == HttpRequestMethod::Options {
            return Self::from_status_code(204, accept_encoding, id)
                .set_date()
                .set_version()
                .set_server_name()
                .to_owned();
        }

        // 若请求方法为HEAD，则设置headonly为true
        let headonly = match method {
            HttpRequestMethod::Head => true,
            _ => false,
        };

        match metadata_result {
            Ok(metadata) => {
                if metadata.is_dir() {  // path是目录
                    Self::from_dir(path, accept_encoding, id, cache, headonly)
                        .set_date()
                        .set_code(200)
                        .set_version()
                        .set_server_name()
                        .to_owned()
                } else {    // path是文件
                    let extention = match Path::new(path).extension() {
                        Some(e) => e,
                        None => {
                            error!("[ID{}]无法确定请求路径{}的文件扩展名", id, path);
                            return Self::response_404(request, id);
                        }
                    };
                    // 特殊情况：文件扩展名是PHP
                    if extention == "php" {
                        let html = match handle_php(path, id) {
                            Ok(html) => html,
                            Err(e) => {
                                error!("[ID{}]解析PHP文件{}时出错：{}", id, path, e);
                                return Self::response_500(request, id);
                            }
                        };
                        return Self::from_html(&html, accept_encoding, id, headonly)
                            .set_date()
                            .set_code(200)
                            .set_version()
                            .set_server_name()
                            .to_owned();
                    }
                    let mime = get_mime(extention);
                    Self::from_file(path, accept_encoding, id, cache, headonly, mime)
                        .set_date()
                        .set_code(200)
                        .set_version()
                        .set_server_name()
                        .clone()
                }
            }
            Err(_) => {
                warn!("[ID{}]无法获取{}的元数据，产生500 response", id, path);
                Self::response_500(request, id)
            }
        }
    }

    /// 将一个 `Response` 对象转换为字节流
    pub fn as_bytes(&self) -> Vec<u8> {
        // 如果content字段是None，那么content-type和content-encoding也必须是None
        if self.content == None {
            assert_eq!(self.content_encoding, None);
            assert_eq!(self.content_type, None);
        }
        // 获取各字段的&str
        let version: &str = match self.version {
            HttpVersion::V1_1 => "HTTP/1.1",
        };
        let status_code: &str = &self.status_code.to_string();
        let information: &str = &self.information;
        let content_length: &str = &self.content_length.to_string();
        let date: &str = &format_date(&self.date);
        let server: &str = &self.server_name;

        // 拼接响应
        [
            version, " ", status_code, " ", information, CRLF,
            // 选择性地填入content_type
            match &self.content_type {
                Some(t) => {
                    ["Content-Type: ", &t, CRLF].concat()
                },
                None => "".to_string(),
            }.as_str(),
            // 选择性地填入content_encoding
            match self.content_encoding {
                Some(e) => {
                    match e {
                        HttpEncoding::Gzip => "gzip",
                        HttpEncoding::Deflate => "deflate",
                        HttpEncoding::Br => "br",
                    }
                },
                None => "",
            },
            "Content-Length: ", content_length, CRLF,
            "Date: ", date, CRLF,
            "Server: ", server, CRLF,
            // 选择性地填入allow
            match &self.allow {
                Some(a) => {
                    let mut allow_str = String::new();
                    for (index, method) in a.iter().enumerate() {
                        allow_str.push_str(&format!("{}", method));
                        // 如果后面还有，就加一个逗号分隔
                        if index < a.len() - 1 {
                            allow_str.push_str(", ");
                        }
                    }
                    allow_str
                },
                None => "".to_string(),
            }.as_str(),
            CRLF,   // 分隔响应头和响应体的空行
            // 选择性地填入响应体
            match &self.content {
                Some(c) => str::from_utf8(&c).expect("Invalid UTF-8"),
                None => "",
            },
        ].concat().into()
    }
}

impl Response {
    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    pub fn information(&self) -> &str {
        &self.information
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
fn compress(data: Vec<u8>, mode: Option<HttpEncoding>) -> io::Result<Vec<u8>> {
    match mode {
        Some(HttpEncoding::Gzip) => {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&data)?;
            encoder.finish()
        },
        Some(HttpEncoding::Deflate) => {
            let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&data)?;
            encoder.finish()
        },
        Some(HttpEncoding::Br) => {
            let params = BrotliEncoderParams::default();
            let mut output = Vec::new();
            enc::BrotliCompress(&mut io::Cursor::new(data), &mut output, &params)?;
            Ok(output)
        },
        None => {
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
fn decide_encoding(accept_encoding: &Vec<HttpEncoding>) -> Option<HttpEncoding> {
    if accept_encoding.contains(&HttpEncoding::Gzip) {
        Some(HttpEncoding::Gzip)
    } else if accept_encoding.contains(&HttpEncoding::Deflate) {
        Some(HttpEncoding::Deflate)
    } else {
        None
    }
}

/// MIME
/// 
/// 保存了常见文件类型的映射关系
/// 
/// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types/Common_types>
fn get_mime(extension: &OsStr) -> &str {
    let extension = match extension.to_str() {
        Some(e) => e,
        None => {
            error!("无法将&OsStr转换为&str类型");
            return "application/octet-stream";
        }
    };
    match MIME_TYPES.get(extension) {
        Some(v) => v,
        None => "application/octet-stream",
    }
}