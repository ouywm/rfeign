# 弹性能力

rfeign 提供重试、熔断、超时和请求取消等弹性机制，保障微服务间调用的稳定性。

## 重试

通过 `ReqwestTransport::builder().retry(n)` 启用指数退避重试：

```rust
use rfeign::{ClientBuilder, ReqwestTransport};

let transport = ReqwestTransport::builder()
    .retry(3)  // 最多重试 3 次
    .build();

let client = ClientBuilder::new(transport)
    .base_url("https://api.example.com")
    .build();
```

重试策略基于指数退避算法，仅对瞬态错误（5xx、网络超时等）进行重试。

需要启用 feature：

```toml
rfeign = { version = "0.0.1", features = ["retry"] }
```

## 熔断器

当下游服务持续失败时，熔断器会快速失败避免雪崩：

```rust
use std::time::Duration;
use rfeign::circuit_breaker::CircuitBreakerMiddleware;
use rfeign::{ClientBuilder, ReqwestTransport};

let breaker = CircuitBreakerMiddleware::new(
    0.5,                          // 错误率阈值 50%
    Duration::from_secs(30),      // 熔断打开后等待 30 秒
);

let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("https://api.example.com")
    .middleware(breaker)
    .build();
```

也可使用默认错误率（50%）的快捷方法：

```rust
let breaker = CircuitBreakerMiddleware::default_with_wait(Duration::from_secs(30));
```

熔断器打开时，请求会立即返回 `Error::CircuitOpen` 错误。

需要启用 feature：

```toml
rfeign = { version = "0.0.1", features = ["circuit-breaker"] }
```

## 超时

### 方法级超时（命令式）

```rust
use std::time::Duration;

let resp = client.get("/slow")
    .timeout(Duration::from_secs(5))
    .send().await?;
```

### 宏级超时（声明式）

```rust
#[rfeign::get("/slow")]
#[timeout(5000)]  // 毫秒
async fn slow_request(&self) -> rfeign::Result<Data>;
```

### 全局超时（ClientBuilder）

```rust
use std::time::Duration;

let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("https://api.example.com")
    .connect_timeout(Duration::from_secs(5))
    .read_timeout(Duration::from_secs(30))
    .write_timeout(Duration::from_secs(10))
    .build();
```

超时触发时返回 `Error::Timeout`。

## 请求取消

使用 `CancellationToken` 可以在外部取消正在进行的请求：

```rust
use tokio_util::sync::CancellationToken;

let token = CancellationToken::new();
let cancel = token.clone();

// 在另一个任务中取消
tokio::spawn(async move {
    tokio::time::sleep(Duration::from_secs(2)).await;
    cancel.cancel();
});

let result = client.get("/long-running")
    .cancel_token(token)
    .send().await;

match result {
    Err(rfeign::Error::Cancelled) => println!("请求已取消"),
    _ => {}
}
```
