use async_trait::async_trait;
use bytes::Bytes;
use http::{Request, Response};

use crate::error::Result;
use crate::part::Part;

#[async_trait]
pub trait Transport: Send + Sync + 'static {
    async fn send(&self, request: Request<Bytes>) -> Result<Response<Bytes>>;

    async fn send_multipart(
        &self,
        request: Request<Bytes>,
        parts: Vec<(String, MultipartField)>,
    ) -> Result<Response<Bytes>> {
        let _ = parts;
        self.send(request).await
    }
}

pub enum MultipartField {
    Text(String),
    File(Part),
}
