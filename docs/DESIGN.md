# 声明式 HTTP 客户端库 — 设计文档

> 库名确定为 **`feignx`**（致敬 OpenFeign，"x" 表示进化版）。

## 定位

一个**独立的、框架无关**的声明式 HTTP 客户端库。

- 不绑定任何 Web 框架（axum / rocket / actix-web / salvo 都可以用）
- 不绑定任何应用框架（summer-rs / loco / poem 都可以集成）
- 上游框架通过各自的**薄插件层**集成本库

类比其他语言：
- Java：`OpenFeign`（独立库）+ `spring-cloud-openfeign`（Spring 插件）
- C#：`Refit`（独立库）+ `Refit.HttpClientFactory`（.NET DI 集成包）
- Go：`go-resty`（独立库）+ 各框架自行封装

## 跨语言调研总结

| 语言 | 库 | 核心传输抽象 | 中间件模型 | 框架集成方式 |
|------|-----|------------|-----------|-------------|
| Java | OpenFeign | `Client.execute(Request) → Response` | `RequestInterceptor` 链 + Client 装饰器 | 独立包（spring-cloud-openfeign） |
| C# | Refit | `HttpClient`（.NET BCL） | `DelegatingHandler` 管道 | 独立包（Refit.HttpClientFactory） |
| Go | go-resty | `Client` 结构体 | `OnBeforeRequest` / `OnAfterResponse` 函数切片 | 无需，纯客户端 |
| JS | Axios | `XMLHttpRequest` / `http` | `interceptors.request/response` Promise 链 | 无需，纯客户端 |
| Python | httpx | `BaseTransport.handle_request()` | Transport 装饰器 | `app=` 参数注入 ASGI app |

## Rust 生态现有方案

### dxx/feignhttp（https://github.com/dxx/feignhttp）

- 活跃维护，当前版本 0.6，60 stars
- 宏驱动，`#[get]` / `#[post]` 属性宏应用于独立函数或 trait 方法
- 通过 trait 抽象多后端（reqwest / isahc），不定义自己的 Request/Response
- 返回类型自动决定反序列化方式（`String` → text, `T` → json）
- **局限**：无中间件、无拦截器、无服务发现、无框架集成

### niuhuan/feign-rs（https://github.com/niuhuan/feign-rs）

- Workspace：`feignx`（运行时）+ `feignx-macros`（过程宏）
- `#[client(host = "...", path = "/base")]` 标注 trait → 生成同名 struct
- `#[derive(Args)]` 将多参数打包为结构体（path + query + body + headers）
- `before_send` 钩子：单一拦截器函数
- `Host` trait：动态地址 + 内置 `HostRound` 轮询负载均衡
- 泛型 `State`：在拦截器中访问共享状态
- **局限**：无中间件链、无重试/熔断/超时、无 trace 透传、每次请求新建 Client

**值得参考的设计**：
1. `#[derive(Args)]` — 复杂请求参数打包为结构体
2. `Host` trait — 动态地址解析（类似我们的 `UrlResolver`）
3. `State` 泛型 — 拦截器访问共享状态（auth token 等）
4. `client_builder` — 用户自定义底层 Client 构建

### 社区结论

三个声明式库各有不足：
- dxx/feignhttp：函数级宏，无法注入、无法 mock、无法与 DI 集成
- niuhuan/feign-rs：有动态 Host 和 State，但只有单一拦截点
- SfietKonstantin/pretend：**设计最接近目标**（trait 级宏 + 多后端），但已停更 2 年

**中间件层已有成熟方案**（不需要自研）：
- `reqwest-middleware`（6000 万下载）— 中间件链框架
- `reqwest-retry`（3700 万下载）— 重试中间件
- `reqwest-tracing`（2400 万下载）— 链路追踪
- `backon`（4700 万下载）— 独立重试库
- `failsafe`（1500 万下载）— 熔断器

**我们的差异化**：
1. **trait 级声明式宏**（生态真实空白）
2. **框架级零配置集成**（底层复用成熟 crate，串联层自研）
3. **服务发现 + DI 集成**（生态空白）

### 从调研中提取的设计原则

1. **核心传输抽象只有一个方法**：`async fn call(Request) -> Result<Response>`
2. **中间件是装饰器/责任链**，不是 callback 数组
3. **编解码可插拔**，独立子包
4. **框架集成是独立包**，核心库零框架依赖
5. **声明式生成**：Rust 用 proc macro（等价于 Java 动态代理 / C# Source Generator）

## 架构总览

```
┌─────────────────────────────────────────────────────────┐
│                    用户代码层                             │
│  #[http_client] trait 定义 / 命令式 HttpClient            │
├─────────────────────────────────────────────────────────┤
│              本库自研：宏 + 服务解析 + 组装                │
│  proc macro 代码生成 / UrlResolver / ClientBuilder       │
├─────────────────────────────────────────────────────────┤
│              复用成熟生态（不自研）                        │
│  reqwest-middleware → reqwest-retry → reqwest-tracing    │
│  backon（重试策略） / failsafe（熔断器）                  │
├─────────────────────────────────────────────────────────┤
│                  服务解析层（自研）                        │
│  StaticUrl / NacosUrl / ConsulUrl                        │
├─────────────────────────────────────────────────────────┤
│                  传输层（复用）                           │
│  reqwest（连接池、TLS、HTTP/2）                           │
└─────────────────────────────────────────────────────────┘
```

**核心原则：只自研生态空白的部分，成熟的直接复用。**

| 层 | 自研 or 复用 | 具体方案 |
|----|-------------|---------|
| 声明式宏 | **自研** | `#[http_client]` proc macro（参考 pretend） |
| 服务发现 | **自研** | `UrlResolver` trait（包一层 nacos-sdk 等现有 SDK） |
| 中间件链 | 复用 | `reqwest-middleware`（6000 万下载） |
| 重试 | 复用 | `reqwest-retry` / `backon` |
| 熔断 | 复用 | `failsafe`（1500 万下载） |
| 链路追踪 | 复用 | `reqwest-tracing`（2400 万下载） |
| HTTP 传输 | 复用 | `reqwest` |

### 与 pretend 的关系

pretend（已停更）的设计最接近我们目标，可参考其：
- trait 级宏解析方式
- `Pretend::for_client(client).with_url(url)` 组装模式
- 参数名约定（`json` → body, `query` → query string）

