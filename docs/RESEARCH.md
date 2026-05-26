# Rust 声明式 HTTP 客户端生态调研

> 调研日期：2026-05-26
> 背景：为 `summer-http` 设计提供参考，评估 Rust 生态中是否已有成熟的类 Feign 方案

---

## 一、调研范围

目标：寻找 Rust 中实现了**声明式 HTTP 客户端**的方案，即通过 trait + 宏/属性标注自动生成 HTTP 客户端实现，类似 Java 的 `@FeignClient` / `OpenFeign`。同时调研重试、熔断、链路追踪等周边能力的成熟度。

---

## 二、声明式 HTTP 客户端方案

### 2.1 feignhttp

| 项目 | 信息 |
|------|------|
| crates.io | https://crates.io/crates/feignhttp |
| GitHub | https://github.com/dxx/feignhttp |
| 最新版本 | 0.6.0 |
| 最后更新 | **2026-05-24（活跃）** |
| 总下载量 | ~33,000 |
| Star | ~60 |

**设计方式：函数级宏**（非 trait 级）

#### Cargo.toml

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
feignhttp = { version = "0.6", features = ["reqwest-json"] }
```

#### 基础用法

```rust
use feignhttp::get;

#[get("https://api.github.com")]
async fn github() -> feignhttp::Result<String> {}

#[tokio::main]
async fn main() {
    let result = github().await.unwrap();
    println!("{}", result);
}
```

#### Path / Query / Body / Header 参数

```rust
use feignhttp::{get, post};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Repo { name: String }

#[derive(Serialize)]
struct CreateIssue { title: String, body: String }

// Path 参数：{owner}/{repo} 对应 #[path] 标注的参数
#[get("https://api.github.com/repos/{owner}/{repo}/issues")]
async fn list_issues(
    #[path] owner: &str,
    #[path] repo: &str,
    #[query] page: u32,
    #[header("Authorization")] token: &str,
) -> feignhttp::Result<Vec<Repo>> {}

// POST JSON body
#[post("https://api.github.com/repos/{owner}/{repo}/issues")]
async fn create_issue(
    #[path] owner: &str,
    #[path] repo: &str,
    #[body] req: CreateIssue,
) -> feignhttp::Result<Repo> {}
```

#### 共享 base_url（#[feign] 宏）

```rust
use feignhttp::{feign, get, post};

#[feign(url = "https://api.github.com")]
struct GithubClient;

impl GithubClient {
    #[get("/repos/{owner}/{repo}")]
    pub async fn get_repo(
        &self,
        #[path] owner: &str,
        #[path] repo: &str,
    ) -> feignhttp::Result<String> {}
}
```

**优点：** 目前唯一活跃维护的声明式方案，API 简洁，支持 reqwest / isahc 后端切换。

**缺点：**
- **函数级宏，不是 trait**，无法注入、无法 mock、无法与 DI 框架集成
- 不支持服务发现、熔断、重试
- Star 极少（~60），社区规模小

**结论：** 勉强可用，但与 `summer-http` 目标差距很大。

---

### 2.2 pretend

| 项目 | 信息 |
|------|------|
| crates.io | https://crates.io/crates/pretend |
| GitHub | https://github.com/SfietKonstantin/pretend |
| 最新版本 | 0.4.0 |
| 最后更新 | **2023-02-22（停更）** |
| 总下载量 | ~28,000 |
| Star | ~31 |

**设计方式：trait 级宏**（最接近 summer-http 目标）

#### Cargo.toml

```toml
[dependencies]
pretend = "0.4.0"
pretend-reqwest = "0.2.2"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

#### 定义 trait

```rust
use pretend::{pretend, request, Json, Pretend, Result, Url};
use pretend_reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct User { id: i64, name: String }

#[derive(Serialize)]
struct CreateUser { name: String }

#[derive(Serialize)]
struct PageQuery { page: u32, size: u32 }

// #[pretend] 宏作用在 trait 上，自动生成实现
#[pretend]
trait UserService {
    // 路径参数：{id} 对应同名参数
    #[request(method = "GET", path = "/users/{id}")]
    async fn get_user(&self, id: i64) -> Result<Json<User>>;

    // 参数名为 json 时自动序列化为 JSON body
    #[request(method = "POST", path = "/users")]
    async fn create_user(&self, json: &CreateUser) -> Result<Json<User>>;

    // 参数名为 query 时自动序列化为查询字符串
    #[request(method = "GET", path = "/users")]
    async fn list_users(&self, query: &PageQuery) -> Result<Json<Vec<User>>>;

    // 返回 () 表示忽略响应体
    #[request(method = "DELETE", path = "/users/{id}")]
    async fn delete_user(&self, id: i64) -> Result<()>;
}
```

