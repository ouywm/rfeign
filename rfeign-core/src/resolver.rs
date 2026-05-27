use async_trait::async_trait;

use crate::error::Result;

#[async_trait]
pub trait UrlResolver: Send + Sync + 'static {
    async fn resolve(&self, service_name: &str) -> Result<String>;
}

#[derive(Clone)]
pub struct StaticUrl(pub String);

#[async_trait]
impl UrlResolver for StaticUrl {
    async fn resolve(&self, _service_name: &str) -> Result<String> {
        Ok(self.0.clone())
    }
}