pretend 缺少的（我们要补的）：服务发现、中间件集成、DI 注入、配置管理。

## Crate 结构（Workspace）

```
feignx/                              ← workspace root
├── Cargo.toml                      ← workspace 定义
│
├── feignx-core/                     ← 核心抽象（最小依赖）
│   └── src/
│       ├── lib.rs
│       ├── transport.rs            ← Transport trait
│       ├── resolver.rs             ← UrlResolver trait + StaticUrl
│       ├── middleware.rs           ← Middleware trait + Next
│       ├── interceptor.rs          ← RequestInterceptor + ResponseInterceptor
│       ├── auth.rs                 ← Auth trait + BearerAuth / BasicAuth / DynamicAuth
│       ├── codec.rs                ← Encoder / Decoder trait + JsonCodec
│       ├── error.rs                ← Error 枚举 + Result 类型别名
│       ├── error_decoder.rs        ← ErrorDecoder trait + DefaultErrorDecoder
│       ├── response.rs             ← ApiResponse<T>
│       ├── timeout.rs              ← Timeout 结构体（connect/read/write）
│       ├── part.rs                 ← Part（文件上传）
│       ├── stream.rs               ← ByteStream（流式响应）
│       ├── log.rs                  ← LogLevel 枚举
│       └── client.rs               ← Client + ClientBuilder
│
├── feignx-macros/                   ← proc macro
│   └── src/
│       ├── lib.rs
│       ├── client.rs               ← #[http_client] 宏实现
│       ├── method.rs               ← #[get/post/put/delete/patch/head]
│       ├── param.rs                ← #[path] / #[query] / #[body] / #[header] / #[headers]
│       ├── request_param.rs        ← #[derive(RequestParam)]
│       └── multipart.rs            ← #[multipart] + #[part]
│
├── feignx-reqwest/                  ← reqwest Transport 实现
│   └── src/
│       ├── lib.rs                  ← ReqwestTransport
│       └── curl.rs                 ← cURL 命令生成
│
├── feignx/                          ← 伞 crate（re-export + 可选功能）
│   └── src/
│       ├── lib.rs                  ← re-export core + macros + reqwest
│       ├── middleware.rs           ← 封装 reqwest-retry / failsafe / reqwest-tracing
│       └── resolver/               ← 服务发现（feature 门控）
│           ├── mod.rs              ← pub use + #[cfg] 门控
│           ├── nacos.rs            ← #[cfg(feature = "nacos")] NacosUrl
│           └── consul.rs           ← #[cfg(feature = "consul")] ConsulUrl
│
└── examples/
    ├── basic/                      ← 最简示例（声明式）
    ├── imperative/                 ← 命令式 Client 示例
    ├── with-retry/                 ← 带重试
    ├── with-nacos/                 ← 服务发现
    └── file-upload/                ← 文件上传
```

### 各 crate 职责

| Crate | 职责 | 核心依赖 |
|-------|------|---------|
| `feignx-core` | 所有 trait 定义 + 核心类型 | http, bytes, async-trait, thiserror, serde |
| `feignx-macros` | proc macro 代码生成 | syn, quote, proc-macro2, darling |
| `feignx-reqwest` | reqwest Transport + cURL 生成 | feignx-core, reqwest |
| `feignx` | 伞 crate + 中间件 + 服务发现 | 以上所有 + reqwest-middleware 等 |

服务发现放在伞 crate 内，通过 feature 门控：
- `features = ["nacos"]` → 编译 `resolver/nacos.rs`，引入 `nacos-sdk` 依赖
- `features = ["consul"]` → 编译 `resolver/consul.rs`，引入 consul 依赖
- 不启用时零额外依赖、零编译开销

### 为什么去掉了 `feignx-middleware`？

之前设计有一个独立的 `feignx-middleware` crate。现在去掉了，因为：

1. 重试 → 直接用 `reqwest-retry`（3700 万下载）
2. 熔断 → 直接用 `failsafe`（1500 万下载）
3. 链路追踪 → 直接用 `reqwest-tracing`（2400 万下载）

这些不需要我们再封装一层 crate。在伞 crate `feignx/src/middleware.rs` 中做薄封装即可：

```rust
// feignx/src/middleware.rs
// 只是把第三方中间件适配到我们的 ClientBuilder API

#[cfg(feature = "retry")]
pub fn retry_middleware(max_attempts: u32) -> impl reqwest_middleware::Middleware {
    use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
    let policy = ExponentialBackoff::builder().build_with_max_retries(max_attempts);
    RetryTransientMiddleware::new_with_policy(policy)
}

#[cfg(feature = "circuit-breaker")]
pub fn circuit_breaker(failure_threshold: u32, timeout: Duration) -> CircuitBreakerMiddleware {
    // 封装 failsafe
}
```

### Feature Flags（条件编译）

```toml
# feignx/Cargo.toml（伞 crate）
[features]
default = ["reqwest", "json"]

# 传输层后端
reqwest = ["dep:feignx-reqwest"]

# 编解码
json = ["dep:serde_json"]

# 中间件（全部可选，不是每个人都需要）
retry = ["dep:reqwest-retry", "dep:backon"]
circuit-breaker = ["dep:failsafe"]
tracing = ["dep:reqwest-tracing"]
timeout = []  # reqwest 自带 timeout，这里只是声明式配置
middleware = ["dep:reqwest-middleware"]  # 中间件链基础
middleware-full = ["middleware", "retry", "circuit-breaker", "tracing", "timeout"]

# 服务发现（feature 门控，代码在伞 crate 内）
nacos = ["dep:nacos-sdk"]
consul = ["dep:consul-api"]
```

**用户按需引入**：

```toml
# 最简使用（只要声明式宏 + reqwest）
feignx = "0.1"

# 需要重试
feignx = { version = "0.1", features = ["retry"] }

# 需要全部中间件
feignx = { version = "0.1", features = ["middleware-full"] }

# 需要 Nacos 服务发现
feignx = { version = "0.1", features = ["nacos"] }
```

**零 feature 时的最小依赖**：
- `feignx-core`（核心 trait）
- `feignx-macros`（proc macro）
- `reqwest`（HTTP 传输）
- `serde` + `serde_json`（编解码）

不启用 `retry` / `circuit-breaker` / `tracing` 时，这些 crate 完全不会被编译。

### 宏对 service 参数的条件处理

`#[http_client]` 宏支持两种模式：

