use async_trait::async_trait;
use bytes::Bytes;
use http::{Request, Response};

use crate::error::Result;

#[async_trait]
pub trait Transport: Send + Sync + 'static {
    async fn send(&self, request: Request<Bytes>) -> Result<Response<Bytes>>;
}