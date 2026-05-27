# Configuration Reference

## Feature Flags

Enable features in your `Cargo.toml`:

```toml
[dependencies]
rfeign = { version = "0.0.1", features = ["retry", "circuit-breaker"] }
```

| Feature           | Description                              | Dependencies              |
|-------------------|------------------------------------------|---------------------------|
| `reqwest`         | Reqwest-based HTTP transport (default)   | reqwest                   |
| `json`            | JSON serialization/deserialization (default) | serde_json            |
| `retry`           | Automatic retry with exponential backoff | reqwest-retry             |
| `tracing`         | Distributed tracing integration          | reqwest-tracing           |
| `circuit-breaker` | Circuit breaker middleware               | recloser                  |
| `middleware`      | reqwest-middleware support               | reqwest-middleware        |
| `middleware-full` | Enables retry + tracing + circuit-breaker | All above                |
| `nacos`           | Nacos service discovery                  | nacos-sdk                 |
| `consul`          | Consul service discovery                 | rs-consul                 |

Default features: `reqwest`, `json`.

## ClientBuilder Methods

| Method                | Description                                |
|-----------------------|--------------------------------------------|
| `base_url(url)`       | Set a static base URL                      |
| `url_resolver(r)`     | Set a dynamic URL resolver                 |
| `service_name(name)`  | Set the service name for discovery         |
| `auth(impl Auth)`     | Set authentication strategy                |
| `bearer_auth(token)`  | Shorthand for Bearer token auth            |
| `basic_auth(user, pass)` | Shorthand for HTTP Basic auth           |
| `interceptor(i)`     | Add a request interceptor                  |
| `response_interceptor(i)` | Add a response interceptor            |
| `middleware(m)`       | Add a middleware (logging, circuit breaker) |
| `timeout(t)`          | Set global timeout configuration           |
| `connect_timeout(dur)` | Set connection timeout                   |
| `read_timeout(dur)`   | Set read timeout                           |
| `write_timeout(dur)`  | Set write timeout                          |
| `log_level(level)`    | Set logging verbosity                      |
| `encoder(e)`          | Custom request body encoder                |
| `decoder(d)`          | Custom response body decoder               |
| `error_decoder(d)`    | Custom error response decoder              |
| `success_status(fn)`  | Custom success status predicate            |
| `build()`             | Build the `Client` instance                |

### Example

```rust
use std::time::Duration;
use rfeign::{ClientBuilder, ReqwestTransport, LogLevel, LoggingMiddleware};

let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("https://api.example.com")
    .bearer_auth("my-token")
    .connect_timeout(Duration::from_secs(5))
    .read_timeout(Duration::from_secs(30))
    .middleware(LoggingMiddleware::new(LogLevel::Headers))
    .log_level(LogLevel::Headers)
    .build();
```

## ReqwestTransport::builder()

The transport builder configures the underlying reqwest client and its middleware stack.

| Method          | Feature Required | Description                          |
|-----------------|-----------------|--------------------------------------|
| `client(c)`    | —               | Use a custom `reqwest::Client`       |
| `retry(n)`     | `retry`         | Add retry with max N attempts        |
| `tracing()`    | `tracing`       | Add OpenTelemetry tracing            |
| `with(mw)`     | `middleware`    | Add custom reqwest middleware         |
| `build()`      | —               | Build the `ReqwestTransport`         |

### Example

```rust
let transport = ReqwestTransport::builder()
    .client(reqwest::Client::builder()
        .pool_max_idle_per_host(10)
        .build()
        .unwrap())
    .retry(3)
    .tracing()
    .build();

let client = ClientBuilder::new(transport)
    .base_url("https://api.example.com")
    .build();
```
