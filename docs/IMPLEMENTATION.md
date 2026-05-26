# 各框架内部实现架构对比

> 对比 OpenFeign / Axios / go-resty 的源码实现，评估我们的设计优雅性

## 零、与 OpenFeign 功能对齐检查

| OpenFeign 功能 | 我们是否有 | 对应设计 | 备注 |
|---------------|:---:|---------|------|
| `@RequestLine("GET /path/{id}")` | ✅ | `#[get("/path/{id}")]` | 语法更简洁 |
| `@Param("id")` | ✅ | `#[path] id` | 同 |
| `@Headers` (类级别) | ❌ | 缺失 | 需要加：`#[http_client(headers = [...])]` |
| `@Headers` (方法级别) | ✅ | `#[header("name")] value` | 同 |
| `@Body` (模板化) | ⚠️ | `#[body]` 只支持序列化 | OpenFeign 支持字符串模板，我们不需要 |
| `@QueryMap` (Map/POJO → query) | ✅ | `#[query] params: T` (T: Serialize) | 自动展开字段 |
| `@HeaderMap` (Map → headers) | ❌ | 缺失 | 需要加：`#[headers] map: HashMap<String, String>` |
| Contract (注解解析抽象) | N/A | proc macro 直接解析 | Rust 不需要，编译时完成 |
| Encoder / Decoder | ✅ | `Encoder` / `Decoder` trait | 同 |
| ErrorDecoder | ✅ | `ErrorDecoder` trait | 同 |
| Client (传输抽象) | ✅ | `Transport` trait | 同 |
| RequestInterceptor | ✅ | `RequestInterceptor` trait | 同 |
| ResponseInterceptor | ❌ | 缺失 | 需要加 |
| Retryer | ✅ | feature `retry` (reqwest-retry) | 复用成熟库 |
| Logger (4 级别) | ❌ | 缺失 | 需要加：NONE/BASIC/HEADERS/FULL |
| Target (服务发现) | ✅ | `UrlResolver` trait | 同 |
| AsyncFeign | ✅ | 默认就是 async | Rust 天然异步 |
| Metrics/Capability | ❌ | 缺失 | P2，通过 reqwest-tracing |
| HystrixFeign (熔断) | ✅ | feature `circuit-breaker` (failsafe) | 复用成熟库 |
| FormEncoder (multipart) | ✅ | `#[multipart]` + `Part` | 同 |
| 接口继承 | ❌ | 缺失 | Rust trait 继承，但 macro 处理复杂 |
| default/static 方法 | ✅ | Rust trait 天然支持 default 方法 | 无需额外处理 |
| Request.Options (per-request 超时) | ❌ | 缺失 | 需要加 |

### 缺失功能分析

**需要补充的（重要）**：

1. **类级别 Headers** — 所有请求共享的默认头
```rust
#[http_client(
    base_url = "http://api.example.com",
    headers = ["Accept: application/json", "X-Api-Version: v2"]
)]
trait MyApi { ... }
```

2. **`#[headers]` 参数** — 动态批量头（类似 @HeaderMap）
```rust
#[get("/data")]
async fn get_data(&self, #[headers] extra: HashMap<String, String>) -> feign::Result<Data>;
```

3. **ResponseInterceptor** — 响应拦截器（修改/检查响应）
```rust
#[async_trait]
pub trait ResponseInterceptor: Send + Sync {
    async fn intercept(&self, response: Response<Bytes>) -> Result<Response<Bytes>, Error>;
}
```

4. **日志级别控制** — 参考 OpenFeign 的 4 级别
```rust
pub enum LogLevel {
    None,    // 不记录
    Basic,   // 方法 + URL + 状态码 + 耗时
    Headers, // Basic + 请求/响应头
    Full,    // Headers + 请求/响应体
}

let client = ClientBuilder::new()
    .log_level(LogLevel::Headers)
    .build();
```

