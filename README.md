## 基于Rust的Web服务器

这是合肥工业大学宣城校区2023年《计算机网络课程设计》项目。题目如下：

> ### 设计目的
>
> 1. 熟悉开发工具(Visual Studio、C/C++、Java 等)的基本操作；
> 2. 掌握 http 协议的工作原理；
> 3. 掌握多线程编程；
> 4. 对于 Socket 编程建立初步的概念。
> 5. 掌握对文件的网络传输操作；
>
> ### 设计要求
>
> 1. 不限平台，熟悉 Socket API 主要函数的使用；
> 2. 实现一个简单的基于 http 协议的 WEB 服务器；
> 3. 实现对服务器运行状态的监控；
>
> ### 设计内容
>
> 请注意:
>
> 1. 此处 Web 服务器，只是对 HTTP 请求予以应答；IE 浏览器访问本服务器，请求当前服务器中的某静态网页文件(html 或 htm 文件等)，服务器端查询服务器端相应的路径下该网页是否存在，如存在，则利用当前的 TCP 连接传递该网页文件，如果不存在，则返回 404 错误提示。
> 2. 不涉及动态网页的解析，如`asp`、`aspx`、`php`、`jsp`等；
> 3. 应考虑服务器的多客户端访问问题，参见：多线程机制、异步的套接字I/O机制或套接字链表等等；
>
> ### 思考题
>
> 1. 该服务器的工作模式是什么？
> 2. 如何对其进行测试，还有哪些可以完善的功能？
> 3. 有什么办法可以提高它的性能？

## 功能 / Features

- 基于 Tokio 实现 TCP 连接的异步处理
- 手动解析 HTTP 请求，手动构造 HTTP 响应
- 支持 HTTP 的 GET、HEAD 请求，部分地支持 OPTIONS 请求（不支持CORS的预检请求）
- 支持 HTTP 1.1
- 支持 HTTP 压缩，支持的编码有 Brotli, Gzip, Deflate，但目前优先使用 Gzip（Br 太慢）
- 通过 MIME 表支持常见的 Web 格式
- 支持简单的命令行控制
- 支持通过配置文件自定义 www root 文件夹和监听端口
- 通过`log4rs`支持简单的日志系统，支持记录到文件或标准输出
- 通过一个FIFO的文件缓存减少磁盘IO的次数
- 支持文件列表模式
    - 支持超链接跳转
    - 文件列表自动排序
    - 表格排版，清晰易读
- 状态码页面动态生成
- 简单的PHP页面支持

各种请求方法的测试：
- GET：使用浏览器测试
- HEAD
    ```bash
    eslzzyl:~$ curl --head 127.0.0.1:7878/ -i
    HTTP/1.1 200 OK
    Content-Length: 858
    Date: Mon, 19 Jun 2023 09:38:16 +0000
    Server: eslzzyl-webserver
    ```
- OPTIONS
    ```bash
    eslzzyl:~$ curl -X OPTIONS 127.0.0.1:7878 -i
    HTTP/1.1 204 No Content
    Content-Length: 0
    Date: Mon, 19 Jun 2023 09:22:51 +0000
    Server: eslzzyl-webserver
    Allow: GET, HEAD, OPTIONS
    ```

### 构建 / Build

