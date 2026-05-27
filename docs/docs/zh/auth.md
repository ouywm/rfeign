# 认证

rfeign 内置多种认证方式，通过 `Auth` trait 实现可扩展的认证机制。

## BearerAuth

最常用的 Token 认证方式：

```rust
use rfeign::{ClientBuilder, ReqwestTransport};

let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("https://api.example.com")
    .bearer_auth("your-access-token")
    .build();

// 所有请求自动携带: Authorization: Bearer your-access-token
```

## BasicAuth

HTTP Basic 认证：

```rust
let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("https://api.example.com")
    .basic_auth("username", "password")
    .build();

// 所有请求自动携带: Authorization: Basic dXNlcm5hbWU6cGFzc3dvcmQ=
```

## DynamicAuth + TokenProvider

对于需要自动刷新的 Token（如 OAuth2），使用 `DynamicAuth`：

```rust
use rfeign::auth::{DynamicAuth, TokenProvider, TokenResponse};
use rfeign::{ClientBuilder, ReqwestTransport};
use async_trait::async_trait;

struct MyTokenProvider;

#[async_trait]
impl TokenProvider for MyTokenProvider {
    async fn fetch_token(&self) -> rfeign::Result<TokenResponse> {
        // 调用认证服务获取 token
        Ok(TokenResponse {
            access_token: "new-token".to_string(),
            expires_in_secs: 3600,  // 1 小时后过期
        })
    }
}

let auth = DynamicAuth::new(MyTokenProvider);

let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("https://api.example.com")
    .auth(auth)
    .build();
```

`DynamicAuth` 的行为：

- 首次请求时调用 `fetch_token()` 获取 token
- 自动缓存 token，在过期前 30 秒刷新
- 线程安全，多个请求共享同一个 token 缓存
- 请求头格式：`Authorization: Bearer <token>`

## 自定义 Auth

实现 `Auth` trait 可以自定义任意认证逻辑：

```rust
use rfeign::auth::Auth;
use async_trait::async_trait;
use bytes::Bytes;
use http::Request;

struct ApiKeyAuth {
    key: String,
}

#[async_trait]
impl Auth for ApiKeyAuth {
    async fn authenticate(&self, request: &mut Request<Bytes>) -> rfeign::Result<()> {
        request.headers_mut().insert(
            "X-API-Key",
            self.key.parse().unwrap(),
        );
        Ok(())
    }
}

let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("https://api.example.com")
    .auth(ApiKeyAuth { key: "my-key".into() })
    .build();
```