5. **Per-request Options** — 单个请求覆盖超时等配置
```rust
#[get("/slow-endpoint")]
#[timeout(30000)]  // 这个请求 30s 超时，覆盖全局配置
async fn slow_call(&self) -> feign::Result<Data>;
```

**不需要的**：
- Contract — Rust proc macro 编译时解析，不需要运行时 Contract 抽象
- `@Body` 模板化 — 现代 API 都用 JSON，字符串模板没必要
- 接口继承 — P3 考虑，proc macro 处理 supertrait 有复杂度

### 与其他框架对齐检查

#### Refit (C#) 独有功能

| Refit 功能 | 我们是否有 | 建议 |
|-----------|:---:|------|
| `[AliasAs("name")]` 参数别名 | ❌ | 需要加：`#[path(name = "user_id")] id: i64` |
| `[Query(CollectionFormat.Multi)]` 数组格式 | ❌ | 需要加：`#[query(format = "multi")]` |
| `Task<ApiResponse<T>>` 带元数据返回 | ✅ | `feign::Result<ApiResponse<T>>` |
| `[Authorize]` 快捷认证 | ✅ | `.bearer_auth()` / Auth trait |
| `[Multipart]` 文件上传 | ✅ | `#[multipart]` + `Part` |
| Interface 继承 | ❌ | P3 |
| `IObservable<T>` 响应式 | ❌ | 不需要，Rust 用 Stream |

#### go-resty (Go) 独有功能

| go-resty 功能 | 我们是否有 | 建议 |
|--------------|:---:|------|
| `.SetDebug(true)` 调试模式 | ❌ | 通过 LogLevel::Full 覆盖 |
| 生成 cURL 命令 | ❌ | 有用！调试利器 |
| Request Trace (DNS/TLS/连接耗时) | ❌ | P2，通过 reqwest-tracing |
| `.SetResult(&T{})` / `.SetError(&T{})` | ✅ | ErrorDecoder + Decoder |
| `.OnError` 错误回调 | ✅ | ResponseInterceptor |
| 自定义 JSON 库 | ✅ | Encoder/Decoder trait |
| Cookie 持久化 | ✅ | reqwest 自带 |

#### Axios (JS) 独有功能

| Axios 功能 | 我们是否有 | 建议 |
|-----------|:---:|------|
| `validateStatus` 自定义成功判断 | ❌ | 需要加 |
| Cancel / AbortController | ❌ | 需要加（tokio CancellationToken） |
| `transformRequest/Response` | ✅ | Encoder/Decoder |
| Progress tracking | ❌ | P3，大文件场景 |
| 请求拦截器 LIFO 顺序 | ✅ | 我们按 Vec 顺序执行 |

#### httpx (Python) 独有功能

| httpx 功能 | 我们是否有 | 建议 |
|-----------|:---:|------|
| 细粒度超时 (connect/read/write/pool) | ❌ | 需要加 |
| MockTransport (测试) | ✅ | Transport trait 天然可 mock |
| Streaming 响应 | ❌ | 需要加 |
| `mounts` 路由级 transport | ❌ | 不需要 |
| Auth 多轮认证流 | ✅ | Auth trait |

### 需要补充的功能（汇总）

**P1 — 生产必需**：

1. **参数别名** — 字段名和 URL 参数名不一致时
```rust
#[get("/users/{user_id}")]
async fn get_user(&self, #[path(name = "user_id")] id: i64) -> feign::Result<User>;
```

2. **Query 数组格式** — 数组参数如何序列化
```rust
// Multi: ?ids=1&ids=2&ids=3
#[get("/users")]
async fn get_users(&self, #[query(format = "multi")] ids: Vec<i64>) -> feign::Result<Vec<User>>;

// Csv: ?ids=1,2,3
#[get("/users")]
async fn get_users(&self, #[query(format = "csv")] ids: Vec<i64>) -> feign::Result<Vec<User>>;
```

3. **自定义成功状态码** — 不是所有 API 都用 2xx 表示成功
```rust
let client = ClientBuilder::new()
    .success_status(|status| status < 500)  // 4xx 也算成功
    .build();
```

