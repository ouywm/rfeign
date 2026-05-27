# 日志

rfeign 提供内置的请求/响应日志中间件，支持四个级别的日志输出。

## LogLevel 四级

| 级别 | 输出内容 |
|------|----------|
| `None` | 不输出任何日志（默认） |
| `Basic` | 请求方法、URL、响应状态码、耗时 |
| `Headers` | Basic + 请求/响应 headers |
| `Full` | Headers + 请求/响应 body |

## LoggingMiddleware

作为中间件添加到 Client：

```rust
use rfeign::{ClientBuilder, ReqwestTransport, LogLevel, LoggingMiddleware};

let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("https://api.example.com")
    .middleware(LoggingMiddleware::new(LogLevel::Headers))
    .build();
```

也可以通过 `ClientBuilder::log_level()` 设置（用于声明式客户端内部日志）：

```rust
let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("https://api.example.com")
    .log_level(LogLevel::Full)
    .build();
```

### 输出示例

`LogLevel::Basic`：
```
--> GET https://api.example.com/users/1
<-- 200 (45ms) https://api.example.com/users/1
```

`LogLevel::Headers`：
```
--> GET https://api.example.com/users/1
    accept: application/json
    authorization: Bearer ***
<-- 200 (45ms) https://api.example.com/users/1
    content-type: application/json
    content-length: 128
```

## .to_curl() 调试

`RequestBuilder` 提供 `.to_curl()` 方法，生成等效的 cURL 命令：

```rust
let req = client.post("/users")
    .header("X-App", "test")
    .json(&user)?;

println!("{}", req.to_curl());
```

输出：
```
curl -X POST \
  -H 'content-type: application/json' \
  -H 'x-app: test' \
  '<base_url>/users' \
  -d '{"name":"alice","email":"alice@example.com"}'
```

适合在开发阶段快速复制到终端验证请求。
