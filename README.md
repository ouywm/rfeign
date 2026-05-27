# rfeign

A declarative HTTP client for Rust, inspired by OpenFeign / Retrofit.

## Features

- **Declarative API** — define HTTP clients as traits with attribute macros
- **Imperative API** — fluent builder for one-off requests
- **Service Discovery** — Nacos / Consul integration
- **Resilience** — retry (reqwest-retry), circuit breaker (recloser), timeout, cancellation
- **File Upload** — multipart/form-data with `#[multipart]` + `Part`
- **Auth** — Bearer / Basic / DynamicAuth (auto token refresh)
- **Logging** — request/response logging with configurable LogLevel
- **Streaming** — `ByteStream` for large file downloads
- **cURL** — `.to_curl()` for debugging

## Quick Start

```toml
[dependencies]
rfeign = "0.0.1"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

### Declarative (macro-based)

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

### Imperative (no macros)

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

## Parameter Attributes

| Attribute | Usage | Description |
|-----------|-------|-------------|
| `#[path]` | `#[path] id: i64` | Path variable `{id}` |
| `#[path(name = "x")]` | `#[path(name = "userId")] id: i64` | Renamed path var |
| `#[query]` | `#[query] page: u32` | Query parameter |
| `#[query(format = "multi")]` | `#[query(format = "multi")] ids: Vec<i64>` | Repeated params `?ids=1&ids=2` |
| `#[query(format = "csv")]` | `#[query(format = "csv")] ids: Vec<i64>` | CSV format `?ids=1,2,3` |
| `#[body]` | `#[body] user: CreateUser` | JSON request body |
| `#[header("X")]` | `#[header("Authorization")] token: String` | Dynamic header |
| `#[part(name = "x")]` | `#[part(name = "file")] file: Part` | Multipart field |

## Method Attributes

```rust
#[rfeign::get("/path")]          // HTTP method + path
#[rfeign::timeout(5000)]         // Method-level timeout (ms)
#[rfeign::multipart]             // Multipart request
#[rfeign::header("K", "V")]     // Static header on this method
```

## Resilience

```rust
let transport = ReqwestTransport::builder()
    .retry(3)           // reqwest-retry with exponential backoff
    .tracing()          // reqwest-tracing for OpenTelemetry
    .build();

let client = ClientBuilder::new(transport)
    .base_url("https://api.example.com")
    .middleware(LoggingMiddleware::new(LogLevel::Headers))
    .middleware(CircuitBreakerMiddleware::new(0.5, Duration::from_secs(30)))
    .build();
```

## Service Discovery

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

## Feature Flags

| Feature | Description |
|---------|-------------|
| `reqwest` (default) | reqwest transport |
| `json` (default) | JSON codec |
| `retry` | reqwest-retry middleware |
| `tracing` | reqwest-tracing middleware |
| `circuit-breaker` | recloser circuit breaker |
| `nacos` | Nacos service discovery |
| `consul` | Consul service discovery |

## License

MIT
