# rfeign

Rust 声明式 HTTP 客户端，灵感来自 OpenFeign / Retrofit。

## 特性

- **声明式 API** — 用 trait + 属性宏定义 HTTP 客户端
- **命令式 API** — 链式 Builder 发送请求
- **服务发现** — Nacos / Consul 集成
- **弹性能力** — 重试 (reqwest-retry)、熔断器 (recloser)、超时、请求取消
- **文件上传** — `#[multipart]` + `Part` 支持 multipart/form-data
- **认证** — Bearer / Basic / DynamicAuth（token 自动刷新）
- **日志** — 请求/响应日志，可配置 LogLevel（None/Basic/Headers/Full）
- **流式响应** — `ByteStream` 大文件下载
- **cURL 生成** — `.to_curl()` 调试利器

## 快速开始

```toml
[dependencies]
rfeign = "0.0.1"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

### 声明式（宏）

```rust
use rfeign::{ClientBuilder, ReqwestTransport};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct User { id: i64, name: String }

#[rfeign::http_client(base_url = "http://localhost:8080", path = "/api/v1")]
#[headers("Accept: application/json")]
trait UserApi {
    #[rfeign::get("/users/{id}")]
    async fn get_user(&self, #[path] id: i64) -> rfeign::Result<User>;

    #[rfeign::get("/users")]
    async fn list_users(&self, #[query] page: u32, #[query] size: u32)
        -> rfeign::Result<Vec<User>>;
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

### 命令式（无宏）

```rust
let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("https://api.example.com")
    .build();

let user: User = client.get("/users/1")
    .header("Authorization", "Bearer token")
    .timeout(Duration::from_secs(5))
    .send()
    .await?
    .json()?;
```

## 参数属性

| 属性 | 用法 | 说明 |
|------|------|------|
| `#[path]` | `#[path] id: i64` | 路径变量 `{id}` |
| `#[path(name = "x")]` | `#[path(name = "userId")] id: i64` | 重命名路径变量 |
| `#[query]` | `#[query] page: u32` | 查询参数 |
| `#[query(format = "multi")]` | `#[query(format = "multi")] ids: Vec<i64>` | 重复参数 `?ids=1&ids=2` |
| `#[query(format = "csv")]` | `#[query(format = "csv")] ids: Vec<i64>` | CSV 格式 `?ids=1,2,3` |
| `#[body]` | `#[body] user: CreateUser` | JSON 请求体 |
| `#[header("X")]` | `#[header("Authorization")] token: String` | 动态请求头 |
| `#[part(name = "x")]` | `#[part(name = "file")] file: Part` | Multipart 字段 |

## 方法属性

```rust
#[rfeign::get("/path")]          // HTTP 方法 + 路径
#[rfeign::timeout(5000)]         // 方法级超时（毫秒）
#[rfeign::multipart]             // Multipart 请求
#[rfeign::header("K", "V")]     // 方法级静态请求头
```

## 弹性能力

```rust
// Transport 层：reqwest-retry + reqwest-tracing
let transport = ReqwestTransport::builder()
    .retry(3)           // 指数退避重试
    .tracing()          // OpenTelemetry 链路追踪
    .build();

// Middleware 层：日志 + 熔断器
let client = ClientBuilder::new(transport)
    .base_url("https://api.example.com")
    .middleware(LoggingMiddleware::new(LogLevel::Headers))
    .middleware(CircuitBreakerMiddleware::new(0.5, Duration::from_secs(30)))
    .build();
```

## 服务发现

```rust
// Nacos
let resolver = NacosResolverBuilder::new("127.0.0.1:8848")
    .namespace("public")
    .build().await?;

// Consul
let resolver = ConsulResolver::new("http://127.0.0.1:8500");

let client = ClientBuilder::new(ReqwestTransport::new())
    .url_resolver(resolver)
    .service_name("user-service")
    .build();
```

## 认证

```rust
// 静态 Bearer Token
let client = ClientBuilder::new(transport)
    .auth(BearerAuth::new("your-token"))
    .build();

// 动态 Token（自动刷新）
let client = ClientBuilder::new(transport)
    .auth(DynamicAuth::new(MyTokenProvider))
    .build();
```

## Feature Flags

| Feature | 说明 |
|---------|------|
| `reqwest`（默认） | reqwest 传输层 |
| `json`（默认） | JSON 编解码 |
| `retry` | reqwest-retry 重试中间件 |
| `tracing` | reqwest-tracing 链路追踪 |
| `circuit-breaker` | recloser 熔断器 |
| `nacos` | Nacos 服务发现 |
| `consul` | Consul 服务发现 |

## License

MIT
