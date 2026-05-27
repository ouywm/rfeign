# 命令式 API

除了声明式 trait，rfeign 也提供命令式的链式调用 API，适合动态构建请求的场景。

## ClientBuilder 配置

```rust
use rfeign::{ClientBuilder, ReqwestTransport, LogLevel};

let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("https://api.example.com")
    .log_level(LogLevel::Basic)
    .build();
```

## HTTP 方法

Client 提供 `get`、`post`、`put`、`delete`、`patch`、`head` 方法，返回 `RequestBuilder`：

```rust
let resp = client.get("/users/1").send().await?;
let resp = client.post("/users").json(&new_user)?.send().await?;
let resp = client.put("/users/1").json(&update)?.send().await?;
let resp = client.delete("/users/1").send().await?;
```

## 请求构建方法

### .header()

添加单个请求头：

```rust
client.get("/data")
    .header("X-Request-Id", "abc-123")
    .header("Accept", "application/json")
    .send().await?;
```

### .query_pair() / .query()

添加查询参数：

```rust
// 单个键值对
client.get("/search")
    .query_pair("q", "rust")
    .query_pair("page", "1")
    .send().await?;

// 通过可序列化结构体
#[derive(Serialize)]
struct Params { page: u32, size: u32 }

client.get("/users")
    .query(&Params { page: 1, size: 10 })
    .send().await?;
```

### .json()

设置 JSON 请求体（自动添加 Content-Type）：

```rust
#[derive(Serialize)]
struct CreateUser { name: String }

client.post("/users")
    .json(&CreateUser { name: "alice".into() })?
    .send().await?;
```

### .timeout()

设置单次请求超时：

```rust
use std::time::Duration;

client.get("/slow-endpoint")
    .timeout(Duration::from_secs(5))
    .send().await?;
```

## 响应处理

`.send().await?` 返回 `RawResponse`，提供以下方法：

```rust
let resp = client.get("/users/1").send().await?;

// 获取状态码
println!("status: {}", resp.status());

// 反序列化为 JSON
let user: User = resp.json()?;

// 或获取原始文本
let text = resp.text()?;

// 或获取原始字节
let bytes = resp.bytes();
```

也可以使用 `.api_response()` 同时获取状态码、headers 和 body：

```rust
let api_resp = client.get("/users/1").send().await?.api_response::<User>()?;
println!("status: {}, body: {:?}", api_resp.status, api_resp.body);
```

## .to_curl() 调试

在发送前调用 `.to_curl()` 可生成等效的 cURL 命令，方便调试：

```rust
let builder = client.post("/users")
    .header("X-App", "test")
    .query_pair("verbose", "true")
    .json(&CreateUser { name: "bob".into() })?;

println!("{}", builder.to_curl());
// 输出类似：
// curl -X POST \
//   -H 'content-type: application/json' \
//   -H 'x-app: test' \
//   '<base_url>/users?verbose=true' \
//   -d '{"name":"bob"}'
```