```rust
// 模式 1：静态 URL（无需注册中心）
#[http_client(base_url = "http://user-service:8080")]
trait UserApi { ... }

// 模式 2：服务名（需要 UrlResolver）
#[http_client(service = "user-service")]
trait UserApi { ... }
```

宏生成的代码根据参数不同：

```rust
// 模式 1 生成：
impl UserApiClient {
    pub fn new(client: Client) -> Self {
        // client 内部使用 StaticUrl
        Self { client }
    }
}

// 模式 2 生成：
impl UserApiClient {
    pub fn new(client: Client) -> Self {
        // client 内部使用用户注入的 UrlResolver
        // 每次请求前调用 resolver.resolve("user-service") 获取地址
        Self { client }
    }
}
```

**关键**：宏本身不关心用的是哪个 UrlResolver 实现，
它只生成调用 `client.resolve_url()` 的代码。
具体用 Nacos 还是 Consul 还是静态 URL，是在 `ClientBuilder` 构建时决定的：

```rust
// 无注册中心（默认）
let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("http://user-service:8080")  // StaticUrl
    .build();

// 有 Nacos（启用 nacos feature，底层用 nacos-sdk）
#[cfg(feature = "nacos")]
let client = ClientBuilder::new(ReqwestTransport::new())
    .url_resolver(NacosUrl::new(naming_service))
    .build();

// 有 Consul（启用 consul feature）
#[cfg(feature = "consul")]
let client = ClientBuilder::new(ReqwestTransport::new())
    .url_resolver(ConsulUrl::new(consul_client))
    .build();
```

### 框架集成（不在本 workspace 内）

```
summer-rs/summer-feignx/         ← summer-rs 仓库中的薄插件
├── Cargo.toml                  ← 依赖 feignx + summer
└── src/lib.rs                  ← FeignPlugin，读取配置，注册 Component
```

其他框架同理：
- `axum-feignx` — 提供 axum 的 Extension/State 集成
- `actix-feignx` — 提供 actix-web 的 Data 集成
- `salvo-feignx` — 提供 salvo 的 Depot 集成

这些集成包只做一件事：**把 feignx 客户端实例注入到框架的 DI/状态管理中**。

## 核心抽象设计

### 设计原则

1. **复用 `http` crate 的标准类型**：不自定义 Request/Response，用 `http::Request<B>` / `http::Response<Bytes>`，
   这是 Rust 生态的通用语言（axum、hyper、reqwest、tower 都用它）
2. **Transport trait 只做传输**：`send(http::Request) -> http::Response`
3. **UrlResolver 预留服务发现扩展**：类似 OpenFeign 的 `Target`
4. **中间件是装饰器**：包裹 Transport，不是 callback 数组

### Transport（传输层）

```rust
// feignx-core/src/transport.rs
use http::{Request, Response};
use bytes::Bytes;

/// 核心传输抽象
/// 类似 OpenFeign 的 Client 接口、tower 的 Service trait
#[async_trait]
pub trait Transport: Send + Sync + 'static {
    async fn send(&self, request: Request<Bytes>) -> Result<Response<Bytes>, Error>;
}
```

**为什么用 `http::Request<Bytes>` 而不是自定义类型？**
- `http` crate 是 Rust HTTP 生态的标准（hyper/axum/reqwest 共用）
- 与 tower 中间件生态天然兼容
- 用户已经熟悉这些类型
- 零额外抽象成本

### UrlResolver（服务发现 — 统一 trait）

```rust
// feignx-core/src/resolver.rs

/// 统一的 URL 解析 trait（类似 OpenFeign 的 Target）
/// 职责：服务名 → 最终可用的 base URL（内部含服务发现 + 负载均衡）
#[async_trait]
pub trait UrlResolver: Send + Sync + 'static {
    async fn resolve(&self, service_name: &str) -> Result<String, Error>;
}

/// 静态 URL（不需要注册中心，直接返回配置的地址）
pub struct StaticUrl(String);

#[async_trait]
impl UrlResolver for StaticUrl {
    async fn resolve(&self, _service_name: &str) -> Result<String, Error> {
        Ok(self.0.clone())
    }
}
```

**各注册中心实现**（独立 crate，只是包一层现有 SDK）：

```rust
// feignx-nacos/src/lib.rs
// 底层直接用 nacos-sdk（已成熟，10 万下载）
use nacos_sdk::api::naming::NamingService;

pub struct NacosUrl {
    naming: Arc<dyn NamingService>,
    counter: AtomicUsize,  // 内置 RoundRobin
}

#[async_trait]
impl UrlResolver for NacosUrl {
    async fn resolve(&self, service_name: &str) -> Result<String, Error> {
        // 1. 调用 nacos-sdk 获取实例列表
        let instances = self.naming
            .get_all_instances(service_name.into(), None, Vec::new(), false)
            .await
            .map_err(|e| Error::Resolve(e.to_string()))?;

        // 2. 过滤健康实例
        let healthy: Vec<_> = instances.iter().filter(|i| i.healthy).collect();
        if healthy.is_empty() {
            return Err(Error::Resolve(format!("no healthy instance for {}", service_name)));
        }

        // 3. 负载均衡（RoundRobin）
        let idx = self.counter.fetch_add(1, Ordering::Relaxed) % healthy.len();
        let inst = &healthy[idx];

        Ok(format!("http://{}:{}", inst.ip, inst.port))
    }
}
```

```rust
// feignx-consul/src/lib.rs（同理，包一层 consul SDK）
pub struct ConsulUrl { /* consul client */ }
```

**为什么合并为一个 trait？**
- 用户不需要关心"先发现再均衡"的两步过程
- OpenFeign 的 `Target` 也是一步到位返回最终 URL
- 内部实现自由组合（Nacos + RoundRobin、Consul + Random 等）
- 对外只暴露一个简洁的 trait

### Middleware（中间件 — 装饰器模式）

```rust
// feignx-core/src/middleware.rs

/// 中间件 trait（包裹 Transport 调用链）
/// 类似 tower::Layer 但更简单，不需要 Service 泛型嵌套
#[async_trait]
pub trait Middleware: Send + Sync + 'static {
    async fn handle(
        &self,
        request: Request<Bytes>,
        next: Next<'_>,
    ) -> Result<Response<Bytes>, Error>;
}

/// Next 代表中间件链中的下一个节点
pub struct Next<'a> {
    transport: &'a dyn Transport,
    middlewares: &'a [Arc<dyn Middleware>],
    index: usize,
}

impl<'a> Next<'a> {
    pub async fn call(mut self, request: Request<Bytes>) -> Result<Response<Bytes>, Error> {
        if self.index < self.middlewares.len() {
            let mw = &self.middlewares[self.index];
            self.index += 1;
            mw.handle(request, self).await
        } else {
            self.transport.send(request).await
        }
    }
}
```

