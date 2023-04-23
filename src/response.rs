use crate::param::*;

use chrono::prelude::*;

#[derive(Debug, Clone)]
pub struct Response {
    version: HttpVersion,
    status_code: u16,
    information: String,
    server: String,
    content_type: String,
    content_length: usize,
    date: DateTime<Utc>,
    content_encoding: HttpEncoding,     // 响应仅指定一种压缩编码即可。若浏览器支持多种，则具体采用哪种由Config决定
}

impl Response {
    pub fn new() -> Self {
        Self {
            version: HttpVersion::V1_1,
            status_code: 200,
            information: "OK".to_string(),
            server: "eslzzyl-webserver".to_string(),
            content_type: "text/plain;charset=utf-8".to_string(),
            content_length: 0,
            date: Utc::now(),
            content_encoding: HttpEncoding::Gzip,
        }
    }

    // 注意：首部总是以一个空行（仅包含一个CRLF）结束，即使没有主体部分也是如此。
    pub fn to_string(&self) -> String {
        // TODO
        "".to_string()
    }
}

fn format_date(date: &DateTime<Utc>) -> String {
    date.to_rfc2822()
}