安装最新稳定版的 Rust stable 工具链：[此处](https://www.rust-lang.org/learn/get-started)。我在编写代码时使用的版本是`1.69.0`。

Clone仓库，然后执行

```bash
cargo build
```
在构建之前，如果需要，可以修改crates.io的索引以加快依赖下载。见[此处](https://mirrors.tuna.tsinghua.edu.cn/help/crates.io-index/)。

### 运行 / Run

1. 安装PHP环境。在Ubuntu下，使用
    ```bash
    sudo apt install php
    ```
    - 在其他系统（如Windows）中，可能需要手动配置环境变量。
    - PHP不是必要的，但是没有PHP环境则无法使用PHP扩展，服务器将返回500。
2. 执行
    ```bash
    cargo run
    ```

测试服务器默认在`127.0.0.1`监听，默认的端口是`7878`，但可以在`files/config.toml`中更改。

如果要在云服务器上运行：
- 将`files/config.toml`中的`local`项改为`false`
- 在云服务器防火墙或安全组管理中放通指定的**TCP入站端口**（默认端口为7878）
- 这个玩具服务器**很不安全**，特别是PHP支持的部分，没有任何安全防护措施。一定不要让它在公网环境长期运行。

程序启动后，打开浏览器，访问`127.0.0.1:7878`。如果运行在公网，则将IP替换为对应的公网IP。

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

- 本机：

    ```bash
    eslzzyl:~/W/c/webbench-1.5 $ ./webbench -c 10000 -t 10 --get --http11 http://127.0.0.1:7878/
    Webbench - Simple Web Benchmark 1.5
    Copyright (c) Radim Kolar 1997-2004, GPL Open Source Software.

    Benchmarking: GET http://127.0.0.1:7878/ (using HTTP/1.1)
    10000 clients, running 10 sec.

    Speed=2991498 pages/min, 44423480 bytes/sec.
    Requests: 498583 susceed, 0 failed.
    ```

    测试20000并发时，端口号不够用了。肯定是我代码的问题，正常不应该是这样的。总之10000并发肯定是有的。

    测试机器：AMD Ryzen 5 4600U, 16G DDR4, Ubuntu 22.04

- 远程服务器：

    ```bash
    eslzzyl:~/W/c/webbench-1.5 $ ./webbench -c 10000 -t 10 --get --http11 http://xx.xx.xx.xx:7878/
    Webbench - Simple Web Benchmark 1.5
    Copyright (c) Radim Kolar 1997-2004, GPL Open Source Software.
    
    Benchmarking: GET http://xx.xx.xx.xx:7878/ (using HTTP/1.1)
    10000 clients, running 10 sec.
    
    Speed=6864 pages/min, 101930 bytes/sec.
    Requests: 1144 susceed, 0 failed.
    ```

    ```bash
    eslzzyl:~/W/c/webbench-1.5 $ ./webbench -c 12000 -t 10 --get --http11 http://xx.xx.xx.xx:7878/
    Webbench - Simple Web Benchmark 1.5
    Copyright (c) Radim Kolar 1997-2004, GPL Open Source Software.
    
    Benchmarking: GET http://xx.xx.xx.xx:7878/ (using HTTP/1.1)
    12000 clients, running 10 sec.
    
    Speed=5430 pages/min, 81993 bytes/sec.
    Requests: 905 susceed, 0 failed.
    ```

    测试20000并发时同样出现问题。

    测试机器：腾讯云上海 1核 2G 1M, Ubuntu 22.04

### 参考文献

- 《HTTP权威指南》 David Gourley 等著(2002)，陈涓 等译（人民邮电出版社 2012年版）
- https://developer.mozilla.org/zh-CN/docs/Web/HTTP
- RFC 2616 Hypertext Transfer Protocol -- HTTP/1.1
- RFC 7230-7235
- [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.htm)

### 开发

#### 功能添加和调整

- ~实现LRU缓存：困难。因为不得不用`unsafe`，而`unsafe`结构在线程之间传递太可怕了。~
- 看一看PHP的安全性方面有没有能挖掘的地方

找个机会精简一下依赖，目前依赖快100个，编译太慢了，很多依赖只是用到一个简单的功能，没必要用库。尤其是`Config`的读取那部分，`serde`的依赖有很多

#### 注意事项

不能让URI退到wwwroot之外，如`www.example.com/../`。对于这种请求，应该给一个拒绝访问的Response。

#### HTTP压缩

Deflate, Gzip:

https://crates.io/crates/flate2

brotli(br):

https://crates.io/crates/brotli

压缩已经实现，但brotli非常慢，因此默认启用gzip，无论浏览器是否支持brotli。