### RequestInterceptor（请求拦截器 — 轻量版）

```rust
// feignx-core/src/interceptor.rs

/// 简单的请求修改器（添加 header、修改 URL 等）
/// 比 Middleware 更轻量，不能控制响应
#[async_trait]
pub trait RequestInterceptor: Send + Sync {
    async fn intercept(&self, request: Request<Bytes>) -> Result<Request<Bytes>, Error>;
}
```

### Auth（认证抽象 — 参考 httpx / go-resty）

```rust
// feignx-core/src/auth.rs

/// 认证抽象，支持简单 token 和复杂认证流（如 OAuth token 刷新）
#[async_trait]
pub trait Auth: Send + Sync {
    /// 在请求发送前注入认证信息
    async fn authenticate(&self, request: &mut Request<Bytes>) -> Result<(), Error>;
}

/// Bearer Token 认证
pub struct BearerAuth {
    token: String,
}

impl BearerAuth {
    pub fn new(token: impl Into<String>) -> Self {
        Self { token: token.into() }
    }
}

#[async_trait]
impl Auth for BearerAuth {
    async fn authenticate(&self, request: &mut Request<Bytes>) -> Result<(), Error> {
        request.headers_mut().insert(
            "Authorization",
            format!("Bearer {}", self.token).parse().unwrap(),
        );
        Ok(())
    }
}

/// Basic 认证
pub struct BasicAuth {
    username: String,
    password: String,
}

/// 动态 Token（支持过期刷新）
pub struct DynamicAuth<F> {
    token_provider: F,  // async fn() -> Result<String>
}
```

使用方式：
```rust
// 简单 Bearer
let client = ClientBuilder::new()
    .auth(BearerAuth::new("my-jwt-token"))
    .build();

// Basic Auth
let client = ClientBuilder::new()
    .auth(BasicAuth::new("user", "pass"))
    .build();

// 动态 Token（自动刷新）
let client = ClientBuilder::new()
    .auth(DynamicAuth::new(|| async {
        // 从缓存或 OAuth 服务获取 token
        get_or_refresh_token().await
    }))
    .build();
```

### ErrorDecoder（错误解码 — 参考 OpenFeign）

```rust
// feignx-core/src/error_decoder.rs

/// 自定义错误解码器
/// 当响应状态码非 2xx 时调用，将响应体解析为具体错误
#[async_trait]
pub trait ErrorDecoder: Send + Sync {
    fn decode(&self, status: u16, headers: &HeaderMap, body: &[u8]) -> Error;
}

/// 默认实现：body 当字符串放进 Error::Status
pub struct DefaultErrorDecoder;

impl ErrorDecoder for DefaultErrorDecoder {
    fn decode(&self, status: u16, _headers: &HeaderMap, body: &[u8]) -> Error {
        Error::Status {
            status,
            body: String::from_utf8_lossy(body).into_owned(),
        }
    }
}
```

用户自定义示例：
```rust
/// 解析业务错误 JSON：{"code": "USER_NOT_FOUND", "message": "..."}
struct MyErrorDecoder;

impl ErrorDecoder for MyErrorDecoder {
    fn decode(&self, status: u16, _headers: &HeaderMap, body: &[u8]) -> Error {
        if let Ok(api_err) = serde_json::from_slice::<ApiError>(body) {
            Error::Business {
                code: api_err.code,
                message: api_err.message,
            }
        } else {
            Error::Status {
                status,
                body: String::from_utf8_lossy(body).into_owned(),
            }
        }
    }
}

let client = ClientBuilder::new()
    .error_decoder(MyErrorDecoder)
    .build();
```

### ResponseInterceptor（响应拦截器 — 参考 OpenFeign）

```rust
// feignx-core/src/interceptor.rs

/// 响应拦截器（检查/修改响应，如统一日志、指标采集）
#[async_trait]
pub trait ResponseInterceptor: Send + Sync {
    async fn intercept(&self, response: Response<Bytes>) -> Result<Response<Bytes>, Error>;
}
```

### LogLevel（日志级别 — 参考 OpenFeign 4 级别）

```rust
// feignx-core/src/log.rs

/// 请求日志级别
#[derive(Debug, Clone, Copy, Default)]
pub enum LogLevel {
    /// 不记录任何日志
    #[default]
    None,
    /// 记录：方法 + URL + 状态码 + 耗时
    Basic,
    /// Basic + 请求/响应头
    Headers,
    /// Headers + 请求/响应体
    Full,
}
```

### Codec（编解码）

```rust
// feignx-core/src/codec.rs

/// 请求体编码
pub trait Encoder: Send + Sync {
    fn encode<T: Serialize>(&self, value: &T) -> Result<Bytes, Error>;
    fn content_type(&self) -> &str;
}

/// 响应体解码
pub trait Decoder: Send + Sync {
    fn decode<T: DeserializeOwned>(&self, body: &[u8]) -> Result<T, Error>;
}

/// 默认 JSON
pub struct JsonCodec;
impl Encoder for JsonCodec { /* serde_json::to_vec */ }
impl Decoder for JsonCodec { /* serde_json::from_slice */ }
```

### ClientBuilder（组装所有扩展点）

