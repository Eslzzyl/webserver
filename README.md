## 基于Rust的Web服务器

这是合肥工业大学宣城校区2023年《计算机网络课程设计》项目。

## 功能 / Features

- 基于 Tokio 实现TCP连接的异步处理
- 手动解析 HTTP 请求，手动构造 HTTP 响应
- 支持 HTTP GET 请求和 HTTP 1.1
- 支持 HTTP 压缩，支持的编码有 Brotli, Gzip, Deflate，但目前优先使用 Gzip（Br 太慢）
- 通过 MIME 表支持常见的 Web 格式
- 支持简单的命令行控制
- 支持通过配置文件自定义 www root 文件夹和监听端口
- 通过`log4rs`支持简单的日志系统，支持记录到文件或标准输出
- 通过一个FIFO的文件缓存减少磁盘IO的次数

### 构建 / Build

安装最新稳定版的 Rust 工具链：[此处](https://www.rust-lang.org/learn/get-started)

Clone仓库，然后执行

```bash
cargo build
```

如需运行，执行

```bash
cargo run
```

测试服务器默认在`127.0.0.1`监听，默认的端口是`7878`，但可以在`files/config.toml`中更改。

程序启动后，打开浏览器，访问`127.0.0.1:7878`

程序只在Linux平台测试过，但理论上是跨平台的。

### 性能测试 / Benchmark

可能我代码有点问题，在高并发时会一次性打开大量socket连接，占用很多socket file discriptor，一旦超过操作系统设定的`nlimit`限制（默认为1024），就会panic。

#### 暂时提升`nlimit`限制

`cargo run`启动之后，查找进程PID
```bash
ps -e | grep webserver
```

然后暂时提升限制
```bash
sudo prlimit --pid [PID] --nofile=32768:32768
```

之后再进行测试。

#### 测试结果

```bash
eslzzyl:~/W/c/webbench-1.5 $ ./webbench -c 10000 -t 10 --get --http11 http://127.0.0.1:7878/
Webbench - Simple Web Benchmark 1.5
Copyright (c) Radim Kolar 1997-2004, GPL Open Source Software.

Benchmarking: GET http://127.0.0.1:7878/ (using HTTP/1.1)
10000 clients, running 10 sec.

Speed=2991498 pages/min, 44423480 bytes/sec.
Requests: 498583 susceed, 0 failed.
```

测试20000并发时，端口号不够用了。肯定是我代码的问题，正常不应该是这样的

测试机器：AMD Ryzen 5 4600U, 16G DDR4, Ubuntu 22.04

### 参考文献

- 《HTTP权威指南》 David Gourley 等著(2002)，陈涓 等译（人民邮电出版社 2012年版）
- RFC 2616 Hypertext Transfer Protocol -- HTTP/1.1
- RFC 7230-7235
- RFC 9110

### 开发

#### 功能添加和调整



#### 注意事项

不能让URI退到wwwroot之外，如`www.example.com/../`。对于这种请求，应该给一个拒绝访问的Response。

#### HTTP压缩

Deflate, Gzip:

https://crates.io/crates/flate2

brotli(br):

https://crates.io/crates/brotli

压缩已经实现，但brotli非常慢，因此默认启用gzip，无论浏览器是否支持brotli。