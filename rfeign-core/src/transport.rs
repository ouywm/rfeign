use async_trait::async_trait;
use bytes::Bytes;
use http::{Request, Response};

use crate::error::Result;
use crate::part::Part;
use crate::stream::ByteStream;

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

    async fn send_streaming(
        &self,
        request: Request<Bytes>,
    ) -> Result<StreamingResponse> {
        let resp = self.send(request).await?;
        let status = resp.status().as_u16();
        let headers = resp.headers().clone();
        let body = resp.into_body();
        let stream: ByteStream = Box::pin(futures_util::stream::once(async { Ok(body) }));
        Ok(StreamingResponse {
            status,
            headers,
            body: stream,
        })
    }
}

pub enum MultipartField {
    Text(String),
    File(Part),
}

pub struct StreamingResponse {
    pub status: u16,
    pub headers: http::HeaderMap,
    pub body: ByteStream,
}
