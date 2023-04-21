use crate::exception::Exception;

enum HttpRequestMethod {
    Post,
    Get,
}

enum HttpRequestVersion {
    V1_1,
}

enum HttpRequestEncoding {
    Plain,
    Gzip,
    Deflate,
    Br,
}

pub struct Request {
    method: HttpRequestMethod,
    path: String,
    version: HttpRequestVersion,
    user_agent: String,     // UA
    accept_encoding: Vec<HttpRequestEncoding>,  // 压缩编码，可以支持多种编码
}

impl Request {
    pub fn new() -> Self {
        Self {
            method: HttpRequestMethod::Get,
            path: "".into(),
            version: HttpRequestVersion::V1_1,
            user_agent: "".into(),
            accept_encoding: vec!(HttpRequestEncoding::Plain),
        }
    }

    pub fn try_from(buffer: Vec<u8>) -> Result<Self, Exception> {
        let request_string = match String::from_utf8(buffer) {
            Ok(string) => string,
            Err(e) => {
                println!("Error when parsing request!");
                return Err(Exception::RequestConstructFailed);
            }
        };

        // TODO 解析请求，然后用解析的结果替代返回值

        Ok(Self {
            method: HttpRequestMethod::Get,
            path: "".into(),
            version: HttpRequestVersion::V1_1,
            user_agent: "".into(),
            accept_encoding: vec!(HttpRequestEncoding::Plain),
        })
    }
}