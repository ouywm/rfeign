use async_trait::async_trait;
use bytes::Bytes;
use http::{Request, Response};
use std::sync::Arc;

use crate::error::Result;

#[async_trait]
pub trait RequestInterceptor: Send + Sync + 'static {
    async fn intercept(&self, request: Request<Bytes>) -> Result<Request<Bytes>>;
}

#[async_trait]
impl RequestInterceptor for Box<dyn RequestInterceptor> {
    async fn intercept(&self, request: Request<Bytes>) -> Result<Request<Bytes>> {
        (**self).intercept(request).await
    }
}

#[async_trait]
impl RequestInterceptor for Arc<dyn RequestInterceptor> {
    async fn intercept(&self, request: Request<Bytes>) -> Result<Request<Bytes>> {
        (**self).intercept(request).await
    }
}

#[async_trait]
pub trait ResponseInterceptor: Send + Sync + 'static {
    async fn intercept(&self, response: Response<Bytes>) -> Result<Response<Bytes>>;
}

#[async_trait]
impl ResponseInterceptor for Box<dyn ResponseInterceptor> {
    async fn intercept(&self, response: Response<Bytes>) -> Result<Response<Bytes>> {
        (**self).intercept(response).await
    }
}

#[async_trait]
impl ResponseInterceptor for Arc<dyn ResponseInterceptor> {
    async fn intercept(&self, response: Response<Bytes>) -> Result<Response<Bytes>> {
        (**self).intercept(response).await
    }
}