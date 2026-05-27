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

#[async_trait]
impl Auth for Box<dyn Auth> {
    async fn authenticate(&self, request: &mut Request<Bytes>) -> Result<()> {
        (**self).authenticate(request).await
    }
}

#[async_trait]
impl Auth for Arc<dyn Auth> {
    async fn authenticate(&self, request: &mut Request<Bytes>) -> Result<()> {
        (**self).authenticate(request).await
    }
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

pub trait TokenLike: Send {
    fn access_token(&self) -> &str;
    fn refresh_token(&self) -> Option<&str> { None }
    fn expires_in_secs(&self) -> u64 { 3600 }
}

#[async_trait]
pub trait TokenProvider: Send + Sync + 'static {
    type Token: TokenLike;
    type Refresh: TokenLike;

    async fn fetch_token(&self) -> Result<Self::Token>;
    async fn refresh(&self, refresh_token: &str) -> Result<Self::Refresh>;
}

pub struct TokenResponse {
    pub access_token: String,
    pub expires_in_secs: u64,
}

impl TokenLike for TokenResponse {
    fn access_token(&self) -> &str { &self.access_token }
    fn expires_in_secs(&self) -> u64 { self.expires_in_secs }
}

pub struct DynamicAuth<P: TokenProvider> {
    provider: P,
    header_name: String,
    token_type: String,
    cache: Arc<RwLock<CachedToken>>,
}

struct CachedToken {
    token: String,
    refresh_token: Option<String>,
    expires_at: Instant,
}

impl<P: TokenProvider> DynamicAuth<P> {
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            header_name: "Authorization".to_string(),
            token_type: "Bearer".to_string(),
            cache: Arc::new(RwLock::new(CachedToken {
                token: String::new(),
                refresh_token: None,
                expires_at: Instant::now(),
            })),
        }
    }

    pub fn header_name(mut self, name: impl Into<String>) -> Self {
        self.header_name = name.into();
        self
    }

    pub fn token_type(mut self, t: impl Into<String>) -> Self {
        self.token_type = t.into();
        self
    }

    async fn get_token(&self) -> Result<String> {
        {
            let cached = self.cache.read().await;
            if !cached.token.is_empty() && Instant::now() < cached.expires_at {
                return Ok(cached.token.clone());
            }
        }

        let mut cached = self.cache.write().await;
        if !cached.token.is_empty() && Instant::now() < cached.expires_at {
            return Ok(cached.token.clone());
        }

        if let Some(ref rt) = cached.refresh_token {
            let resp = self.provider.refresh(rt).await?;
            cached.token = resp.access_token().to_string();
            cached.refresh_token = resp.refresh_token().map(|s| s.to_string());
            cached.expires_at = Instant::now()
                + std::time::Duration::from_secs(
                    resp.expires_in_secs().saturating_sub(30),
                );
        } else {
            let resp = self.provider.fetch_token().await?;
            cached.token = resp.access_token().to_string();
            cached.refresh_token = resp.refresh_token().map(|s| s.to_string());
            cached.expires_at = Instant::now()
                + std::time::Duration::from_secs(
                    resp.expires_in_secs().saturating_sub(30),
                );
        }

        Ok(cached.token.clone())
    }
}

#[async_trait]
impl<P: TokenProvider> Auth for DynamicAuth<P> {
    async fn authenticate(&self, request: &mut Request<Bytes>) -> Result<()> {
        let token = self.get_token().await?;
        let header_value = if self.token_type.is_empty() {
            token
        } else {
            format!("{} {}", self.token_type, token)
        };
        let value: http::HeaderValue = header_value.parse()
            .map_err(|e: http::header::InvalidHeaderValue| {
                crate::error::Error::Other(e.to_string())
            })?;
        request.headers_mut().insert(
            http::header::HeaderName::from_bytes(self.header_name.as_bytes())
                .unwrap_or(http::header::AUTHORIZATION),
            value,
        );
        Ok(())
    }
}
