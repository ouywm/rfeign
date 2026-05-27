# Nacos 服务发现

rfeign 集成 Nacos 注册中心，通过服务名自动发现健康实例。

## 启用

```toml
[dependencies]
rfeign = { version = "0.0.1", features = ["nacos"] }
```

## NacosResolverBuilder 配置

```rust
use rfeign::resolver_ext::NacosResolverBuilder;

let resolver = NacosResolverBuilder::new("127.0.0.1:8848")
    .namespace("public")           // 命名空间（可选）
    .group("DEFAULT_GROUP")        // 服务分组（可选）
    .clusters(vec!["cluster1".into()])  // 集群过滤（可选）
    .scheme("http")                // URL scheme，默认 http
    .auth("nacos", "nacos")        // 认证（可选）
    .build()
    .await?;
```

| 方法 | 说明 | 默认值 |
|------|------|--------|
| `new(server_addr)` | Nacos 服务器地址 | - |
| `namespace(ns)` | 命名空间 | 无 |
| `group(group)` | 服务分组 | 无 |
| `clusters(vec)` | 集群列表 | 空 |
| `scheme(scheme)` | URL 协议 | `"http"` |
| `auth(user, pass)` | Nacos 认证 | 无 |

## 完整示例

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
    // 构建 Nacos 解析器
    let resolver = NacosResolverBuilder::new("127.0.0.1:8848")
        .namespace("public")
        .group("DEFAULT_GROUP")
        .build()
        .await?;

    // 构建客户端
    let client = ClientBuilder::new(ReqwestTransport::new())
        .url_resolver(resolver)
        .service_name("user-service")
        .build();

    // 使用声明式 API
    let api = UserApiClient::new(client);
    let user = api.get_user(1).await?;
    println!("{:?}", user);

    Ok(())
}
```

每次调用 `get_user()` 时，rfeign 会从 Nacos 获取 `user-service` 的一个健康实例地址，拼接为完整 URL 后发起请求。
