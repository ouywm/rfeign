use async_trait::async_trait;
use bytes::Bytes;
use http::{Request, Response};

use feignx_core::error::{Error, Result};
use feignx_core::transport::Transport;

pub struct ReqwestTransport {
    client: reqwest::Client,
}

impl ReqwestTransport {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub fn with_client(client: reqwest::Client) -> Self {
        Self { client }
    }
}

impl Default for ReqwestTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Transport for ReqwestTransport {
    async fn send(&self, request: Request<Bytes>) -> Result<Response<Bytes>> {
        let (parts, body) = request.into_parts();
        let url = parts.uri.to_string();
        let method = parts.method.clone();

        let mut req_builder = self.client.request(method, &url);
        for (name, value) in &parts.headers {
            req_builder = req_builder.header(name, value);
        }
        req_builder = req_builder.body(body);

        let resp = req_builder
            .send()
            .await
            .map_err(|e| Error::Transport(Box::new(e)))?;

        let status = resp.status();
        let headers = resp.headers().clone();
        let body = resp
            .bytes()
            .await
            .map_err(|e| Error::Transport(Box::new(e)))?;

        let mut response = Response::builder().status(status);
        if let Some(h) = response.headers_mut() {
            *h = headers;
        }
        let response = match response.body(body) {
            Ok(r) => r,
            Err(e) => return Err(Error::Other(e.to_string())),
        };

        Ok(response)
    }
}