```rust
// feignx-core/src/client.rs

pub struct ClientBuilder {
    transport: Box<dyn Transport>,
    url_resolver: Box<dyn UrlResolver>,
    auth: Option<Box<dyn Auth>>,
    error_decoder: Box<dyn ErrorDecoder>,
    interceptors: Vec<Box<dyn RequestInterceptor>>,
    middlewares: Vec<Box<dyn Middleware>>,
    encoder: Box<dyn Encoder>,
    decoder: Box<dyn Decoder>,
}

impl ClientBuilder {
    pub fn new(transport: impl Transport) -> Self {
        Self {
            transport: Box::new(transport),
            url_resolver: Box::new(StaticUrl("".into())),
            auth: None,
            error_decoder: Box::new(DefaultErrorDecoder),
            interceptors: vec![],
            middlewares: vec![],
            encoder: Box::new(JsonCodec),
            decoder: Box::new(JsonCodec),
        }
    }

    // 基础配置
    pub fn base_url(mut self, url: &str) -> Self {
        self.url_resolver = Box::new(StaticUrl(url.into()));
        self
    }
    pub fn url_resolver(mut self, resolver: impl UrlResolver) -> Self { ... }

    // 认证
    pub fn auth(mut self, auth: impl Auth + 'static) -> Self { ... }
    pub fn bearer_auth(self, token: &str) -> Self { self.auth(BearerAuth::new(token)) }
    pub fn basic_auth(self, user: &str, pass: &str) -> Self { self.auth(BasicAuth::new(user, pass)) }

    // 错误处理
    pub fn error_decoder(mut self, d: impl ErrorDecoder + 'static) -> Self { ... }

    // 中间件 / 拦截器
    pub fn interceptor(mut self, i: impl RequestInterceptor + 'static) -> Self { ... }
    pub fn response_interceptor(mut self, i: impl ResponseInterceptor + 'static) -> Self { ... }
    pub fn middleware(mut self, m: impl Middleware + 'static) -> Self { ... }

    // 超时（细粒度，参考 httpx）
    pub fn timeout(mut self, timeout: Timeout) -> Self { ... }
    pub fn connect_timeout(mut self, dur: Duration) -> Self { ... }
    pub fn read_timeout(mut self, dur: Duration) -> Self { ... }

    // 成功状态码判断（参考 Axios validateStatus）
    pub fn success_status(mut self, f: fn(u16) -> bool) -> Self { ... }

    // 日志
    pub fn log_level(mut self, level: LogLevel) -> Self { ... }

    // 编解码
    pub fn encoder(mut self, e: impl Encoder + 'static) -> Self { ... }
    pub fn decoder(mut self, d: impl Decoder + 'static) -> Self { ... }

    // 底层 reqwest 配置（HTTP/2、代理、TLS 等）
    pub fn reqwest_config(mut self, f: impl FnOnce(reqwest::ClientBuilder) -> reqwest::ClientBuilder) -> Self { ... }

    pub fn build(self) -> Client { ... }
}
```

### Timeout（细粒度超时 — 参考 httpx）

```rust
// feignx-core/src/timeout.rs

pub struct Timeout {
    /// TCP 连接超时
    pub connect: Duration,
    /// 读取响应超时
    pub read: Duration,
    /// 写入请求超时（大 body 时有用）
    pub write: Duration,
}

impl Default for Timeout {
    fn default() -> Self {
        Self {
            connect: Duration::from_secs(5),
            read: Duration::from_secs(30),
            write: Duration::from_secs(30),
        }
    }
}
```

### 请求取消（参考 Axios AbortController）

命令式 Client 支持取消长时间请求：

```rust
use tokio_util::sync::CancellationToken;

let cancel = CancellationToken::new();

// 发起请求
let handle = tokio::spawn({
    let cancel = cancel.clone();
    async move {
        client.get("/slow-endpoint")
            .cancel_on(cancel)
            .send()
            .await
    }
});

// 其他地方取消
cancel.cancel();

// handle.await => Err(Error::Cancelled)
```

### 生成 cURL 命令（参考 go-resty — 调试利器）

当 `LogLevel::Full` 时，自动在日志中输出等价的 cURL 命令：

```rust
let client = ClientBuilder::new()
    .log_level(LogLevel::Full)
    .build();

// 日志输出示例：
// [feignx] --> curl -X POST 'http://api.example.com/users' \
//   -H 'Content-Type: application/json' \
//   -H 'Authorization: Bearer xxx' \
//   -d '{"name":"Alice"}'
// [feignx] <-- 201 Created (23ms)
```

也可以编程式获取：

```rust
// 命令式 Client
let curl_cmd = client.post("/users")
    .json(&user)
    .to_curl();  // 返回 String，不发送请求

println!("{}", curl_cmd);
// curl -X POST 'http://...' -H '...' -d '...'
```

### 完整调用流程

```
用户调用 user_api.get_user(123)
    ↓
宏生成代码构建 http::Request<Bytes>
    ↓
Auth.authenticate(&mut request)         ← 认证扩展点（注入 token）
    ↓
RequestInterceptor 链（添加 header 等）
    ↓
UrlResolver.resolve(service_name)       ← 服务发现 + 负载均衡（一步到位）
    ↓
Middleware 链（retry → circuit_breaker → timeout → logging）
    ↓
Transport.send(request)                 ← 传输层扩展点
    ↓
状态码判断：
  2xx → Decoder.decode(response.body)   ← 编解码扩展点
  404 + Option<T> → Ok(None)
  非 2xx → ErrorDecoder.decode(status, headers, body) ← 错误解码扩展点
    ↓
返回 feignx::Result<T>
```

## 返回类型设计

提供 `feignx::Result<T>` 类型别名，用户不需要每次写 `Error`：

```rust
// feignx-core/src/error.rs

/// 客户端错误
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// HTTP 请求失败（网络错误、超时等）
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),

    /// 响应状态码非 2xx（默认错误解码）
    #[error("HTTP {status}: {body}")]
    Status { status: u16, body: String },

    /// 业务错误（自定义 ErrorDecoder 解析出的结构化错误）
    #[error("{code}: {message}")]
    Business { code: String, message: String },

    /// 响应体反序列化失败
    #[error("decode error: {0}")]
    Decode(#[from] serde_json::Error),

    /// 服务发现失败
    #[error("service resolve failed: {0}")]
    Resolve(String),

    /// 熔断器拒绝
    #[error("circuit breaker open")]
    CircuitOpen,
}

/// 类型别名，减少样板代码
pub type Result<T> = std::result::Result<T, Error>;
```

### 支持的返回类型

```rust
feignx::Result<T>              // 标准：2xx 反序列化为 T，非 2xx 走 ErrorDecoder
feignx::Result<Option<T>>      // 404 → Ok(None)，2xx → Ok(Some(T))
feignx::Result<()>             // 忽略响应体，只关心是否成功
feignx::Result<String>         // 原始文本
feignx::Result<Vec<u8>>        // 原始字节
feignx::Result<ApiResponse<T>> // 需要访问 status + headers + body
```

### ApiResponse（参考 Refit 的 ApiResponse<T>）

有时候用户不只要 body，还需要 status code 和 headers：

