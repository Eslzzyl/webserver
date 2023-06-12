use crate::param::STATUS_CODES;

pub struct HtmlBuilder {
    title: String,
    css: String,
    script: String,
    body: String,
}

impl HtmlBuilder {
    pub fn from_status_code(code: u16) -> Self {
        assert_ne!(code, 404);  // 404有独特的定制html，不应自动生成
        let title = format!("{}", code);
        let css = r"body {
            width: 35em;
            margin: 0 auto;
            font-family: Tahoma, Verdana, Arial, sans-serif;
          }".to_string();
        let description = match STATUS_CODES.get(&code) {
            Some(d) => *d,
            None => {
                panic!("非法的状态码：{}", code);
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