#### 使用

```rust
#[tokio::main]
async fn main() {
    let client = Client::default();
    let url = Url::parse("https://api.example.com").unwrap();
    // Pretend::for_client 将 trait 实现绑定到具体 HTTP 客户端
    let service = Pretend::for_client(client).with_url(url);

    let user = service.get_user(1).await.unwrap();
    println!("{}", user.value().name);
}
```

#### 自定义请求头（模板化）

```rust
#[pretend]
trait AuthService {
    // header 宏支持模板化，参数名直接替换
    #[request(method = "GET", path = "/{path}")]
    #[header(name = "Authorization", value = "Bearer {token}")]
    async fn get_with_auth(
        &self,
        path: &str,
        token: &str,
    ) -> Result<String>;
}
```

**优点：** 设计思路最接近 Feign，是真正的 trait 级声明，支持多 HTTP 后端（reqwest、isahc、awc）。

**缺点：**
- **已停止维护超过 2 年**，不可用于生产
- 31 star，社区几乎无人使用
- 不支持服务发现、熔断、重试、链路追踪

**结论：** 设计方向正确，但已死亡，不可用于生产。

---

### 2.3 feign-rs（feign crate）

| 项目 | 信息 |
|------|------|
| crates.io | https://crates.io/crates/feign |
| GitHub | https://github.com/niuhuan/feign-rs |
| 最新版本 | 0.3.2 |
| 最后更新 | 2025-09-04 |
| 总下载量 | ~13,490 |

文档极少，API 设计不清晰，社区几乎无人使用。**不推荐。**

---

## 三、中间件与周边能力方案

### 3.1 reqwest-middleware + reqwest-retry + reqwest-tracing

这三个 crate 由 TrueLayer 维护，是 Rust HTTP 客户端中间件的事实标准组合。

| 项目 | 版本 | 最后更新 | 总下载量 |
|------|------|---------|---------|
| reqwest-middleware | 0.5.2 | 2026-05-19（活跃） | ~60,000,000 |
| reqwest-retry | 0.9.1 | 2026-02-05（活跃） | ~37,600,000 |
| reqwest-tracing | 0.7.1 | 2026-05-19（活跃） | ~24,500,000 |

#### Cargo.toml

```toml
[dependencies]
reqwest = { version = "0.13", features = ["json"] }
reqwest-middleware = "0.5"
reqwest-retry = "0.9"
reqwest-tracing = "0.7"
tokio = { version = "1", features = ["full"] }
```

#### 完整用法：重试 + 链路追踪中间件

```rust
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use reqwest_tracing::TracingMiddleware;

async fn build_client() -> reqwest_middleware::ClientWithMiddleware {
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

    ClientBuilder::new(reqwest::Client::new())
        .with(TracingMiddleware::default())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build()
}
```

#### 自定义重试策略（按状态码）

```rust
use reqwest_retry::{RetryableStrategy, Retryable};
use reqwest_middleware::Error;

struct RetryOn503;

impl RetryableStrategy for RetryOn503 {
    fn handle(&self, res: &Result<reqwest::Response, Error>) -> Option<Retryable> {
        match res {
            Ok(r) if r.status() == 503 || r.status() == 429 => Some(Retryable::Transient),
            _ => None,
        }
    }
}

let policy = ExponentialBackoff::builder().build_with_max_retries(3);
let client = ClientBuilder::new(reqwest::Client::new())
    .with(RetryTransientMiddleware::new_with_policy_and_strategy(policy, RetryOn503))
    .build();
```

#### 自定义链路追踪 Span 名称

```rust
use reqwest_middleware::{ClientBuilder, Extension};
use reqwest_tracing::{TracingMiddleware, OtelName};

let client = ClientBuilder::new(reqwest::Client::new())
    .with_init(Extension(OtelName("my-service-client".into())))
    .with(TracingMiddleware::default())
    .build();

// 单个请求级别覆盖
let resp = client
    .post("https://api.example.com/payments")
    .with_extension(OtelName("POST /payments".into()))
    .send()
    .await
    .unwrap();
```

**结论：** `summer-http` 底层中间件层的最佳选择，但不是声明式客户端，不能替代 `#[http_client]` 宏层。

---

### 3.2 backon（独立重试库）

| 项目 | 信息 |
|------|------|
| crates.io | https://crates.io/crates/backon |
| GitHub | https://github.com/Xuanwo/backon |
| 最新版本 | 1.6.0 |
| 最后更新 | 2025-10-18（活跃） |
| 总下载量 | ~47,000,000 |

**定位：** 独立的重试库，不绑定任何 HTTP 客户端，通过闭包包装任意 async fn。

