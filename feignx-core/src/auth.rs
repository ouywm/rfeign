use async_trait::async_trait;
use bytes::Bytes;
use http::Request;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

use crate::error::Result;

#[async_trait]
pub trait Auth: Send + Sync + 'static {
    async fn authenticate(&self, request: &mut Request<Bytes>) -> Result<()>;
}

pub struct BearerAuth {
    token: String,
}

impl BearerAuth {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

#[async_trait]
impl Auth for BearerAuth {
    async fn authenticate(&self, request: &mut Request<Bytes>) -> Result<()> {
        let value: http::HeaderValue = match format!("Bearer {}", self.token).parse() {
            Ok(v) => v,
            Err(e) => return Err(crate::error::Error::Other(e.to_string())),
        };
        request.headers_mut().insert("Authorization", value);
        Ok(())
    }
}

pub struct BasicAuth {
    username: String,
    password: String,
}

impl BasicAuth {
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }
}

#[async_trait]
impl Auth for BasicAuth {
    async fn authenticate(&self, request: &mut Request<Bytes>) -> Result<()> {
        use base64::Engine;
        let credentials = base64::engine::general_purpose::STANDARD
            .encode(format!("{}:{}", self.username, self.password));
        let value: http::HeaderValue = match format!("Basic {}", credentials).parse() {
            Ok(v) => v,
            Err(e) => return Err(crate::error::Error::Other(e.to_string())),
        };
        request.headers_mut().insert("Authorization", value);
        Ok(())
    }
}

#[async_trait]
pub trait TokenProvider: Send + Sync + 'static {
    async fn fetch_token(&self) -> Result<TokenResponse>;
}

pub struct TokenResponse {
    pub access_token: String,
    pub expires_in_secs: u64,
}

pub struct DynamicAuth {
    provider: Box<dyn TokenProvider>,
    cache: Arc<RwLock<CachedToken>>,
}

struct CachedToken {
    token: String,
    expires_at: Instant,
}

impl DynamicAuth {
    pub fn new(provider: impl TokenProvider) -> Self {
        Self {
            provider: Box::new(provider),
            cache: Arc::new(RwLock::new(CachedToken {
                token: String::new(),
                expires_at: Instant::now(),
            })),
        }
    }

    async fn get_token(&self) -> Result<String> {
        {
            let cached = self.cache.read().await;
            if !cached.token.is_empty() && Instant::now() < cached.expires_at {
                return Ok(cached.token.clone());
            }
        }

        let resp = self.provider.fetch_token().await?;
        let mut cached = self.cache.write().await;
        cached.token = resp.access_token.clone();
        cached.expires_at = Instant::now()
            + std::time::Duration::from_secs(resp.expires_in_secs.saturating_sub(30));
        Ok(resp.access_token)
    }
}

#[async_trait]
impl Auth for DynamicAuth {
    async fn authenticate(&self, request: &mut Request<Bytes>) -> Result<()> {
        let token = self.get_token().await?;
        let value: http::HeaderValue = match format!("Bearer {}", token).parse() {
            Ok(v) => v,
            Err(e) => return Err(crate::error::Error::Other(e.to_string())),
        };
        request.headers_mut().insert("Authorization", value);
        Ok(())
    }
}