```rust
// feignx-core/src/response.rs

pub struct ApiResponse<T> {
    pub status: u16,
    pub headers: http::HeaderMap,
    pub body: T,
}

impl<T> ApiResponse<T> {
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}
```

使用示例：
```rust
#[get("/users/{id}")]
async fn get_user(&self, #[path] id: i64) -> feignx::Result<ApiResponse<User>>;

let resp = api.get_user(1).await?;
println!("status: {}, etag: {:?}", resp.status, resp.headers.get("etag"));
```

用户写法对比：

```rust
// 之前（冗长）
async fn get_user(&self, #[path] id: i64) -> Result<User, feignx::Error>;

// 现在（简洁）
async fn get_user(&self, #[path] id: i64) -> feignx::Result<User>;
```

### 可选：支持 Option 语义

404 响应自动映射为 `None`，而不是 Error：

```rust
// 返回 Option<T>：404 → Ok(None)，2xx → Ok(Some(T))，其他 → Err
#[get("/users/{id}")]
async fn find_user(&self, #[path] id: i64) -> feignx::Result<Option<User>>;
```

## 两种使用模式

**宏只是语法糖**，底层的 `Client` 是独立可用的。用户可以选择：

### 模式 1：声明式（#[http_client] 宏）

适合：接口固定、调用频繁、想要最简代码

```rust
#[http_client(service = "user-service")]
pub trait UserApi {
    #[get("/users/{id}")]
    async fn get_user(&self, #[path] id: i64) -> feignx::Result<User>;
}
```

### 模式 2：命令式（直接用 Client）

适合：动态 URL、自定义逻辑、不想用宏、但仍想要服务发现 + 重试 + 熔断

```rust
use feignx::{Client, ClientBuilder};

// 构建 Client（带服务发现 + 重试 + 熔断）
let client = ClientBuilder::new()
    .url_resolver(NacosUrl::new(naming_service))
    .retry(3)
    .circuit_breaker(5, Duration::from_secs(30))
    .tracing(true)
    .build();

// 手动发请求，但自动享有：
// - Nacos 服务发现（解析 user-service → 实际地址）
// - 负载均衡（多实例轮询）
// - 重试（失败自动重试 3 次）
// - 熔断（连续失败 5 次后熔断 30s）
// - 链路追踪（自动注入 traceparent 头）
let user: User = client
    .get("/users/123")
    .header("Authorization", format!("Bearer {}", token))
    .send()
    .await?
    .json()
    .await?;

// 动态 URL 也行
let resp = client
    .post("/webhooks/notify")
    .json(&payload)
    .send()
    .await?;
```

### 模式 3：混合（自己实现 trait，但用 Client 做底层）

适合：需要自定义逻辑（如签名、加密），但仍想复用基础设施

```rust
use feignx::Client;

#[derive(Clone)]
pub struct MyPaymentClient {
    client: Client,
    app_key: String,
    app_secret: String,
}

impl MyPaymentClient {
    pub fn new(client: Client, app_key: String, app_secret: String) -> Self {
        Self { client, app_key, app_secret }
    }

    pub async fn charge(&self, amount: u64, order_id: &str) -> feignx::Result<ChargeResult> {
        // 自定义签名逻辑
        let timestamp = chrono::Utc::now().timestamp();
        let sign = self.compute_sign(amount, order_id, timestamp);

        // 但底层仍然走 Client（享有服务发现 + 重试 + 熔断）
        self.client
            .post("/v2/payments/charge")
            .header("X-App-Key", &self.app_key)
            .header("X-Signature", &sign)
            .header("X-Timestamp", &timestamp.to_string())
            .json(&serde_json::json!({
                "amount": amount,
                "order_id": order_id,
            }))
            .send()
            .await?
            .json()
            .await
    }

    fn compute_sign(&self, amount: u64, order_id: &str, ts: i64) -> String {
        // HMAC-SHA256 签名...
        todo!()
    }
}
```

### 架构关系

```
#[http_client] 宏（模式 1）
        ↓ 生成代码调用
    feignx::Client（核心运行时）  ← 模式 2/3 直接使用
        ↓ 内部组合
    UrlResolver + reqwest-middleware
```

**宏不是必须的**。`Client` 是独立的、完整的运行时组件。
宏只是帮你省去手写 `client.get("/users/{id}")` 的样板代码。

## 宏设计

### 用户 API

```rust
use feignx::{http_client, get, post, put, delete, RequestParam};

// 类级别 headers：所有请求自动带这些头
#[http_client(
    base_url = "http://user-service:8080",
    headers = ["Accept: application/json", "X-Api-Version: v2"]
)]
pub trait UserApi {
    #[get("/users/{id}")]
    async fn get_user(&self, #[path] id: i64) -> feignx::Result<User>;

    #[get("/users")]
    async fn list_users(&self, #[query] params: ListParams) -> feignx::Result<Vec<User>>;

    #[post("/users")]
    async fn create_user(&self, #[body] req: CreateUserReq) -> feignx::Result<User>;

    #[delete("/users/{id}")]
    async fn delete_user(&self, #[path] id: i64) -> feignx::Result<()>;

    // 404 → Ok(None)
    #[get("/users/{id}")]
    async fn find_user(&self, #[path] id: i64) -> feignx::Result<Option<User>>;

    #[get("/users/{id}/avatar")]
    async fn get_avatar(
        &self,
        #[path] id: i64,
        #[header("Authorization")] token: String,
    ) -> feignx::Result<Vec<u8>>;

    // 批量动态头（类似 OpenFeign @HeaderMap）
    #[get("/data")]
    async fn get_data(
        &self,
        #[headers] extra_headers: HashMap<String, String>,
    ) -> feignx::Result<Data>;

    // 方法级超时覆盖（类似 OpenFeign Request.Options）
    #[get("/slow-report")]
    #[timeout(30000)]  // 30s，覆盖全局超时
    async fn get_slow_report(&self) -> feignx::Result<Report>;

    // 复杂请求使用 RequestParam 结构体
    #[put("/users/{id}")]
    async fn update_user(&self, req: UpdateUserRequest) -> feignx::Result<User>;
}

// 服务名模式（需要 UrlResolver）
#[http_client(service = "user-service", path = "/api/v1")]
pub trait UserApiV2 {
    #[get("/users/{id}")]
    async fn get_user(&self, #[path] id: i64) -> feignx::Result<User>;
}
```

### 参数别名（参考 Refit [AliasAs]）

