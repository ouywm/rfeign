# Service Discovery Overview

rfeign supports dynamic service discovery through the `UrlResolver` trait, allowing you to call services by name instead of hardcoded URLs.

## UrlResolver Trait

```rust
#[async_trait]
pub trait UrlResolver: Send + Sync + 'static {
    async fn resolve(&self, service_name: &str) -> Result<String>;
}
```

Any type implementing this trait can be plugged into the client to resolve service names to base URLs at request time.

## StaticUrl (Default)

When you use `.base_url("...")`, rfeign uses `StaticUrl` internally:

```rust
use rfeign::{ClientBuilder, ReqwestTransport};

let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("http://localhost:8080")
    .build();
```

This always resolves to the same URL regardless of the service name.

## Dynamic Discovery

For dynamic environments, use `.url_resolver()` with `.service_name()`:

```rust
use rfeign::{ClientBuilder, ReqwestTransport};

let client = ClientBuilder::new(ReqwestTransport::new())
    .url_resolver(my_resolver)
    .service_name("user-service")
    .build();
```

Each request will call `resolver.resolve("user-service")` to get the current base URL.

## With Declarative API

Use the `service` attribute in `#[http_client]`:

```rust
#[rfeign::http_client(service = "user-service", path = "/api")]
trait UserApi {
    #[rfeign::get("/users/{id}")]
    async fn get_user(&self, #[path] id: i64) -> rfeign::Result<User>;
}

// Build client with a resolver
let client = ClientBuilder::new(ReqwestTransport::new())
    .url_resolver(nacos_resolver)
    .service_name("user-service")
    .build();

let api = UserApiClient::new(client);
let user = api.get_user(1).await?;
```

## Built-in Resolvers

| Resolver         | Feature  | Description                    |
|------------------|----------|--------------------------------|
| `StaticUrl`      | (core)   | Fixed URL, no discovery        |
| `NacosResolver`  | `nacos`  | Alibaba Nacos naming service   |
| `ConsulResolver` | `consul` | HashiCorp Consul catalog       |

## Custom Resolver

Implement `UrlResolver` for any service registry:

```rust
use rfeign::resolver::UrlResolver;
use rfeign::error::Result;
use async_trait::async_trait;

struct MyResolver { /* ... */ }

#[async_trait]
impl UrlResolver for MyResolver {
    async fn resolve(&self, service_name: &str) -> Result<String> {
        // Look up service_name in your registry
        Ok(format!("http://{}:8080", service_name))
    }
}
```
