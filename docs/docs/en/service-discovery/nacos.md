# Nacos Service Discovery

rfeign integrates with [Alibaba Nacos](https://nacos.io/) for service discovery.

## Setup

Enable the `nacos` feature:

```toml
[dependencies]
rfeign = { version = "0.0.1", features = ["nacos"] }
```

## NacosResolverBuilder

Configure the resolver with `NacosResolverBuilder`:

```rust
use rfeign::resolver_ext::NacosResolverBuilder;

let resolver = NacosResolverBuilder::new("127.0.0.1:8848")
    .namespace("public")
    .group("DEFAULT_GROUP")
    .scheme("http")
    .build()
    .await?;
```

### Builder Methods

| Method                    | Description                          |
|---------------------------|--------------------------------------|
| `new(server_addr)`        | Nacos server address                 |
| `namespace(ns)`           | Nacos namespace (default: public)    |
| `group(group)`            | Service group name                   |
| `clusters(vec)`           | Cluster filter list                  |
| `scheme(scheme)`          | URL scheme: "http" or "https"        |
| `auth(username, password)` | Nacos authentication credentials    |
| `build().await`           | Build the resolver (async)           |

## Full Example

```rust
use rfeign::resolver_ext::NacosResolverBuilder;
use rfeign::{ClientBuilder, ReqwestTransport};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct User {
    id: i64,
    name: String,
}

#[rfeign::http_client(service = "user-service", path = "/api")]
trait UserApi {
    #[rfeign::get("/users/{id}")]
    async fn get_user(&self, #[path] id: i64) -> rfeign::Result<User>;
}

#[tokio::main]
async fn main() -> rfeign::Result<()> {
    let resolver = NacosResolverBuilder::new("127.0.0.1:8848")
        .namespace("public")
        .group("DEFAULT_GROUP")
        .build()
        .await?;

    let client = ClientBuilder::new(ReqwestTransport::new())
        .url_resolver(resolver)
        .service_name("user-service")
        .build();

    let api = UserApiClient::new(client);
    let user = api.get_user(1).await?;
    println!("{:?}", user);

    Ok(())
}
```

## How It Works

`NacosResolver` calls Nacos naming service's `select_one_healthy_instance` on each request. It picks a random healthy instance and constructs the URL as `{scheme}://{ip}:{port}`.

For services with authentication:

```rust
let resolver = NacosResolverBuilder::new("127.0.0.1:8848")
    .namespace("production")
    .group("MY_GROUP")
    .clusters(vec!["cluster-a".into(), "cluster-b".into()])
    .auth("nacos", "nacos-password")
    .scheme("https")
    .build()
    .await?;
```
