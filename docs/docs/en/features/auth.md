# Authentication

rfeign supports static tokens, basic auth, and dynamic token refresh.

## Bearer Token

```rust
use rfeign::{ClientBuilder, ReqwestTransport};

let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("https://api.example.com")
    .bearer_auth("my-access-token")
    .build();
```

Or construct it explicitly:

```rust
use rfeign::auth::BearerAuth;

let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("https://api.example.com")
    .auth(BearerAuth::new("my-access-token"))
    .build();
```

## Basic Auth

```rust
let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("https://api.example.com")
    .basic_auth("username", "password")
    .build();
```

Or explicitly:

```rust
use rfeign::auth::BasicAuth;

let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("https://api.example.com")
    .auth(BasicAuth::new("username", "password"))
    .build();
```

## Dynamic Auth (Auto Refresh)

For OAuth2 or other token-based flows where tokens expire, implement the `TokenProvider` trait:

```rust
use rfeign::auth::{DynamicAuth, TokenProvider, TokenResponse};
use rfeign::error::Result;
use async_trait::async_trait;

struct MyTokenProvider {
    client_id: String,
    client_secret: String,
}

#[async_trait]
impl TokenProvider for MyTokenProvider {
    async fn fetch_token(&self) -> Result<TokenResponse> {
        // Call your OAuth2 token endpoint here
        Ok(TokenResponse {
            access_token: "new-token".into(),
            expires_in_secs: 3600,
        })
    }
}
```

Then use it with `DynamicAuth`:

```rust
let provider = MyTokenProvider {
    client_id: "my-app".into(),
    client_secret: "secret".into(),
};

let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("https://api.example.com")
    .auth(DynamicAuth::new(provider))
    .build();
```

`DynamicAuth` caches the token and automatically refreshes it 30 seconds before expiry. The token is shared safely across concurrent requests using `RwLock`.

## Custom Auth

Implement the `Auth` trait for any custom authentication scheme:

```rust
use rfeign::auth::Auth;
use rfeign::error::Result;
use async_trait::async_trait;
use bytes::Bytes;
use http::Request;

struct ApiKeyAuth {
    key: String,
}

#[async_trait]
impl Auth for ApiKeyAuth {
    async fn authenticate(&self, request: &mut Request<Bytes>) -> Result<()> {
        request.headers_mut().insert(
            "X-API-Key",
            self.key.parse().unwrap(),
        );
        Ok(())
    }
}

let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("https://api.example.com")
    .auth(ApiKeyAuth { key: "sk-123".into() })
    .build();
```
