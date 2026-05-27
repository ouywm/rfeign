# 声明式 API

rfeign 的核心能力是通过 trait 定义 HTTP 接口，宏自动生成实现代码。

## #[http_client] 属性

在 trait 上标注 `#[http_client]` 即可声明一个 HTTP 客户端接口：

```rust
#[rfeign::http_client(base_url = "http://localhost:8080", path = "/api/v1")]
trait UserApi {
    #[rfeign::get("/users/{id}")]
    async fn get_user(&self, #[path] id: i64) -> rfeign::Result<User>;
}
```

支持的属性参数：

| 参数 | 说明 | 示例 |
|------|------|------|
| `base_url` | 静态基础 URL | `base_url = "http://localhost:8080"` |
| `path` | 公共路径前缀，拼接在每个方法路径之前 | `path = "/api/v1"` |
| `service` | 服务名称，配合服务发现使用 | `service = "user_service"` |

## #[headers] trait 级别

在 trait 上使用 `#[headers(...)]` 为所有方法添加公共请求头：

```rust
#[rfeign::http_client(base_url = "http://localhost:8080")]
#[headers("Content-Type: application/json", "X-App: my-service")]
trait UserApi {
    #[rfeign::get("/users/{id}")]
    async fn get_user(&self, #[path] id: i64) -> rfeign::Result<User>;
}
```

所有方法发出的请求都会自动携带这些 header。

## HTTP 方法宏

在 trait 方法上使用对应的 HTTP 方法宏，指定请求路径：

```rust
#[rfeign::http_client(base_url = "http://localhost:8080")]
trait MyApi {
    #[rfeign::get("/resources")]
    async fn list(&self) -> rfeign::Result<Vec<Resource>>;

    #[rfeign::post("/resources")]
    async fn create(&self, #[body] data: CreateResource) -> rfeign::Result<Resource>;

    #[rfeign::put("/resources/{id}")]
    async fn update(&self, #[path] id: i64, #[body] data: UpdateResource) -> rfeign::Result<Resource>;

    #[rfeign::delete("/resources/{id}")]
    async fn delete(&self, #[path] id: i64) -> rfeign::Result<()>;

    #[rfeign::patch("/resources/{id}")]
    async fn patch(&self, #[path] id: i64, #[body] data: PatchResource) -> rfeign::Result<Resource>;

    #[rfeign::head("/resources/{id}")]
    async fn exists(&self, #[path] id: i64) -> rfeign::Result<()>;
}
```

支持的方法宏：`#[get]`、`#[post]`、`#[put]`、`#[delete]`、`#[patch]`、`#[head]`。

## 生成的 struct 和 new() 方法

宏会为 trait 生成一个名为 `{TraitName}Client` 的 struct，并实现该 trait：

```rust
#[rfeign::http_client(base_url = "http://localhost:8080")]
trait UserApi {
    #[rfeign::get("/users/{id}")]
    async fn get_user(&self, #[path] id: i64) -> rfeign::Result<User>;
}

// 宏自动生成：
// pub struct UserApiClient { client: Client }
// impl UserApiClient { pub fn new(client: Client) -> Self { ... } }
// impl UserApi for UserApiClient { ... }
```

使用方式：

```rust
use rfeign::{ClientBuilder, ReqwestTransport};

let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("http://localhost:8080")
    .build();

let api = UserApiClient::new(client);
let user = api.get_user(42).await?;
```

生成的 struct 还提供以下静态方法：

- `UserApiClient::base_url()` — 返回 `Option<&'static str>`
- `UserApiClient::service_name()` — 返回 `Option<&'static str>`
- `UserApiClient::path()` — 返回 `&'static str`
