# rfeign

A declarative HTTP client for Rust, inspired by OpenFeign / Retrofit.

[中文文档](README.zh.md)

## Features

- Declarative & imperative API
- Service discovery (Nacos / Consul)
- Retry, circuit breaker, timeout, cancellation
- Multipart file upload
- Auth (Bearer / Basic / DynamicAuth)
- Request/response logging
- Streaming downloads

## Quick Start

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
