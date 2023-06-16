use std::{
    path::PathBuf,
    process::Command,
};

use chrono::{DateTime, Local};
use log::error;

use crate::{
    param::STATUS_CODES,
    exception::Exception,
};

/// `HtmlBuilder`
/// 
/// 用于动态构建HTML页面的结构
pub struct HtmlBuilder {
    title: String,
    css: String,
    script: String,
    body: String,
}

impl HtmlBuilder {
    /// 通过状态码创建一个`HtmlBuilder`
    /// 
    /// ## 参数：
    /// - `code`: HTML状态码
    /// - `note`: 自定义的说明文字。可以没有。
    /// 
    /// ## 返回
    /// - 一个`HtmlBuilder`对象。要构建它，使用`build()`。
    /// 
    /// ## 示例
    /// ```rust
    /// // 带有自定义说明文字的HTML
    /// let html404: String = HtmlBuilder::from_status_code(404, Some("无法找到网页")).build();
    /// // 不指定说明文字，此时使用默认的说明（“I'm a teapot”）
    /// let html418: String = HtmlBuilder::from_status_code(418, None).build();
    /// ```
    pub fn from_status_code(code: u16, note: Option<&str>) -> Self {
        let title = format!("{}", code);
        let css = r"
            body {
                width: 35em;
                margin: 0 auto;
                font-family: Tahoma, Verdana, Arial, sans-serif;
            }
            ".to_string();
        let description = match note {
            // 如果有自定义说明，则使用自定义说明
            Some(n) => n,
            // 否则，使用预定义的状态码说明
            None => match STATUS_CODES.get(&code) {
                Some(d) => *d,
                None => {
                    panic!("非法的状态码：{}", code);
                }
            }
        };
        let body = format!(
            r"
            <h1>{}</h1>
            <p>{}</p>
            ", code, description
        );
        Self {
            title,
            css,
            script: "".to_string(),
            body,
        }
    }

    /// 通过文件列表创建一个`HtmlBuilder`
    /// 
    /// ## 参数
    /// - `path`: 路径名
    /// - `dir_vec`: 文件列表
    /// 
    /// ## 返回
    /// - 一个`HtmlBuilder`对象。要构建它，使用`build()`。
    pub fn from_dir(path: &str, dir_vec: &Vec<PathBuf>) -> Self {
        let mut body = String::new();
        body.push_str(&format!("<h1>{}下的文件列表</h1>", path));
        body.push_str("<table>");
        body.push_str(r"
            <tr>
                <td>文件名</td>
                <td>大小</td>
                <td>修改时间</td>
            </tr>
            "
        );
        for entry in dir_vec {
            let metadata = entry.metadata().unwrap();
            // 使用本地时区格式化为当前本地时间
            let local_time: DateTime<Local> = metadata.modified().unwrap().into();
            let formatted_local = local_time.format("%Y-%m-%d %H:%M:%S %Z").to_string();
            let filename = entry.file_name().unwrap().to_string_lossy();
            if entry.is_file() {
                let size = metadata.len();
                let formatted_size = format_file_size(size);
                body.push_str(&format!(
                    r"
                    <tr>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{}</td>
                    </tr>
                    ",
                    &filename,
                    &formatted_size,
                    &formatted_local
                ));
            } else if entry.is_dir() {
                body.push_str(&format!(
                    r"
                    <tr>
                        <td>{}</td>
                        <td>DIR</td>
                        <td>{}</td>
                    </tr>
                    ",
                    &filename,
                    &formatted_local
                ));
            } else {
                // 虽然我觉得这个条件永远不会被访问到。
                panic!();
            }
        }
        body.push_str("</table>");
        let title = format!("{}", path);
        let css = r"
            table {
                border-collapse: collapse;
                width: 100%;
            }

            td {
                padding: 8px;
                white-space: pre-wrap; /* 保留换行符和空格 */
                border: none; /* 隐藏单元格边框 */
            }

            th {
                padding: 8px;
                border: none; /* 隐藏表头边框 */
            }".to_string();
        HtmlBuilder {
            title,
            css,
            script: "".to_string(),
            body,
        }
    }

    /// 构建一个`HtmlBuilder`
    pub fn build(&self) -> String {
        format!(r##"<!DOCTYPE html>
            <html>
                <head>
                    <meta charset="utf-8">
                    <script>{}</script>
                    <title>{}</title>
                    <style>{}</style>
                </head>
                <body>
                {}
                </body>
            </html>"##,
            self.script, self.title, self.css, self.body
        )
    }
}

/// 格式化文件大小
/// 
/// ## 参数
/// - `size`: 以字节为单位的文件大小
/// 
/// ## 返回
/// - 格式化后的文件大小，原始大小的单位将被动态地调整到`B`、`KB`、`MB`、`GB`、`TB`等单位，并保留1位小数。
fn format_file_size(size: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < units.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1} {}", size, units[unit_index])
}

/// 处理对PHP文件的请求
pub fn handle_php(path: &str, id: u128) -> Result<String, Exception> {
    let result = Command::new("php")
        .arg(path) // PHP文件路径
        .output();
    let output = match result {
        Ok(o) => o,
        Err(_) => return Err(Exception::PHPExecuteFailed)
    };

    if output.status.success() {    // 执行完毕
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(String::from(stdout))
    } else {    // 解释器出错
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("[ID{}]PHP解释器出错：{}", id, stderr);
        Err(Exception::PHPCodeError)
    }
}