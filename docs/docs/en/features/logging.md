# Logging

rfeign provides built-in request/response logging through `LoggingMiddleware`.

## Log Levels

| Level     | Output                                      |
|-----------|---------------------------------------------|
| `None`    | No logging (default)                        |
| `Basic`   | Method, URL, status code, elapsed time      |
| `Headers` | Basic + request/response headers            |
| `Full`    | Headers + request/response bodies           |

## LoggingMiddleware

Add `LoggingMiddleware` to the client to log all requests:

```rust
use rfeign::{ClientBuilder, ReqwestTransport, LogLevel, LoggingMiddleware};

let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("https://api.example.com")
    .middleware(LoggingMiddleware::new(LogLevel::Full))
    .log_level(LogLevel::Full)
    .build();
```

### Output Examples

**Basic level:**
```
--> GET https://api.example.com/users/1
<-- 200 (45ms) https://api.example.com/users/1
```

**Headers level:**
```
--> GET https://api.example.com/users/1
    accept: application/json
    authorization: Bearer ***
<-- 200 (45ms) https://api.example.com/users/1
    content-type: application/json
    content-length: 128
```

**Full level** adds the request and response body content.

## .to_curl() Debugging

Generate a cURL-equivalent command from any request builder:

```rust
let req = client.post("/users")
    .header("Content-Type", "application/json")
    .query_pair("dry_run", "true")
    .json(&user)?;

println!("{}", req.to_curl());
```

Output:
```
curl -X POST \
  -H 'content-type: application/json' \
  '<base_url>/users?dry_run=true' \
  -d '{"name":"Alice","email":"alice@example.com"}'
```

This is useful for reproducing requests outside your application or sharing with teammates.
