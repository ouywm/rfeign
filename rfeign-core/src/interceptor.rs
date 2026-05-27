use async_trait::async_trait;
use bytes::Bytes;
use http::{Request, Response};

use crate::error::Result;

#[async_trait]
pub trait RequestInterceptor: Send + Sync + 'static {
    async fn intercept(&self, request: Request<Bytes>) -> Result<Request<Bytes>>;
}

#[async_trait]
pub trait ResponseInterceptor: Send + Sync + 'static {
    async fn intercept(&self, response: Response<Bytes>) -> Result<Response<Bytes>>;
}