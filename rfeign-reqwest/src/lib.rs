use async_trait::async_trait;
use bytes::Bytes;
use http::{Request, Response};

use rfeign_core::error::{Error, Result};
use rfeign_core::transport::MultipartField;
use rfeign_core::transport::Transport;

pub struct ReqwestTransport {
    inner: InnerClient,
}

enum InnerClient {
    Plain(reqwest::Client),
    #[cfg(feature = "middleware")]
    WithMiddleware(reqwest_middleware::ClientWithMiddleware),
}

impl ReqwestTransport {
    pub fn new() -> Self {
        Self {
            inner: InnerClient::Plain(reqwest::Client::new()),
        }
    }

    pub fn with_client(client: reqwest::Client) -> Self {
        Self {
            inner: InnerClient::Plain(client),
        }
    }

    pub fn builder() -> ReqwestTransportBuilder {
        ReqwestTransportBuilder {
            client: reqwest::Client::new(),
            #[cfg(feature = "middleware")]
            middlewares: Vec::new(),
        }
    }
}

impl Default for ReqwestTransport {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ReqwestTransportBuilder {
    client: reqwest::Client,
    #[cfg(feature = "middleware")]
    middlewares: Vec<std::sync::Arc<dyn reqwest_middleware::Middleware>>,
}

impl ReqwestTransportBuilder {
    pub fn client(mut self, client: reqwest::Client) -> Self {
        self.client = client;
        self
    }

    #[cfg(feature = "middleware")]
    pub fn with(mut self, middleware: impl reqwest_middleware::Middleware) -> Self {
        self.middlewares.push(std::sync::Arc::new(middleware));
        self
    }

    #[cfg(feature = "retry")]
    pub fn retry(self, max_retries: u32) -> Self {
        use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
        let policy = ExponentialBackoff::builder().build_with_max_retries(max_retries);
        self.with(RetryTransientMiddleware::new_with_policy(policy))
    }

    #[cfg(feature = "tracing")]
    pub fn tracing(self) -> Self {
        self.with(reqwest_tracing::TracingMiddleware::default())
    }

    pub fn build(self) -> ReqwestTransport {
        #[cfg(feature = "middleware")]
        {
            if !self.middlewares.is_empty() {
                let mut client_builder =
                    reqwest_middleware::ClientBuilder::new(self.client);
                for mw in self.middlewares {
                    client_builder = client_builder.with_arc(mw);
                }
                return ReqwestTransport {
                    inner: InnerClient::WithMiddleware(client_builder.build()),
                };
            }
        }
        ReqwestTransport {
            inner: InnerClient::Plain(self.client),
        }
    }
}

#[async_trait]
impl Transport for ReqwestTransport {
    async fn send(&self, request: Request<Bytes>) -> Result<Response<Bytes>> {
        let (parts, body) = request.into_parts();
        let url = parts.uri.to_string();
        let method = match reqwest::Method::from_bytes(parts.method.as_str().as_bytes()) {
            Ok(m) => m,
            Err(_) => reqwest::Method::GET,
        };

        let resp = match &self.inner {
            InnerClient::Plain(client) => {
                let mut req = client.request(method, &url);
                for (name, value) in &parts.headers {
                    req = req.header(name.as_str(), value.as_bytes());
                }
                req.body(body)
                    .send()
                    .await
                    .map_err(|e| Error::Transport(Box::new(e)))?
            }
            #[cfg(feature = "middleware")]
            InnerClient::WithMiddleware(client) => {
                let mut req = client.request(method, &url);
                for (name, value) in &parts.headers {
                    req = req.header(name.as_str(), value.as_bytes());
                }
                req.body(body)
                    .send()
                    .await
                    .map_err(|e| Error::Transport(Box::new(e)))?
            }
        };

        let status = resp.status();
        let headers = resp.headers().clone();
        let resp_body = resp
            .bytes()
            .await
            .map_err(|e| Error::Transport(Box::new(e)))?;

        let mut response = Response::builder().status(status);
        if let Some(h) = response.headers_mut() {
            *h = headers;
        }
        match response.body(resp_body) {
            Ok(r) => Ok(r),
            Err(e) => Err(Error::Other(e.to_string())),
        }
    }

    async fn send_multipart(
        &self,
        request: Request<Bytes>,
        parts: Vec<(String, MultipartField)>,
    ) -> Result<Response<Bytes>> {
        let (req_parts, _) = request.into_parts();
        let url = req_parts.uri.to_string();
        let method = match reqwest::Method::from_bytes(req_parts.method.as_str().as_bytes()) {
            Ok(m) => m,
            Err(_) => reqwest::Method::POST,
        };

        let mut form = reqwest::multipart::Form::new();
        for (name, field) in parts {
            match field {
                MultipartField::Text(text) => {
                    form = form.text(name, text);
                }
                MultipartField::File(part) => {
                    let base = reqwest::multipart::Part::bytes(part.data.to_vec())
                        .file_name(part.filename);
                    let rpart = match base.mime_str(&part.content_type) {
                        Ok(p) => p,
                        Err(_) => reqwest::multipart::Part::bytes(Vec::new()),
                    };
                    form = form.part(name, rpart);
                }
            }
        }

        let mut req = match &self.inner {
            InnerClient::Plain(c) => c.request(method, &url),
            #[cfg(feature = "middleware")]
            InnerClient::WithMiddleware(_) => {
                reqwest::Client::new().request(method, &url)
            }
        };
        for (name, value) in &req_parts.headers {
            req = req.header(name.as_str(), value.as_bytes());
        }

        let resp = req
            .multipart(form)
            .send()
            .await
            .map_err(|e| Error::Transport(Box::new(e)))?;

        let status = resp.status();
        let headers = resp.headers().clone();
        let resp_body = resp
            .bytes()
            .await
            .map_err(|e| Error::Transport(Box::new(e)))?;

        let mut response = Response::builder().status(status);
        if let Some(h) = response.headers_mut() {
            *h = headers;
        }
        match response.body(resp_body) {
            Ok(r) => Ok(r),
            Err(e) => Err(Error::Other(e.to_string())),
        }
    }
}
