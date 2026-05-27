# Resilience

rfeign provides retry, circuit breaker, timeout, and cancellation out of the box.

## Retry

Enable the `retry` feature and configure via the transport builder:

```toml
[dependencies]
rfeign = { version = "0.0.1", features = ["retry"] }
```

```rust
use rfeign::{ClientBuilder, ReqwestTransport};

let transport = ReqwestTransport::builder()
    .retry(3) // max 3 retry attempts with exponential backoff
    .build();

let client = ClientBuilder::new(transport)
    .base_url("https://api.example.com")
    .build();
```

Retries are triggered on transient errors (5xx, timeouts, connection failures). The backoff policy uses exponential delays.

## Circuit Breaker

Enable the `circuit-breaker` feature:

```toml
[dependencies]
rfeign = { version = "0.0.1", features = ["circuit-breaker"] }
```

```rust
use std::time::Duration;
use rfeign::circuit_breaker::CircuitBreakerMiddleware;
use rfeign::{ClientBuilder, ReqwestTransport};

let cb = CircuitBreakerMiddleware::new(
    0.5,                          // open when error rate exceeds 50%
    Duration::from_secs(30),      // wait 30s before half-open
);

let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("https://api.example.com")
    .middleware(cb)
    .build();
```

When the circuit is open, requests fail immediately with `Error::CircuitOpen`.

Use `default_with_wait` for a 50% error rate threshold with custom wait:

```rust
let cb = CircuitBreakerMiddleware::default_with_wait(Duration::from_secs(15));
```

## Timeout

### Imperative (per-request)

```rust
use std::time::Duration;

let resp = client.get("/slow")
    .timeout(Duration::from_secs(5))
    .send()
    .await?;
```

### Declarative (macro-level)

Use `#[timeout(ms)]` to set a per-method timeout in milliseconds:

```rust
#[rfeign::http_client(base_url = "http://localhost:8080")]
trait Api {
    #[rfeign::get("/data")]
    #[rfeign::timeout(5000)] // 5 second timeout
    async fn get_data(&self) -> rfeign::Result<Data>;
}
```

### Global timeout

Set default timeouts on the client builder:

```rust
let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("https://api.example.com")
    .connect_timeout(Duration::from_secs(5))
    .read_timeout(Duration::from_secs(30))
    .write_timeout(Duration::from_secs(10))
    .build();
```

## Cancellation

Use `tokio_util::sync::CancellationToken` to cancel in-flight requests:

```rust
use std::time::Duration;
use tokio_util::sync::CancellationToken;

let token = CancellationToken::new();
let cancel = token.clone();

// Cancel after 2 seconds from another task
tokio::spawn(async move {
    tokio::time::sleep(Duration::from_secs(2)).await;
    cancel.cancel();
});

let result = client.get("/long-running")
    .cancel_token(token)
    .send()
    .await;

match result {
    Err(rfeign::Error::Cancelled) => println!("Request was cancelled"),
    Ok(resp) => println!("Got response: {}", resp.status()),
    Err(e) => println!("Other error: {}", e),
}
```

## Combining Strategies

You can stack retry, circuit breaker, and timeout together:

```rust
use std::time::Duration;
use rfeign::{ClientBuilder, ReqwestTransport, LogLevel, LoggingMiddleware};
use rfeign::circuit_breaker::CircuitBreakerMiddleware;

let transport = ReqwestTransport::builder()
    .retry(3)
    .build();

let client = ClientBuilder::new(transport)
    .base_url("https://api.example.com")
    .middleware(CircuitBreakerMiddleware::new(0.5, Duration::from_secs(30)))
    .middleware(LoggingMiddleware::new(LogLevel::Basic))
    .connect_timeout(Duration::from_secs(5))
    .build();
```
