## 基于Rust的Web服务器

这是合肥工业大学宣城校区2023年《计算机网络课程设计》项目。

文档有空再写

### 参考文献

- 《HTTP权威指南》 David Gourley 等著(2002)，陈涓 等译（人民邮电出版社 2012年版）
- RFC 2616 Hypertext Transfer Protocol -- HTTP/1.1
- RFC 7230-7235
- RFC 9110

### 开发

#### 功能添加和调整

考虑使用httparse库取代http请求头解析部分。但是应注意工作量。

https://github.com/seanmonstar/httparse

添加日志记录功能，找个合适的crate

也许应该加个MIME表，但是还没想到合适的实现。在足够复杂之前先硬编码吧。

#### 注意事项

不能让URI退到wwwroot之外，如`www.example.com/../`。对于这种请求，应该给一个拒绝访问的Response。

#### HTTP压缩

Deflate, Gzip:

https://crates.io/crates/flate2

brotli(br):

https://crates.io/crates/brotli