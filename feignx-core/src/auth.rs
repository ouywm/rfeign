use async_trait::async_trait;
use bytes::Bytes;
use http::Request;

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
        request.headers_mut().insert(
            "Authorization",
            format!("Bearer {}", self.token).parse().unwrap(),
        );
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
        request.headers_mut().insert(
            "Authorization",
            format!("Basic {}", credentials).parse().unwrap(),
        );
        Ok(())
    }
}