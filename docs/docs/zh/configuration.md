# 配置参考

## Feature Flags

rfeign 通过 Cargo feature flags 控制可选功能：

| Feature | 说明 | 依赖 |
|---------|------|------|
| `reqwest` | 基于 reqwest 的 HTTP 传输层（默认） | reqwest |
| `json` | JSON 编解码支持（默认） | serde_json |
| `retry` | 请求重试（指数退避） | reqwest-retry |
| `tracing` | 分布式追踪集成 | reqwest-tracing |
| `circuit-breaker` | 熔断器 | recloser |
| `nacos` | Nacos 服务发现 | nacos-sdk |
| `consul` | Consul 服务发现 | rs-consul |

在 `Cargo.toml` 中启用：

```toml
[dependencies]
rfeign = { version = "0.0.1", features = ["retry", "circuit-breaker"] }
```

## ClientBuilder 方法

| 方法 | 说明 |
|------|------|
| `base_url(url)` | 设置静态基础 URL |
| `url_resolver(resolver)` | 设置动态 URL 解析器 |
| `service_name(name)` | 设置服务名称 |
| `auth(impl Auth)` | 设置认证策略 |
| `bearer_auth(token)` | Bearer Token 认证 |
| `basic_auth(user, pass)` | Basic 认证 |
| `interceptor(impl RequestInterceptor)` | 添加请求拦截器 |
| `response_interceptor(impl ResponseInterceptor)` | 添加响应拦截器 |
| `middleware(impl Middleware)` | 添加中间件 |
| `timeout(Timeout)` | 设置全局超时配置 |
| `connect_timeout(Duration)` | 连接超时 |
| `read_timeout(Duration)` | 读取超时 |
| `write_timeout(Duration)` | 写入超时 |
| `log_level(LogLevel)` | 日志级别 |
| `encoder(impl Encoder)` | 自定义编码器 |
| `decoder(impl Decoder)` | 自定义解码器 |
| `error_decoder(impl ErrorDecoder)` | 自定义错误解码器 |
| `success_status(fn(u16) -> bool)` | 自定义成功状态判断 |

示例：

```rust
use std::time::Duration;
use rfeign::{ClientBuilder, ReqwestTransport, LogLevel};

let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("https://api.example.com")
    .bearer_auth("my-token")
    .connect_timeout(Duration::from_secs(5))
    .read_timeout(Duration::from_secs(30))
    .log_level(LogLevel::Headers)
    .build();
```

## ReqwestTransport::builder() 配置

`ReqwestTransportBuilder` 用于配置底层 reqwest 传输层：

```rust
use rfeign::ReqwestTransport;

let transport = ReqwestTransport::builder()
    .client(reqwest::Client::new())  // 自定义 reqwest Client
    .retry(3)                         // 启用重试，最多 3 次（需 retry feature）
    .tracing()                        // 启用 tracing（需 tracing feature）
    .build();
```

| 方法 | 说明 | 所需 Feature |
|------|------|-------------|
| `client(reqwest::Client)` | 自定义底层 reqwest Client | - |
| `retry(max_retries)` | 指数退避重试 | `retry` |
| `tracing()` | 请求追踪 | `tracing` |
| `with(impl Middleware)` | 添加 reqwest-middleware | `middleware` |