#### Cargo.toml

```toml
[dependencies]
backon = { version = "1", features = ["tokio-sleep"] }
```

#### 基础用法

```rust
use backon::{ExponentialBuilder, Retryable};

async fn fetch_user(id: i64) -> Result<User, reqwest::Error> {
    reqwest::get(format!("https://api.example.com/users/{id}"))
        .await?
        .json::<User>()
        .await
}

#[tokio::main]
async fn main() {
    let user = (|| fetch_user(1))
        .retry(ExponentialBuilder::default())
        .await
        .unwrap();
}
```

#### 三种退避策略

```rust
use backon::{ConstantBuilder, ExponentialBuilder, FibonacciBuilder};
use std::time::Duration;

// 固定间隔，最多重试 3 次
let constant = ConstantBuilder::default()
    .with_delay(Duration::from_secs(1))
    .with_max_times(3);

// 指数退避，带抖动
let exponential = ExponentialBuilder::default()
    .with_min_delay(Duration::from_millis(100))
    .with_max_delay(Duration::from_secs(10))
    .with_max_times(5)
    .with_jitter();

// 斐波那契退避
let fibonacci = FibonacciBuilder::default()
    .with_min_delay(Duration::from_millis(500))
    .with_max_times(4);
```

#### 条件重试 + 通知回调

```rust
let result = (|| async { reqwest::get("https://api.example.com").await })
    .retry(ExponentialBuilder::default())
    .when(|e| e.is_timeout() || e.is_connect())   // 只重试超时和连接错误
    .notify(|err, dur| {
        tracing::warn!("retrying after {:?}: {}", dur, err);
    })
    .await?;
```

**结论：** 下载量 4700 万，非常成熟。但它是独立库，不与 HTTP 框架集成，`summer-http` 可以在宏生成的代码里内部使用它。

---

### 3.3 failsafe（熔断器）

| 项目 | 信息 |
|------|------|
| crates.io | https://crates.io/crates/failsafe |
| GitHub | https://github.com/dmexe/failsafe-rs |
| 最新版本 | 1.3.0 |
| 最后更新 | 2024-07-05 |
| 总下载量 | ~15,000,000 |

**定位：** Rust 生态中下载量最高的熔断器实现，但 API 设计偏底层，无声明式支持。

#### Cargo.toml

```toml
[dependencies]
failsafe = { version = "1", features = ["futures-core"] }
```

#### 基础用法（同步）

```rust
use failsafe::{Config, CircuitBreaker, Error};

let circuit_breaker = Config::new().build();

match circuit_breaker.call(|| dangerous_call()) {
    Err(Error::Inner(e)) => eprintln!("inner error: {}", e),
    Err(Error::Rejected) => eprintln!("circuit open, request rejected"),
    Ok(result) => println!("success: {:?}", result),
}
```

#### 自定义配置（指数退避 + 连续失败策略）

```rust
use std::time::Duration;
use failsafe::{backoff, failure_policy, Config};

let backoff = backoff::exponential(
    Duration::from_secs(10),
    Duration::from_secs(60),
);
// 连续失败 3 次后打开熔断器
let policy = failure_policy::consecutive_failures(3, backoff);
let cb = Config::new().failure_policy(policy).build();
```

#### 异步用法

```rust
use failsafe::futures::CircuitBreaker;

let cb = Config::new().build();
let result = cb.call(async { fetch_data().await }).await;
```

**缺点：**
- 无声明式 API，需要手动包装每个调用
- 熔断器状态只在单进程内，不支持多实例共享（无 Redis 后端）
- 与 HTTP 框架完全割裂

**结论：** 有一定成熟度，但缺乏声明式支持，`summer-http` 需要在宏层封装它。

---

## 四、OpenAPI 代码生成方案

从 OpenAPI 规范生成客户端代码，与运行时宏方案思路不同。

| 方案 | GitHub | 维护状态 | 说明 |
|------|--------|---------|------|
| progenitor | oxidecomputer/progenitor | ✅ 活跃 | Oxide Computer 出品，质量最高 |
| libninja | kurtbuilds/libninja | ✅ 活跃 | 生成 Stripe 质量的 API 客户端 |
| openapi-generator | OpenAPITools/openapi-generator | ✅ 活跃 | 通用工具，支持 Rust 目标 |

#### progenitor 用法示例

```toml
# Cargo.toml
[build-dependencies]
progenitor = "0.14"

[dependencies]
progenitor-client = "0.14"
reqwest = { version = "0.13", features = ["json", "stream"] }
serde = { version = "1", features = ["derive"] }
```