当 Rust 字段名和 URL 参数名不一致时：

```rust
// URL 中是 user_id，但 Rust 变量名想用 id
#[get("/users/{user_id}")]
async fn get_user(&self, #[path(name = "user_id")] id: i64) -> feignx::Result<User>;

// Query 参数别名
#[get("/search")]
async fn search(&self, #[query(name = "q")] keyword: String) -> feignx::Result<Vec<Item>>;
```

### Query 数组格式（参考 Refit CollectionFormat）

数组参数序列化为 query string 的方式：

```rust
// Multi（默认）: ?ids=1&ids=2&ids=3
#[get("/users")]
async fn get_by_ids(&self, #[query] ids: Vec<i64>) -> feignx::Result<Vec<User>>;

// Csv: ?ids=1,2,3
#[get("/users")]
async fn get_by_ids(&self, #[query(format = "csv")] ids: Vec<i64>) -> feignx::Result<Vec<User>>;

// 支持的格式：
// - multi（默认）: ?ids=1&ids=2&ids=3
// - csv:           ?ids=1,2,3
// - ssv:           ?ids=1 2 3（空格分隔）
// - pipes:         ?ids=1|2|3
```

### 参数传递规则

**规则 1**：普通结构体不加注解 → 默认作为 `#[body]` JSON 发送

```rust
#[post("/users")]
async fn create_user(&self, #[body] req: CreateUserReq) -> Result<User, Error>;
// CreateUserReq 会被 serde_json::to_vec 序列化为 JSON body
```

**规则 2**：需要混合参数（path + query + body + header 同时存在）→ 用 `#[derive(RequestParam)]`

```rust
use feignx::RequestParam;

#[derive(RequestParam)]
pub struct UpdateUserRequest {
    #[path]
    pub id: i64,

    #[query]
    pub version: Option<i32>,

    #[body]
    pub data: UpdateUserReq,

    #[header("X-Request-Id")]
    pub request_id: String,
}

// 使用时只需一个参数
#[put("/users/{id}")]
async fn update_user(&self, req: UpdateUserRequest) -> Result<User, Error>;
```

宏自动将结构体字段拆分到请求的不同部分：
- `#[path]` → URL 路径参数替换
- `#[query]` → URL query string（`?version=2`）
- `#[body]` → 请求体（JSON 序列化）
- `#[header("name")]` → 请求头

**规则 3**：没有 `#[derive(RequestParam)]` 的结构体，直接传递时默认为 body

```rust
// 这两种写法等价：
#[post("/users")]
async fn create_user(&self, #[body] req: CreateUserReq) -> Result<User, Error>;

#[post("/users")]
async fn create_user(&self, req: CreateUserReq) -> Result<User, Error>;
// 没有标注 → 默认当 body 处理
```

**何时用 `#[derive(RequestParam)]`**：
- 一个请求同时需要 path + query + body + header 中的多种
- 参数组合需要在多个方法间复用
- 参数超过 3 个，逐个标注太冗长

### 文件上传（#[multipart]）

参考 Refit 的 `[Multipart]` + go-resty 的 `.SetFile()`：

```rust
use feignx::Part;

#[http_client(base_url = "http://file-service:8080")]
pub trait FileApi {
    /// 单文件上传
    #[post("/upload")]
    #[multipart]
    async fn upload(
        &self,
        #[part(name = "file")] file: Part,
        #[part(name = "description")] desc: String,
    ) -> feignx::Result<UploadResult>;

    /// 多文件上传
    #[post("/batch-upload")]
    #[multipart]
    async fn batch_upload(
        &self,
        #[part(name = "files")] files: Vec<Part>,
    ) -> feignx::Result<Vec<UploadResult>>;
}
```

`Part` 类型：
```rust
// feignx-core/src/part.rs

pub struct Part {
    pub filename: String,
    pub content_type: String,
    pub data: Bytes,
}

impl Part {
    /// 从文件路径创建
    pub async fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> { ... }

    /// 从内存数据创建
    pub fn from_bytes(filename: &str, content_type: &str, data: Vec<u8>) -> Self { ... }
}
```

使用示例：
```rust
let file = Part::from_file("/path/to/photo.jpg").await?;
let result = file_api.upload(file, "my photo".into()).await?;
```

### Streaming 响应（参考 httpx / go-resty）

大文件下载或 SSE 场景，不应该把整个 body 加载到内存：

```rust
use feignx::ByteStream;
use futures::StreamExt;

#[http_client(base_url = "http://file-service:8080")]
pub trait FileApi {
    /// 返回 ByteStream，逐块读取
    #[get("/files/{id}/download")]
    async fn download(&self, #[path] id: i64) -> feignx::Result<ByteStream>;
}

// ByteStream 是 impl Stream<Item = Result<Bytes, Error>>
let mut stream = file_api.download(123).await?;
while let Some(chunk) = stream.next().await {
    let bytes = chunk?;
    file.write_all(&bytes).await?;
}
```

### 宏生成的代码（简化）

```rust
// 宏自动生成：
#[derive(Clone)]
pub struct UserApiClient {
    client: feignx_core::Client,
}

impl UserApiClient {
    pub fn new(client: feignx_core::Client) -> Self {
        Self { client }
    }
}

impl UserApi for UserApiClient {
    async fn get_user(&self, id: i64) -> Result<User, Error> {
        let request = Request {
            method: Method::GET,
            url: format!("{}/users/{}", self.client.base_url(), id),
            headers: HeaderMap::new(),
            body: None,
        };
        let response = self.client.execute(request).await?;
        self.client.decode(&response.body)
    }

    // ... 其他方法类似
}
```

### 用户如何实例化

```rust
// 纯 Rust，无框架：
let client = feign::ClientBuilder::new(feignx_reqwest::ReqwestTransport::new())
    .base_url("http://user-service:8080")
    .interceptor(AuthInterceptor::new("my-token"))
    .middleware(feignx_middleware::Retry::new(3))
    .middleware(feignx_middleware::Timeout::new(Duration::from_secs(5)))
    .build();

let user_api = UserApiClient::new(client);
let user = user_api.get_user(123).await?;
```

## 中间件示例

### 重试

