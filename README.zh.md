# rfeign

Rust 声明式 HTTP 客户端，灵感来自 OpenFeign / Retrofit。

[English](README.md)

## 特性

- 声明式 & 命令式 API
- 服务发现（Nacos / Consul）
- 重试、熔断器、超时、请求取消
- Multipart 文件上传
- 认证（Bearer / Basic / DynamicAuth 自动刷新）
- 请求/响应日志
- 流式下载

## 快速开始

```toml
[dependencies]
rfeign = "0.0.1"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

```rust
use rfeign::{ClientBuilder, ReqwestTransport};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct User { id: i64, name: String }

#[rfeign::http_client(base_url = "http://localhost:8080")]
trait UserApi {
    #[rfeign::get("/users/{id}")]
    async fn get_user(&self, #[path] id: i64) -> rfeign::Result<User>;
}

#[tokio::main]
async fn main() {
    let client = ClientBuilder::new(ReqwestTransport::new())
        .base_url("http://localhost:8080")
        .build();
    let api = UserApiClient::new(client);
    let user = api.get_user(1).await.unwrap();
}
```

## License

MIT