```rust
// build.rs
use std::path::Path;
use progenitor::{GenerationSettings, Generator};

fn main() {
    let src = Path::new("openapi.json");
    let file = std::fs::File::open(src).unwrap();
    let spec = serde_json::from_reader(file).unwrap();

    let mut generator = Generator::new(GenerationSettings::default());
    let tokens = generator.generate_tokens(&spec).unwrap();

    let out = Path::new(&std::env::var("OUT_DIR").unwrap()).join("client.rs");
    std::fs::write(out, tokens.to_string()).unwrap();
}
```

```rust
// src/main.rs
include!(concat!(env!("OUT_DIR"), "/client.rs"));

#[tokio::main]
async fn main() {
    let client = Client::new("https://api.example.com");
    // 生成的方法直接调用，完全类型安全
    let user = client.get_user().id(1).send().await.unwrap();
}
```

**结论：** 适合对接有 OpenAPI 文档的第三方 API，不适合微服务内部调用（没有 OpenAPI 文档，且代码生成与框架割裂）。

---

## 五、各核心能力成熟度总览

| 能力 | 成熟度 | 代表方案 | 说明 |
|------|--------|---------|------|
| 声明式 HTTP 客户端（trait 级） | ❌ 空白 | 无 | pretend 已死，feignhttp 是函数级 |
| 命令式 HTTP 客户端 | ✅ 成熟 | reqwest | 事实标准，6000 万下载 |
| 中间件链 | ✅ 成熟 | reqwest-middleware | 6000 万下载，活跃维护 |
| 重试（独立库） | ✅ 成熟 | backon | 4700 万下载，支持多种退避策略 |
| 重试（HTTP 中间件） | ✅ 成熟 | reqwest-retry | 3700 万下载，与 reqwest 深度集成 |
| 熔断器 | ⚠️ 基本可用 | failsafe | 1500 万下载，但无声明式 API |
| 链路追踪透传 | ✅ 成熟 | reqwest-tracing | 2400 万下载，需手动配置 |
| 服务发现集成 | ❌ 空白 | 无 | Nacos/Consul SDK 有，但无框架级整合 |
| 框架级整体集成 | ❌ 空白 | 无 | 无任何方案将上述能力串联 |

---

## 六、与 summer-http 设计的对比

| summer-http 特性 | 生态现状 | 可复用方案 |
|----------------|---------|----------|
| `#[http_client]` trait 宏 | ❌ 空白 | 需自研，可参考 pretend 设计 |
| 命令式 `HttpClient` | ✅ 有 | 直接封装 reqwest |
| 重试声明式配置 | ❌ 无声明式 | 底层用 backon 或 reqwest-retry |
| 熔断器声明式配置 | ❌ 无声明式 | 底层用 failsafe |
| 链路追踪自动透传 | ⚠️ 需手动 | 底层用 reqwest-tracing |
| 服务发现集成 | ❌ 空白 | 需自研 |
| DI 容器注入 | ❌ 空白 | 需自研 |
| 配置集中管理 | ❌ 空白 | 需自研 |

---

## 七、结论

**Rust 生态目前没有一个达到生产可用标准的 trait 级声明式 HTTP 客户端。**

最接近目标的 `pretend` 设计方向正确，但已停更 2 年、社区规模极小。`feignhttp` 还在维护，但是函数级宏，无法与 DI 框架集成。

`summer-http` 的核心价值在于：

1. **声明式 `#[http_client]` trait 宏** — 生态真实空白，需自研
2. **框架级零配置集成** — 底层可复用 reqwest-middleware / reqwest-tracing / backon / failsafe，但串联层需自研
3. **与 DI 容器深度集成** — 生态空白，需自研

在国内 Java 转 Rust 的微服务场景中，这个方向有明确的需求和价值。

---

## 八、参考链接

- [feignhttp](https://crates.io/crates/feignhttp) / [GitHub](https://github.com/dxx/feignhttp)
- [pretend](https://crates.io/crates/pretend) / [GitHub](https://github.com/SfietKonstantin/pretend)
- [feign-rs](https://crates.io/crates/feign) / [GitHub](https://github.com/niuhuan/feign-rs)
- [reqwest-middleware](https://crates.io/crates/reqwest-middleware) / [GitHub](https://github.com/TrueLayer/reqwest-middleware)
- [reqwest-retry](https://crates.io/crates/reqwest-retry)
- [reqwest-tracing](https://crates.io/crates/reqwest-tracing)
- [backon](https://crates.io/crates/backon) / [GitHub](https://github.com/Xuanwo/backon)
- [failsafe](https://crates.io/crates/failsafe) / [GitHub](https://github.com/dmexe/failsafe-rs)
- [progenitor](https://crates.io/crates/progenitor) / [GitHub](https://github.com/oxidecomputer/progenitor)