```rust
// feignx-middleware/src/retry.rs

pub struct Retry {
    max_attempts: u32,
    backoff: BackoffStrategy,
    retryable: fn(u16) -> bool,  // 哪些状态码可重试
}

#[async_trait]
impl Middleware for Retry {
    async fn handle(&self, request: Request, next: &dyn Transport) -> Result<Response, Error> {
        let mut attempts = 0;
        loop {
            let req = request.clone();
            match next.send(req).await {
                Ok(resp) if !self.retryable(resp.status) || attempts >= self.max_attempts => {
                    return Ok(resp);
                }
                Err(e) if attempts >= self.max_attempts => {
                    return Err(e);
                }
                _ => {
                    attempts += 1;
                    tokio::time::sleep(self.backoff.delay(attempts)).await;
                }
            }
        }
    }
}
```

### 熔断器

```rust
// feignx-middleware/src/circuit_breaker.rs

pub struct CircuitBreaker {
    failure_threshold: u32,
    recovery_timeout: Duration,
    state: Arc<RwLock<State>>,
}

enum State {
    Closed { failures: u32 },
    Open { since: Instant },
    HalfOpen,
}
```

### Trace 透传

```rust
// feignx-middleware/src/trace.rs

pub struct TracePropagation;

#[async_trait]
impl RequestInterceptor for TracePropagation {
    async fn intercept(&self, mut request: Request) -> Result<Request, Error> {
        // 从当前 span 提取 trace context，注入到请求头
        if let Some(ctx) = current_trace_context() {
            request.headers.insert("traceparent", ctx.traceparent());
        }
        Ok(request)
    }
}
```

## summer-rs 集成（薄插件）

```rust
// summer-rs 仓库中：summer-feignx/src/lib.rs

use summer::{app::AppBuilder, plugin::Plugin, async_trait};
use summer::config::Configurable;
use feignx::{ClientBuilder, Client};
use feignx_reqwest::ReqwestTransport;

pub struct FeignPlugin;

#[async_trait]
impl Plugin for FeignPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let config = app.get_config::<HttpClientConfig>().unwrap_or_default();

        // 构建全局 Client
        let mut builder = ClientBuilder::new(ReqwestTransport::new())
            .timeout(config.timeout);

        // 检测 summer-opentelemetry 是否存在，自动加 trace 透传
        #[cfg(feature = "opentelemetry")]
        {
            builder = builder.interceptor(feignx_middleware::TracePropagation);
        }

        let client = builder.build();
        app.add_component(client);
    }
}

// 配置
#[derive(Configurable, Deserialize)]
#[config_prefix = "http"]
struct HttpClientConfig {
    #[serde(default = "default_timeout")]
    timeout: Duration,
}
```

用户在 summer-rs 中使用：

```rust
use summer::App;
use summer_feignx::FeignPlugin;

#[auto_config(FeignConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(FeignPlugin)
        .run()
        .await;
}

// 注入使用
#[derive(Clone, Service)]
struct OrderService {
    #[inject(component)]
    user_api: UserApiClient,
}
```

## 与 feignhttp 的差异

| 维度 | feignhttp | 本库 |
|------|-----------|------|
| 架构 | 单 crate | workspace 多 crate |
| 传输层 | 硬编码 reqwest/isahc | `Transport` trait 可插拔 |
| 中间件 | 无 | `Middleware` + `RequestInterceptor` |
| 编解码 | 硬编码 | `Encoder` / `Decoder` trait 可插拔 |
| 框架集成 | 无 | 独立集成包（summer-feignx 等） |
| 重试/熔断 | 无 | 复用 reqwest-retry / failsafe |
| 可观测性 | 无 | trace 透传 + 请求日志 |

## 实现计划

### Phase 1：核心 + 宏（MVP）

**目标**：能声明式定义 HTTP 客户端，发请求，拿结果。

- [ ] `feignx-core`：Transport / Middleware / RequestInterceptor / ResponseInterceptor / Codec / Error
- [ ] `feignx-core`：Auth trait + BearerAuth / BasicAuth 内置实现
- [ ] `feignx-core`：ErrorDecoder trait + DefaultErrorDecoder
- [ ] `feignx-core`：`feignx::Result<T>` / `ApiResponse<T>` / `Option<T>` 语义
- [ ] `feignx-core`：Timeout 结构体（connect / read / write）
- [ ] `feignx-macros`：`#[http_client]` + `#[get/post/put/delete/patch/head]`
- [ ] `feignx-macros`：`#[path]` / `#[query]` / `#[body]` / `#[header]` / `#[headers]`
- [ ] `feignx-macros`：参数别名 `#[path(name = "xxx")]`
- [ ] `feignx-macros`：类级别 headers `#[http_client(headers = [...])]`
- [ ] `feignx-macros`：`#[derive(RequestParam)]`
- [ ] `feignx-macros`：Query 数组格式 `#[query(format = "multi/csv")]`
- [ ] `feignx-reqwest`：reqwest Transport 实现
- [ ] `feignx`：伞 crate
- [ ] 命令式 Client API（不用宏也能用）
- [ ] `reqwest_config` 钩子（暴露底层 reqwest 配置）
- [ ] `.success_status()` 自定义成功状态码判断
- [ ] 基础示例

### Phase 2：生产可用

**目标**：重试、超时、文件上传、日志、取消。

- [ ] 重试（feature `retry`，底层 reqwest-retry / backon）
- [ ] `#[timeout(ms)]` 方法级超时覆盖
- [ ] 熔断器（feature `circuit-breaker`，底层 failsafe）
- [ ] `#[multipart]` + `Part` 文件上传
- [ ] `ByteStream` Streaming 响应（大文件下载）
- [ ] 请求取消（`CancellationToken`）
- [ ] 请求/响应日志 + LogLevel（NONE/BASIC/HEADERS/FULL）
- [ ] 生成 cURL 命令（LogLevel::Full 自动输出 / `.to_curl()` 编程式）
- [ ] DynamicAuth（token 过期自动刷新）

### Phase 3：服务发现

**目标**：按服务名调用，负载均衡。

- [ ] `UrlResolver` trait（统一服务发现 + 负载均衡）
- [ ] `StaticUrl` 内置实现
- [ ] `#[http_client(service = "xxx")]` 语法
- [ ] `feignx-nacos`（feature `nacos`，底层包 nacos-sdk）
- [ ] `feignx-consul`（feature `consul`）

### Phase 4：框架集成

**目标**：与 summer-rs 等框架打通。

- [ ] `summer-feignx`（summer-rs 插件，读取配置 + 注册 Component）
- [ ] 文档 + 其他框架集成指南（axum / actix-web / salvo）