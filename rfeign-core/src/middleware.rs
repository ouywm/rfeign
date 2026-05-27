use async_trait::async_trait;
use bytes::Bytes;
use http::{Request, Response};
use std::sync::Arc;

use crate::error::Result;
use crate::transport::Transport;

#[async_trait]
pub trait Middleware: Send + Sync + 'static {
    async fn handle(
        &self,
        request: Request<Bytes>,
        next: Next<'_>,
    ) -> Result<Response<Bytes>>;
}

#[derive(Clone)]
pub struct Next<'a> {
    transport: &'a dyn Transport,
    middlewares: &'a [Arc<dyn Middleware>],
    index: usize,
}

impl<'a> Next<'a> {
    pub fn new(
        transport: &'a dyn Transport,
        middlewares: &'a [Arc<dyn Middleware>],
    ) -> Self {
        Self {
            transport,
            middlewares,
            index: 0,
        }
    }

    pub async fn call(mut self, request: Request<Bytes>) -> Result<Response<Bytes>> {
        if self.index < self.middlewares.len() {
            let mw = &self.middlewares[self.index];
            self.index += 1;
            mw.handle(request, self).await
        } else {
            self.transport.send(request).await
        }
    }
}