use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use http::{Request, Response};

use crate::auth::{Auth, BasicAuth, BearerAuth};
use crate::codec::{Decoder, Encoder, JsonCodec};
use crate::error::Result;
use crate::error_decoder::{DefaultErrorDecoder, ErrorDecoder};
use crate::interceptor::{RequestInterceptor, ResponseInterceptor};
use crate::log::LogLevel;
use crate::middleware::{Middleware, Next};
use crate::resolver::{StaticUrl, UrlResolver};
use crate::timeout::Timeout;
use crate::transport::Transport;

#[derive(Clone)]
pub struct Client {
    transport: Arc<dyn Transport>,
    url_resolver: Arc<dyn UrlResolver>,
    service_name: String,
    auth: Option<Arc<dyn Auth>>,
    error_decoder: Arc<dyn ErrorDecoder>,
    request_interceptors: Vec<Arc<dyn RequestInterceptor>>,
    response_interceptors: Vec<Arc<dyn ResponseInterceptor>>,
    middlewares: Vec<Arc<dyn Middleware>>,
    encoder: Arc<dyn Encoder>,
    decoder: Arc<dyn Decoder>,
    timeout: Timeout,
    log_level: LogLevel,
    success_status: fn(u16) -> bool,
}

impl Client {
    pub fn builder(transport: impl Transport) -> ClientBuilder {
        ClientBuilder::new(transport)
    }

    pub fn transport(&self) -> &dyn Transport {
        self.transport.as_ref()
    }

    pub fn url_resolver(&self) -> &dyn UrlResolver {
        self.url_resolver.as_ref()
    }

    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    pub fn with_service_name(mut self, name: impl Into<String>) -> Self {
        self.service_name = name.into();
        self
    }

    pub async fn resolve_base_url(&self) -> Result<String> {
        self.url_resolver.resolve(&self.service_name).await
    }

    pub fn encoder(&self) -> &dyn Encoder {
        self.encoder.as_ref()
    }

    pub fn decoder(&self) -> &dyn Decoder {
        self.decoder.as_ref()
    }

    pub fn decode_response<T: serde::de::DeserializeOwned>(&self, body: &[u8]) -> Result<T> {
        let value = self.decoder.decode(body)?;
        crate::codec::deserialize(value)
    }

    pub fn encode_body(&self, value: &dyn erased_serde::Serialize) -> Result<Bytes> {
        self.encoder.encode(value)
    }

    pub fn is_success(&self, status: u16) -> bool {
        (self.success_status)(status)
    }

    pub fn error_decoder(&self) -> &dyn ErrorDecoder {
        self.error_decoder.as_ref()
    }

    pub fn log_level(&self) -> LogLevel {
        self.log_level
    }

    pub fn timeout(&self) -> &Timeout {
        &self.timeout
    }

    pub async fn execute(&self, mut request: Request<Bytes>) -> Result<Response<Bytes>> {
        if let Some(auth) = &self.auth {
            auth.authenticate(&mut request).await?;
        }

        for interceptor in &self.request_interceptors {
            request = interceptor.intercept(request).await?;
        }

        let next = Next::new(self.transport.as_ref(), &self.middlewares);
        let mut response = next.call(request).await?;

        for interceptor in &self.response_interceptors {
            response = interceptor.intercept(response).await?;
        }

        Ok(response)
    }

    pub async fn execute_plain(&self, request: Request<Bytes>) -> Result<Response<Bytes>> {
        self.execute(request).await
    }

    pub async fn execute_multipart(
        &self,
        mut request: Request<Bytes>,
        parts: Vec<(String, crate::transport::MultipartField)>,
    ) -> Result<Response<Bytes>> {
        if let Some(auth) = &self.auth {
            auth.authenticate(&mut request).await?;
        }

        for interceptor in &self.request_interceptors {
            request = interceptor.intercept(request).await?;
        }

        let response = self.transport.send_multipart(request, parts).await?;
        Ok(response)
    }

    pub fn request(&self, method: http::Method, path: impl Into<String>) -> crate::request::RequestBuilder {
        crate::request::RequestBuilder::new(self.clone(), method, path)
    }

    pub fn get(&self, path: impl Into<String>) -> crate::request::RequestBuilder {
        self.request(http::Method::GET, path)
    }

    pub fn post(&self, path: impl Into<String>) -> crate::request::RequestBuilder {
        self.request(http::Method::POST, path)
    }

    pub fn put(&self, path: impl Into<String>) -> crate::request::RequestBuilder {
        self.request(http::Method::PUT, path)
    }

    pub fn delete(&self, path: impl Into<String>) -> crate::request::RequestBuilder {
        self.request(http::Method::DELETE, path)
    }

    pub fn patch(&self, path: impl Into<String>) -> crate::request::RequestBuilder {
        self.request(http::Method::PATCH, path)
    }

