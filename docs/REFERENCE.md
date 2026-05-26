# 各语言 HTTP 客户端功能对比参考

> 用于决策 summer-http 需要实现哪些功能

## 一、功能矩阵总览

| 功能 | OpenFeign (Java) | Refit (C#) | go-resty (Go) | Axios (JS) | httpx (Python) |
|------|:---:|:---:|:---:|:---:|:---:|
| 声明式定义 | ✅ trait/interface | ✅ interface | ❌ | ❌ | ❌ |
| 路径参数 | ✅ `@Param` | ✅ `{id}` | ✅ 手动 | ✅ 手动 | ✅ 手动 |
| Query 参数 | ✅ `@QueryMap` | ✅ `[Query]` | ✅ `.SetQueryParam` | ✅ `params` | ✅ `params` |
| 请求头 | ✅ `@Headers` | ✅ `[Header]` | ✅ `.SetHeader` | ✅ `headers` | ✅ `headers` |
| Body 序列化 | ✅ Encoder | ✅ `[Body]` | ✅ `.SetBody` 自动 | ✅ 自动 | ✅ `json=` |
| 多种 Content-Type | ✅ JSON/Form/XML | ✅ JSON/Form/XML/Stream | ✅ JSON/XML/Form | ✅ JSON/Form | ✅ JSON/Form |
| 文件上传 | ✅ FormEncoder | ✅ `[Multipart]` | ✅ `.SetFile` | ✅ FormData | ✅ `files=` |
| 拦截器/中间件 | ✅ RequestInterceptor | ✅ DelegatingHandler | ✅ OnBeforeRequest | ✅ interceptors | ✅ event_hooks |
| 重试 | ✅ Retryer | ⚠️ 通过 Polly | ✅ 内置 | ❌ 第三方 | ❌ 第三方 |
| 熔断 | ✅ HystrixFeign | ⚠️ 通过 Polly | ❌ 第三方 | ❌ | ❌ |
| 超时 | ✅ Request.Options | ✅ HttpClient | ✅ `.SetTimeout` | ✅ `timeout` | ✅ Timeout 四维 |
| 认证 | ✅ BasicAuth 拦截器 | ✅ `[Authorize]` | ✅ `.SetAuthToken` | ✅ 拦截器 | ✅ Auth 类 |
| 日志 | ✅ 4 级别 | ⚠️ Handler | ✅ `.SetDebug` | ❌ | ✅ event_hooks |
| 异步 | ✅ CompletableFuture | ✅ Task<T> | ❌ 同步 | ✅ Promise | ✅ AsyncClient |
| 错误处理 | ✅ ErrorDecoder | ✅ ApiException | ✅ `.SetError` 自动 | ✅ AxiosError | ✅ HTTPStatusError |
| 取消请求 | ❌ | ✅ CancellationToken | ✅ context | ✅ AbortController | ✅ cancel |
| 连接池 | ✅ Client 实现 | ✅ HttpClient | ✅ 内置 | ✅ Agent | ✅ Limits |
| HTTP/2 | ✅ Http2Client | ✅ HttpClient | ❌ | ❌ | ✅ `http2=True` |
| 代理 | ✅ Client 实现 | ✅ HttpClient | ✅ `.SetProxy` | ✅ `proxy` | ✅ `proxy=` |
| 进度追踪 | ❌ | ❌ | ❌ | ✅ onUploadProgress | ✅ stream |
| Mock/测试 | ❌ | ✅ Handler | ❌ | ✅ adapter | ✅ MockTransport |
| 接口继承 | ✅ | ✅ | N/A | N/A | N/A |
| 服务发现 | ✅ Target | ❌ | ❌ | ❌ | ❌ |
| 编解码可插拔 | ✅ Encoder/Decoder | ✅ IHttpContentSerializer | ✅ Marshal/Unmarshal | ✅ transformers | ❌ |

---

## 二、各功能详细对比

### 2.1 声明式定义方式

**OpenFeign (Java)**
```java
@RequestLine("GET /repos/{owner}/{repo}/contributors")
@Headers("Accept: application/json")
List<Contributor> contributors(@Param("owner") String owner, @Param("repo") String repo);
```

**Refit (C#)**
```csharp
[Get("/users/{id}")]
Task<User> GetUser(int id);

[Post("/users")]
Task CreateUser([Body] User user);
```

**我们的设计**
```rust
#[get("/users/{id}")]
async fn get_user(&self, #[path] id: i64) -> feign::Result<User>;
```

**决策点**：我们的语法更接近 Refit（简洁），同时借鉴 OpenFeign 的 `@Param` 显式标注。

---

### 2.2 拦截器/中间件

**OpenFeign** — `RequestInterceptor`（修改请求）+ `ResponseInterceptor`（修改响应）
```java
static class AuthInterceptor implements RequestInterceptor {
    public void apply(RequestTemplate template) {
        template.header("Authorization", "Bearer " + getToken());
    }
}
```

**Refit** — `DelegatingHandler`（.NET 标准管道）
```csharp
class LoggingHandler : DelegatingHandler {
    protected override async Task<HttpResponseMessage> SendAsync(
        HttpRequestMessage request, CancellationToken ct) {
        Log(request);
        var response = await base.SendAsync(request, ct);
        Log(response);
        return response;
    }
}
```

**go-resty** — 函数切片（OnBeforeRequest / OnAfterResponse / OnError）
```go
client.OnBeforeRequest(func(c *resty.Client, req *resty.Request) error {
    req.SetHeader("X-Request-Id", uuid.New().String())
    return nil
})
client.OnAfterResponse(func(c *resty.Client, resp *resty.Response) error {
    log.Printf("%s %s -> %d", resp.Request.Method, resp.Request.URL, resp.StatusCode())
    return nil
})
```

**Axios** — Promise 链（request LIFO, response FIFO）
```js
axios.interceptors.request.use(config => {
    config.headers.Authorization = `Bearer ${token}`;
    return config;
});
```

**httpx** — event_hooks（简单钩子列表）
```python
client = httpx.Client(event_hooks={
    'request': [log_request],
    'response': [raise_on_4xx_5xx]
})
```

**决策点**：我们复用 `reqwest-middleware`（DelegatingHandler 模式），同时提供简单的 `RequestInterceptor` trait。

---

### 2.3 重试机制

**OpenFeign** — `Retryer` 接口，有状态（每次请求 clone 新实例）
```java
// 默认重试 IOException + RetryableException
// 自定义：
Feign.builder().retryer(new Retryer.Default(100, 1000, 3))
```

**go-resty** — 内置，配置式
```go
client.SetRetryCount(3).
    SetRetryWaitTime(5 * time.Second).
    SetRetryMaxWaitTime(20 * time.Second).
    AddRetryCondition(func(r *resty.Response, err error) bool {
        return r.StatusCode() == 429
    })
```

**Refit / Axios / httpx** — 不内置，依赖第三方（Polly / axios-retry / tenacity）

**决策点**：我们通过 feature `retry` 引入 `reqwest-retry`，声明式配置通过宏属性。

---

### 2.4 认证

**OpenFeign** — `BasicAuthRequestInterceptor` + 自定义拦截器
```java
Feign.builder().requestInterceptor(new BasicAuthRequestInterceptor("user", "pass"))
```

**Refit** — `[Authorize]` 属性 + `AuthorizationHeaderValueGetter`
```csharp
[Get("/users/me")]
Task<User> GetMe([Authorize("Bearer")] string token);

// 或全局
settings.AuthorizationHeaderValueGetter = (req, ct) => GetTokenAsync();
```

**go-resty** — 内置方法
```go
client.SetBasicAuth("user", "pass")
client.SetAuthToken("my-jwt-token")
client.SetAuthScheme("OAuth")  // 自定义 scheme
```

**httpx** — Auth 类（支持多轮认证流）
```python
class BearerAuth(httpx.Auth):
    def auth_flow(self, request):
        request.headers['Authorization'] = f'Bearer {self.token}'
        yield request
```

**决策点**：提供 `#[header("Authorization")]` 参数级 + `RequestInterceptor` 全局级。

---

### 2.5 错误处理

**OpenFeign** — `ErrorDecoder` 接口，非 2xx 响应走 decode
```java
class MyErrorDecoder implements ErrorDecoder {
    public Exception decode(String methodKey, Response response) {
        if (response.status() == 401) return new RetryableException(...);
        return new Default().decode(methodKey, response);
    }
}
```

**Refit** — `ApiException` / `ApiResponse<T>`（不抛异常模式）
```csharp
// 抛异常模式
try { var user = await api.GetUser(1); }
catch (ApiException ex) { /* ex.StatusCode, ex.Content */ }

// 不抛异常模式
var resp = await api.GetUser(1);  // Task<ApiResponse<User>>
if (!resp.IsSuccessStatusCode) { /* resp.Error */ }
```

**go-resty** — `.SetResult` / `.SetError` 自动反序列化
```go
resp, _ := client.R().
    SetResult(&AuthSuccess{}).  // 2xx 自动反序列化到这里
    SetError(&AuthError{}).     // 非 2xx 自动反序列化到这里
    Post("/login")
```

**Axios** — `AxiosError` 结构化错误
```js
catch (error) {
    error.response  // 服务器返回了响应
    error.request   // 请求已发出但无响应
    error.code      // ERR_NETWORK, ETIMEDOUT 等
}
```

**决策点**：我们用 `feign::Error` 枚举 + `feign::Result<T>`。支持 `Option<T>` 语义（404 → None）。

---

### 2.6 返回类型

**OpenFeign**：`void` / `String` / `byte[]` / `Response` / `Stream` / `CompletableFuture<T>` / 任意 POJO

**Refit**：`Task` / `Task<T>` / `Task<string>` / `Task<HttpResponseMessage>` / `Task<ApiResponse<T>>` / `IObservable<T>`

**我们的设计**：
```rust
feign::Result<T>          // 标准返回
feign::Result<Option<T>>  // 404 → None
feign::Result<()>         // 忽略响应体
feign::Result<Vec<u8>>    // 原始字节
feign::Result<String>     // 原始文本
```

---

### 2.7 Query 参数处理

**OpenFeign** — `@QueryMap` 支持 Map 和 POJO
```java
@RequestLine("GET /find")
V find(@QueryMap CustomPojo pojo);  // POJO 字段自动展开
```

**Refit** — 对象属性自动展开 + CollectionFormat
```csharp
[Get("/users")]
Task Search([Query(CollectionFormat.Multi)] int[] ages);
// => /users?ages=10&ages=20&ages=30

[Get("/users")]
Task Search([Query(CollectionFormat.Csv)] int[] ages);
// => /users?ages=10,20,30
```

**go-resty** — 手动设置
```go
client.R().SetQueryParams(map[string]string{"page": "1", "size": "10"})
client.R().SetQueryString("page=1&size=10")
```

**决策点**：`#[query]` 标注的 struct 自动展开字段为 query 参数（需实现 Serialize）。

---

### 2.8 文件上传

**OpenFeign** — FormEncoder + `@Param`
```java
@RequestLine("POST /upload")
@Headers("Content-Type: multipart/form-data")
void upload(@Param("file") File file);
```

**Refit** — `[Multipart]` + `StreamPart`
```csharp
[Multipart]
[Post("/upload")]
Task Upload([AliasAs("file")] StreamPart stream);
```

**go-resty** — 内置方法
```go
client.R().SetFile("photo", "/path/to/photo.jpg").Post("/upload")
client.R().SetFileReader("file", "name.png", reader).Post("/upload")
```

**决策点**：Phase 1 不做，后续通过 `#[multipart]` + `FilePart` 类型支持。

---

### 2.9 接口继承/组合

**OpenFeign** — 支持接口继承
```java
interface BaseApi<T> {
    @RequestLine("GET /api/{id}") T get(@Param("id") String id);
    @RequestLine("GET /api") List<T> list();
}
interface UserApi extends BaseApi<User> {}
```

**Refit** — 支持接口继承，Headers 也继承
```csharp
public interface IBaseService {
    [Get("/resources/{id}")]
    Task<Resource> GetResource(string id);
}
public interface IDerivedService : IBaseService {
    [Delete("/resources/{id}")]
    Task DeleteResource(string id);
}
```

**决策点**：Rust trait 天然支持继承（supertrait），但 proc macro 处理有复杂度。Phase 1 不做。

---

### 2.10 服务发现 / 动态 URL

**OpenFeign** — `Target<T>` 接口
```java
static class DynamicTarget<T> implements Target<T> {
    public Request apply(RequestTemplate input) {
        input.insert(0, discoveryClient.getUrl(serviceName));
        return input.request();
    }
}
```

**其他框架** — 均不内置，需外部实现

**决策点**：这是我们的核心差异化。`ServiceResolver` trait + feature 控制。

---

### 2.11 Mock / 测试支持

**Refit** — 通过 DelegatingHandler mock
**httpx** — `MockTransport` / `ASGITransport`
```python
transport = httpx.MockTransport(lambda req: httpx.Response(200, json={'ok': True}))
client = httpx.Client(transport=transport)
```

**Axios** — adapter 替换
```js
const mock = (config) => Promise.resolve({ data: {}, status: 200 });
axios.create({ adapter: mock });
```

**决策点**：我们的 `Transport` trait 天然可 mock。测试时注入 mock Transport 即可。

---

## 三、功能优先级建议

### P0 — MVP 必须有

| 功能 | 理由 |
|------|------|
| 声明式 trait 宏 | 核心价值，生态空白 |
| 路径参数 `#[path]` | 基础能力 |
| Query 参数 `#[query]` | 基础能力 |
| Body 序列化 `#[body]` | 基础能力 |
| Header `#[header]` | 基础能力 |
| `feign::Result<T>` | 统一错误处理 |
| 命令式 Client | 不用宏也能用 |
| JSON 编解码 | 默认 |

### P1 — 生产必须有

| 功能 | 理由 |
|------|------|
| 重试（feature） | 生产环境必需 |
| 超时 | 生产环境必需 |
| 拦截器 | 认证、日志等横切关注点 |
| 服务发现 | 微服务核心差异化 |
| 连接池 | reqwest 自带 |
| `Option<T>` 404 语义 | 实用 |
| `RequestParam` derive | 复杂请求 |

### P2 — 锦上添花

| 功能 | 理由 |
|------|------|
| 熔断器（feature） | 高可用 |
| 链路追踪（feature） | 可观测性 |
| 文件上传 `#[multipart]` | 特定场景 |
| 编解码可插拔 | XML/Protobuf 场景 |
| 接口继承 | 代码复用 |
| 日志级别控制 | 调试 |

### P3 — 未来考虑

| 功能 | 理由 |
|------|------|
| HTTP/2 | reqwest 支持 |
| 代理 | reqwest 支持 |
| 进度追踪 | 大文件场景 |
| 取消请求 | 长时间请求 |
| Mock Transport | 测试 |
| 负载均衡策略 | 多实例 |