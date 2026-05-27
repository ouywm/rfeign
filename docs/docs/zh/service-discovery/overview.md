# 服务发现概述

rfeign 通过 `UrlResolver` trait 抽象服务发现，支持静态 URL 和动态服务注册中心。

## UrlResolver trait

```rust
#[async_trait]
pub trait UrlResolver: Send + Sync + 'static {
    async fn resolve(&self, service_name: &str) -> Result<String>;
}
```

每次请求时，rfeign 调用 `resolve()` 获取目标服务的基础 URL。

## StaticUrl

最简单的实现，直接返回固定 URL：

```rust
use rfeign::{ClientBuilder, ReqwestTransport};

let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("http://localhost:8080")  // 内部使用 StaticUrl
    .build();
```

## 动态服务发现

配合服务注册中心（Nacos、Consul 等），通过服务名动态解析地址：

```rust
use rfeign::{ClientBuilder, ReqwestTransport};

let resolver = /* NacosResolver 或 ConsulResolver */;

let client = ClientBuilder::new(ReqwestTransport::new())
    .url_resolver(resolver)
    .service_name("user-service")
    .build();

// 每次请求时自动从注册中心获取健康实例地址
let resp = client.get("/users/1").send().await?;
```

## ClientBuilder::service_name()

`service_name()` 设置的值会传递给 `UrlResolver::resolve()` 方法：

```rust
let client = ClientBuilder::new(ReqwestTransport::new())
    .url_resolver(my_resolver)
    .service_name("order-service")  // 传给 resolve("order-service")
    .build();
```

在声明式 API 中，通过 `#[http_client(service = "...")]` 指定：

```rust
#[rfeign::http_client(service = "user-service", path = "/api")]
trait UserApi {
    #[rfeign::get("/users/{id}")]
    async fn get_user(&self, #[path] id: i64) -> rfeign::Result<User>;
}
```

## 自定义 UrlResolver

实现 `UrlResolver` trait 即可接入任意服务发现系统：

```rust
use rfeign::resolver::UrlResolver;
use async_trait::async_trait;

struct MyResolver { /* ... */ }

#[async_trait]
impl UrlResolver for MyResolver {
    async fn resolve(&self, service_name: &str) -> rfeign::Result<String> {
        // 从自定义注册中心查询
        Ok(format!("http://{}.internal:8080", service_name))
    }
}
```
