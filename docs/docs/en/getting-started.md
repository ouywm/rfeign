# Getting Started

## Installation

```toml
[dependencies]
rfeign = "0.0.1"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

## Declarative Usage

```rust
use rfeign::{ClientBuilder, ReqwestTransport};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct User { id: i64, name: String }

#[rfeign::http_client(base_url = "http://localhost:8080")]
trait UserApi {
    #[rfeign::get("/users/{id}")]
    async fn get_user(&self, #[path] id: i64) -> rfeign::Result<User>;

    #[rfeign::get("/users")]
    async fn list_users(&self, #[query] page: u32, #[query] size: u32)
        -> rfeign::Result<Vec<User>>;

    #[rfeign::post("/users")]
    async fn create_user(&self, #[body] user: CreateUser)
        -> rfeign::Result<User>;
}

#[tokio::main]
async fn main() {
    let client = ClientBuilder::new(ReqwestTransport::new())
        .base_url("http://localhost:8080")
        .build();
    let api = UserApiClient::new(client);
    let user = api.get_user(1).await.unwrap();
    println!("{:?}", user);
}
```

## Imperative Usage

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
