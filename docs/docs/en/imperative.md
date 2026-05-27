# Imperative API

The imperative API gives you a fluent request builder for cases where you want full control without defining a trait.

## ClientBuilder Configuration

```rust
use std::time::Duration;
use rfeign::{ClientBuilder, ReqwestTransport, LogLevel};

let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("https://api.example.com")
    .connect_timeout(Duration::from_secs(5))
    .read_timeout(Duration::from_secs(30))
    .log_level(LogLevel::Basic)
    .build();
```

## HTTP Methods

The client exposes `.get()`, `.post()`, `.put()`, `.delete()`, `.patch()`, and `.head()`:

```rust
let resp = client.get("/users/1").send().await?;
let resp = client.post("/users").json(&new_user)?.send().await?;
let resp = client.put("/users/1").json(&updated)?.send().await?;
let resp = client.delete("/users/1").send().await?;
```

## Request Builder Methods

### Headers

```rust
let resp = client.get("/protected")
    .header("Authorization", "Bearer my-token")
    .header("X-Request-Id", "abc-123")
    .send()
    .await?;
```

### Query Parameters

```rust
// Single key-value pair
let resp = client.get("/search")
    .query_pair("q", "rust http client")
    .query_pair("page", "1")
    .send()
    .await?;

// From a serializable struct
#[derive(Serialize)]
struct Params { page: u32, size: u32 }

let resp = client.get("/users")
    .query(&Params { page: 1, size: 20 })
    .send()
    .await?;
```

### JSON Body

```rust
#[derive(Serialize)]
struct CreateUser { name: String, email: String }

let user: User = client.post("/users")
    .json(&CreateUser { name: "Alice".into(), email: "alice@example.com".into() })?
    .send()
    .await?
    .json()?;
```

### Timeout

```rust
use std::time::Duration;

let resp = client.get("/slow-endpoint")
    .timeout(Duration::from_secs(10))
    .send()
    .await?;
```

## Response Handling

The `.send().await?` call returns a `RawResponse` with several consumption methods:

```rust
// Deserialize JSON response
let user: User = client.get("/users/1").send().await?.json()?;

// Get raw text
let html = client.get("/page").send().await?.text()?;

// Get raw bytes
let data = client.get("/file").send().await?.bytes();

// Check status only
let resp = client.get("/health").send().await?;
println!("status: {}", resp.status());
println!("success: {}", resp.is_success());

// Ensure success or return error
client.delete("/users/1").send().await?.ensure_success()?;
```

## Debugging with .to_curl()

Generate a cURL command from any request builder for debugging:

```rust
let req = client.post("/users")
    .header("Content-Type", "application/json")
    .json(&CreateUser { name: "Alice".into() })?;

println!("{}", req.to_curl());
// Output:
// curl -X POST \
//   -H 'content-type: application/json' \
//   '<base_url>/users' \
//   -d '{"name":"Alice"}'
```

## Full Example

```rust
use std::time::Duration;
use rfeign::{ClientBuilder, ReqwestTransport};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct User { id: i64, name: String }

#[derive(Serialize)]
struct CreateUser { name: String }

#[tokio::main]
async fn main() -> rfeign::Result<()> {
    let client = ClientBuilder::new(ReqwestTransport::new())
        .base_url("https://httpbin.org")
        .build();

    // GET with query params
    let resp = client.get("/get")
        .query_pair("page", "1")
        .header("X-App", "rfeign")
        .send()
        .await?;
    println!("status: {}", resp.status());

    // POST with JSON body
    let resp = client.post("/post")
        .json(&CreateUser { name: "alice".into() })?
        .timeout(Duration::from_secs(5))
        .send()
        .await?;
    println!("status: {}", resp.status());

    Ok(())
}
```
