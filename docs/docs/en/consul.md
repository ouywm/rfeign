# Consul Service Discovery

rfeign integrates with [HashiCorp Consul](https://www.consul.io/) for service discovery.

## Setup

Enable the `consul` feature:

```toml
[dependencies]
rfeign = { version = "0.0.1", features = ["consul"] }
```

## ConsulResolver

Create a resolver pointing to your Consul agent:

```rust
use rfeign::resolver_ext::ConsulResolver;

// Without ACL token
let resolver = ConsulResolver::new("http://127.0.0.1:8500");

// With ACL token
let resolver = ConsulResolver::with_token(
    "http://127.0.0.1:8500",
    "my-consul-token",
);

// With HTTPS scheme for resolved URLs
let resolver = ConsulResolver::new("http://127.0.0.1:8500")
    .scheme("https");
```

### Methods

| Method                        | Description                          |
|-------------------------------|--------------------------------------|
| `new(addr)`                   | Consul agent HTTP address            |
| `with_token(addr, token)`     | With ACL token authentication        |
| `scheme(s)`                   | URL scheme for resolved services     |

## Full Example

```rust
use rfeign::resolver_ext::ConsulResolver;
use rfeign::{ClientBuilder, ReqwestTransport};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Order {
    id: i64,
    status: String,
}

#[rfeign::http_client(service = "order-service", path = "/api")]
trait OrderApi {
    #[rfeign::get("/orders/{id}")]
    async fn get_order(&self, #[path] id: i64) -> rfeign::Result<Order>;
}

#[tokio::main]
async fn main() -> rfeign::Result<()> {
    let resolver = ConsulResolver::new("http://127.0.0.1:8500")
        .scheme("http");

    let client = ClientBuilder::new(ReqwestTransport::new())
        .url_resolver(resolver)
        .service_name("order-service")
        .build();

    let api = OrderApiClient::new(client);
    let order = api.get_order(42).await?;
    println!("{:?}", order);

    Ok(())
}
```

## How It Works

`ConsulResolver` queries Consul's service catalog via `get_service_nodes`. It picks the first healthy node and constructs the URL as `{scheme}://{address}:{port}`.

If no healthy instances are found, the request fails with `Error::Resolve`.

## With ACL Token

For Consul clusters with ACL enabled:

```rust
let resolver = ConsulResolver::with_token(
    "http://consul.internal:8500",
    "s3cr3t-token",
).scheme("https");

let client = ClientBuilder::new(ReqwestTransport::new())
    .url_resolver(resolver)
    .service_name("payment-service")
    .build();
```
