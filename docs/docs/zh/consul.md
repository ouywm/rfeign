# Consul 服务发现

rfeign 集成 Consul 注册中心，通过服务名自动发现健康实例。

## 启用

```toml
[dependencies]
rfeign = { version = "0.0.1", features = ["consul"] }
```

## ConsulResolver 配置

```rust
use rfeign::resolver_ext::ConsulResolver;

// 基本用法
let resolver = ConsulResolver::new("http://127.0.0.1:8500");

// 带 ACL Token
let resolver = ConsulResolver::with_token(
    "http://127.0.0.1:8500",
    "my-consul-token",
);

// 指定 scheme（默认 http）
let resolver = ConsulResolver::new("http://127.0.0.1:8500")
    .scheme("https");
```

| 方法 | 说明 | 默认值 |
|------|------|--------|
| `new(addr)` | Consul 地址 | - |
| `with_token(addr, token)` | 带 ACL Token 的 Consul 地址 | - |
| `scheme(scheme)` | 解析出的 URL 协议 | `"http"` |

## 完整示例

```rust
use rfeign::resolver_ext::ConsulResolver;
use rfeign::{ClientBuilder, ReqwestTransport};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Order {
    id: i64,
    amount: f64,
    status: String,
}

#[rfeign::http_client(service = "order-service", path = "/api")]
trait OrderApi {
    #[rfeign::get("/orders/{id}")]
    async fn get_order(&self, #[path] id: i64) -> rfeign::Result<Order>;
}

#[tokio::main]
async fn main() -> rfeign::Result<()> {
    // 构建 Consul 解析器
    let resolver = ConsulResolver::new("http://127.0.0.1:8500");

    // 构建客户端
    let client = ClientBuilder::new(ReqwestTransport::new())
        .url_resolver(resolver)
        .service_name("order-service")
        .build();

    // 使用声明式 API
    let api = OrderApiClient::new(client);
    let order = api.get_order(1001).await?;
    println!("{:?}", order);

    Ok(())
}
```

`ConsulResolver` 通过 Consul 的 Health API 获取服务的健康节点列表，选取第一个可用实例的地址和端口拼接为 URL。如果没有健康实例，返回 `Error::Resolve` 错误。
