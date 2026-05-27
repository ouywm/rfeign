# summer-rfeign

[summer-rs](https://github.com/mzdk100/spring-rs) plugin for [rfeign](https://crates.io/crates/rfeign).

Builds a global rfeign `Client` and registers it in IoC. Client structs are auto-injected as Service components.

## Usage

```rust
use rfeign;
use serde::Deserialize;

// 1. Declare your HTTP client
#[rfeign::http_client(service = "user-service")]
trait UserApi {
    #[rfeign::get("/users/{id}")]
    async fn get_user(&self, #[path] id: i64) -> rfeign::Result<User>;
}

// 2. Register as summer Service (one line)
summer_rfeign::register!(UserApiClient);

// 3. Inject anywhere — no manual new() needed
#[get("/")]
async fn handler(Component(api): Component<UserApiClient>) -> impl IntoResponse {
    let user = api.get_user(1).await.unwrap();
    Json(user)
}
```

## Config (app.toml)

```toml
[rfeign]
base_url = "http://localhost:8080"  # optional fallback
timeout_ms = 30000
```

With nacos, no `base_url` needed — service URLs are resolved automatically.

## Setup

```rust
App::new()
    .add_plugin(NacosPlugin)       // optional: for service discovery
    .add_plugin(RfeignPlugin)
    .add_plugin(WebPlugin)
    .run()
    .await;
```

## Features

| Feature | Description |
|---------|-------------|
| `nacos` | Auto-resolve service URLs from summer-nacos NamingService |

## License

MIT OR Apache-2.0
