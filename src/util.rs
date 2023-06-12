use crate::param::STATUS_CODES;

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
        assert_ne!(code, 404);  // 404有独特的定制html，不应自动生成
        let title = format!("{}", code);
        let css = r"body {
            width: 35em;
            margin: 0 auto;
            font-family: Tahoma, Verdana, Arial, sans-serif;
          }".to_string();
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
            r"<h2>{}</h2>
            <p>{}</p>", code, description
        );
        Self {
            title,
            css,
            script: "".to_string(),
            body,
        }
    }

    /// 构建一个`HtmlBuilder`
    pub fn build(&self) -> String {
        format!(r##"<!DOCTYPE html>
            <html><head>
                <meta charset="utf-8">
                <script>{}</script>
                <title>{}</title>
                <style>{}</style>
            </head>
            <body>{}</body></html>"##,
            self.script, self.title, self.css, self.body
        )
    }
}