    pub fn head(&self, path: impl Into<String>) -> crate::request::RequestBuilder {
        self.request(http::Method::HEAD, path)
    }

    pub async fn resolve_url(&self, path: &str) -> Result<String> {
        let base = self.resolve_base_url().await?;
        Ok(format!("{}{}", base, path))
    }

    pub async fn send_and_decode<T: serde::de::DeserializeOwned>(
        &self,
        request: Request<Bytes>,
    ) -> Result<T> {
        let response = self.execute(request).await?;
        let status = response.status().as_u16();
        let body = response.into_body();

        if !self.is_success(status) {
            return Err(self.error_decoder.decode(status, &Default::default(), &body));
        }

        self.decode_response(&body)
    }
}

fn default_success_status(status: u16) -> bool {
    (200..300).contains(&status)
}

pub struct ClientBuilder {
    transport: Box<dyn Transport>,
    url_resolver: Box<dyn UrlResolver>,
    service_name: String,
    auth: Option<Box<dyn Auth>>,
    error_decoder: Box<dyn ErrorDecoder>,
    request_interceptors: Vec<Arc<dyn RequestInterceptor>>,
    response_interceptors: Vec<Arc<dyn ResponseInterceptor>>,
    middlewares: Vec<Arc<dyn Middleware>>,
    encoder: Box<dyn Encoder>,
    decoder: Box<dyn Decoder>,
    timeout: Timeout,
    log_level: LogLevel,
    success_status: fn(u16) -> bool,
}

impl ClientBuilder {
    pub fn new(transport: impl Transport) -> Self {
        Self {
            transport: Box::new(transport),
            url_resolver: Box::new(StaticUrl(String::new())),
            service_name: String::new(),
            auth: None,
            error_decoder: Box::new(DefaultErrorDecoder),
            request_interceptors: vec![],
            response_interceptors: vec![],
            middlewares: vec![],
            encoder: Box::new(JsonCodec),
            decoder: Box::new(JsonCodec),
            timeout: Timeout::default(),
            log_level: LogLevel::None,
            success_status: default_success_status,
        }
    }

    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.url_resolver = Box::new(StaticUrl(url.into()));
        self
    }

    pub fn url_resolver(mut self, resolver: impl UrlResolver) -> Self {
        self.url_resolver = Box::new(resolver);
        self
    }

    pub fn service_name(mut self, name: impl Into<String>) -> Self {
        self.service_name = name.into();
        self
    }

    pub fn auth(mut self, auth: impl Auth) -> Self {
        self.auth = Some(Box::new(auth));
        self
    }

    pub fn bearer_auth(self, token: impl Into<String>) -> Self {
        self.auth(BearerAuth::new(token))
    }

    pub fn basic_auth(self, user: impl Into<String>, pass: impl Into<String>) -> Self {
        self.auth(BasicAuth::new(user, pass))
    }

    pub fn error_decoder(mut self, d: impl ErrorDecoder) -> Self {
        self.error_decoder = Box::new(d);
        self
    }

    pub fn interceptor(mut self, i: impl RequestInterceptor) -> Self {
        self.request_interceptors.push(Arc::new(i));
        self
    }

    pub fn response_interceptor(mut self, i: impl ResponseInterceptor) -> Self {
        self.response_interceptors.push(Arc::new(i));
        self
    }

    pub fn middleware(mut self, m: impl Middleware) -> Self {
        self.middlewares.push(Arc::new(m));
        self
    }

    pub fn timeout(mut self, timeout: Timeout) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn connect_timeout(mut self, dur: Duration) -> Self {
        self.timeout.connect = dur;
        self
    }

    pub fn read_timeout(mut self, dur: Duration) -> Self {
        self.timeout.read = dur;
        self
    }

    pub fn write_timeout(mut self, dur: Duration) -> Self {
        self.timeout.write = dur;
        self
    }

    pub fn log_level(mut self, level: LogLevel) -> Self {
        self.log_level = level;
        self
    }

    pub fn success_status(mut self, f: fn(u16) -> bool) -> Self {
        self.success_status = f;
        self
    }

    pub fn encoder(mut self, e: impl Encoder) -> Self {
        self.encoder = Box::new(e);
        self
    }

    pub fn decoder(mut self, d: impl Decoder) -> Self {
        self.decoder = Box::new(d);
        self
    }

    pub fn build(self) -> Client {
        Client {
            transport: Arc::from(self.transport),
            url_resolver: Arc::from(self.url_resolver),
            service_name: self.service_name,
            auth: self.auth.map(Arc::from),
            error_decoder: Arc::from(self.error_decoder),
            request_interceptors: self.request_interceptors,
            response_interceptors: self.response_interceptors,
            middlewares: self.middlewares,
            encoder: Arc::from(self.encoder),
            decoder: Arc::from(self.decoder),
            timeout: self.timeout,
            log_level: self.log_level,
            success_status: self.success_status,
        }
    }
}