4. **请求取消** — 长时间请求需要能取消
```rust
use tokio_util::sync::CancellationToken;

let token = CancellationToken::new();
let result = client.get("/slow")
    .cancel_on(token.clone())
    .send()
    .await;

// 其他地方取消
token.cancel();
```

5. **细粒度超时** — 参考 httpx 的四维超时
```rust
let client = ClientBuilder::new()
    .timeout(Timeout {
        connect: Duration::from_secs(5),
        read: Duration::from_secs(30),
        write: Duration::from_secs(10),
    })
    .build();
```

**P2 — 锦上添花**：

6. **生成 cURL 命令** — 调试利器（参考 go-resty）
```rust
let client = ClientBuilder::new()
    .log_level(LogLevel::Full)  // Full 级别自动输出 cURL
    .build();

// 日志输出：
// --> curl -X GET 'http://api.example.com/users/1' -H 'Authorization: Bearer xxx'
```

7. **Streaming 响应** — 大文件下载
```rust
#[get("/files/{id}/download")]
async fn download(&self, #[path] id: i64) -> feign::Result<feign::Stream>;

// Stream 是 impl futures::Stream<Item = Result<Bytes, Error>>
```

---

## 一、核心执行模式对比

### OpenFeign — 动态代理 + MethodHandler

```
api.getUser(123)
    ↓
JDK Proxy → FeignInvocationHandler.invoke()
    ↓ 查 dispatch Map<Method, MethodHandler>
SynchronousMethodHandler.invoke(argv)
    ↓
1. argv → RequestTemplate（模板化请求构建）
2. RequestInterceptor 链逐个 apply
3. Target.apply(template) → Request（服务发现在这里）
4. runWithRetry {
       client.execute(request, options)
       responseHandler.handleResponse()
   }
```

**设计模式**：动态代理 + 策略模式 + 模板方法

### Axios — Promise 链

```
axios.get(url)
    ↓
构建 Promise 链：
[reqInterceptor2, reqInterceptor1, dispatchRequest, resInterceptor1, resInterceptor2]
    ↓
Promise.resolve(config)
  .then(reqInterceptor2)
  .then(reqInterceptor1)
  .then(dispatchRequest)    ← adapter(config) → response
  .then(resInterceptor1)
  .then(resInterceptor2)
```

**设计模式**：责任链（Promise 链）+ 适配器模式

### go-resty — 中间件管道

```
client.R().Get(url)
    ↓
Execute(method, url):
1. beforeRequest 中间件链（按顺序）：
   parseURL → parseHeader → parseBody → createHTTPRequest → addCredentials → logger
2. httpClient.Do(request)  ← 实际 HTTP 调用
3. afterResponse 中间件链（按顺序）：
   autoParseResponse（自动反序列化）→ 用户 hooks
```

**设计模式**：Builder + 中间件管道（函数切片）

---

## 二、关键设计决策对比

| 维度 | OpenFeign | Axios | go-resty | 我们的设计 |
|------|-----------|-------|----------|-----------|
| 代码生成 | 运行时（JDK Proxy） | 无 | 无 | 编译时（proc macro） |
| 中间件类型 | interface | function pair | function | trait |
| 中间件存储 | List | Array | Slice | Vec |
| 请求构建 | RequestTemplate | config 对象 | Request struct | http::Request<Bytes> |
| 重试位置 | 包裹 execute+decode | 无内置 | 包裹 HTTP 调用 | Middleware 层 |
| 认证 | RequestInterceptor | interceptor | 内置中间件 | Auth trait |
| 错误处理 | ErrorDecoder | AxiosError | SetResult/SetError | ErrorDecoder trait |
| 服务发现 | Target.apply() | 无 | 无 | ServiceResolver trait |

---

## 三、优雅性评估

### 我们做得好的地方

1. **编译时生成 vs 运行时代理** — OpenFeign 用 JDK Proxy（运行时反射），我们用 proc macro（编译时生成）。零运行时开销，类型安全，IDE 友好。

2. **标准类型** — 用 `http::Request<Bytes>` 而非自定义类型。OpenFeign 和 go-resty 都定义了自己的 Request/Response，增加了学习成本。

3. **Middleware 的 `next` 模式** — 类似 tower 的洋葱模型，比 go-resty 的"前后分离"更强大（可以做 timing、retry 等需要包裹整个调用的逻辑）。

### 需要反思的地方

**问题 1：trait 是否太多？**

当前设计有 9 个 trait：
```
Transport, Middleware, RequestInterceptor, Auth,
ErrorDecoder, Encoder, Decoder, ServiceResolver, LoadBalancer
```

对比：
- OpenFeign：5 个核心接口（Client, Encoder, Decoder, RequestInterceptor, Retryer）
- go-resty：2 个函数类型（RequestMiddleware, ResponseMiddleware）
- Axios：1 个概念（interceptor = {fulfilled, rejected}）

**结论**：trait 数量可以接受。因为：
- 用户通常只接触 `ClientBuilder`，不需要知道所有 trait
- 每个 trait 职责单一，不会混淆
- Rust 的 trait 是零成本抽象

**问题 2：Auth 是否应该独立？**

OpenFeign 中 Auth 就是一个 `RequestInterceptor`。go-resty 中 Auth 是内置中间件。

我们把 Auth 独立出来的好处：
- `ClientBuilder` 上有 `.bearer_auth()` / `.basic_auth()` 快捷方法
- Auth 在 Interceptor 之前执行（语义更清晰）
- 支持动态 token 刷新（比普通 Interceptor 更复杂）

**结论**：保留独立 Auth trait，但内部实现时 Auth 就是第一个执行的 Interceptor。

**问题 3：ServiceResolver + LoadBalancer 是否应该合并？**

当前是两步：`resolve(name) → Vec<Instance>` → `choose(instances) → Instance`

go-resty 没有这个概念。OpenFeign 的 Target 是一步到位（直接返回最终 URL）。

**结论**：合并为一个 trait 更简洁：

```rust
/// 服务地址解析（含负载均衡）
pub trait UrlResolver: Send + Sync {
    async fn resolve(&self, service_name: &str) -> Result<String, Error>;
}
```

内部实现可以组合 ServiceDiscovery + LoadBalancer，但对外只暴露一个 trait。

**问题 4：执行流程是否太长？**

当前：Auth → Interceptors → Resolver → Middleware → Transport → ErrorDecoder

go-resty：beforeMiddlewares → HTTP → afterMiddlewares（3 步）

**结论**：流程长但每步职责清晰。关键是用户不需要关心这个流程——宏生成的代码自动走完。只有自定义扩展时才需要理解。

---

## 四、建议的设计调整

基于以上分析，建议一处调整：

**合并 ServiceResolver + LoadBalancer 为 UrlResolver**

之前：
```rust
pub trait ServiceResolver { async fn resolve(&self, name: &str) -> Result<Vec<ServiceInstance>>; }
pub trait LoadBalancer { async fn choose(&self, instances: &[ServiceInstance]) -> Option<&ServiceInstance>; }
```

之后：
```rust
/// 统一的 URL 解析 trait（类似 OpenFeign 的 Target）
pub trait UrlResolver: Send + Sync + 'static {
    async fn resolve(&self, service_name: &str) -> Result<String, Error>;
}

/// 静态 URL（不需要服务发现）
pub struct StaticUrl(String);

/// Nacos 服务发现 + 负载均衡（内部组合）
pub struct NacosUrl {
    client: NacosClient,
    balancer: RoundRobin,  // 内部持有
}
```

这样 ClientBuilder 更简洁：
```rust
// 之前
.service("user-service", NacosResolver::new(nacos))
.balancer(RoundRobin::new())

// 之后
.url_resolver(NacosUrl::new(nacos))  // 内部已含负载均衡
```

其余设计保持不变，整体架构是优雅的。