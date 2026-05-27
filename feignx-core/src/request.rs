use bytes::Bytes;
use http::{HeaderMap, HeaderName, HeaderValue, Method, Request, Response};
use serde::de::DeserializeOwned;

use crate::client::Client;
use crate::error::{Error, Result};

pub struct RequestBuilder {
    client: Client,
    method: Method,
    path: String,
    headers: HeaderMap,
    query: Vec<(String, String)>,
    body: Bytes,
}

impl RequestBuilder {
    pub(crate) fn new(client: Client, method: Method, path: impl Into<String>) -> Self {
        Self {
            client,
            method,
            path: path.into(),
            headers: HeaderMap::new(),
            query: Vec::new(),
            body: Bytes::new(),
        }
    }

    pub fn header(mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        if let (Ok(n), Ok(v)) = (
            HeaderName::try_from(name.as_ref()),
            HeaderValue::from_str(value.as_ref()),
        ) {
            self.headers.insert(n, v);
        }
        self
    }

    pub fn query<T: serde::Serialize>(mut self, params: &T) -> Self {
        if let Ok(s) = serde_urlencoded::to_string(params) {
            for pair in s.split('&').filter(|p| !p.is_empty()) {
                if let Some((k, v)) = pair.split_once('=') {
                    self.query.push((k.to_string(), v.to_string()));
                }
            }
        }
        self
    }

    pub fn query_pair(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query.push((key.into(), value.into()));
        self
    }

    pub fn headers_map(mut self, map: &std::collections::HashMap<String, String>) -> Self {
        for (k, v) in map {
            if let (Ok(n), Ok(val)) = (
                HeaderName::try_from(k.as_str()),
                HeaderValue::from_str(v.as_str()),
            ) {
                self.headers.insert(n, val);
            }
        }
        self
    }

    pub fn json<T: serde::Serialize>(mut self, value: &T) -> Result<Self> {
        self.body = self.client.encode_body(value)?;
        let ct = self.client.encoder().content_type();
        if let Ok(v) = HeaderValue::from_str(ct) {
            self.headers.insert(http::header::CONTENT_TYPE, v);
        }
        Ok(self)
    }

    pub fn body(mut self, body: impl Into<Bytes>) -> Self {
        self.body = body.into();
        self
    }

    fn build_url(&self, base: &str) -> String {
        let mut url = format!("{}{}", base, self.path);
        if !self.query.is_empty() {
            let qs = self
                .query
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            url.push('?');
            url.push_str(&qs);
        }
        url
    }

    pub async fn send(self) -> Result<RawResponse> {
        let base = self.client.url_resolver().resolve("").await?;
        let url = self.build_url(&base);

        let mut builder = Request::builder().method(self.method).uri(&url);
        if let Some(h) = builder.headers_mut() {
            *h = self.headers;
        }
        let request = match builder.body(self.body) {
            Ok(r) => r,
            Err(e) => return Err(Error::Other(e.to_string())),
        };

        let response = self.client.execute(request).await?;
        Ok(RawResponse {
            client: self.client,
            response,
        })
    }
}

pub struct RawResponse {
    client: Client,
    response: Response<Bytes>,
}

impl RawResponse {
    pub fn status(&self) -> u16 {
        self.response.status().as_u16()
    }

    pub fn headers(&self) -> &HeaderMap {
        self.response.headers()
    }

    pub fn is_success(&self) -> bool {
        self.client.is_success(self.status())
    }

    pub fn bytes(self) -> Bytes {
        self.response.into_body()
    }

    pub fn text(self) -> Result<String> {
        let bytes = self.response.into_body();
        String::from_utf8(bytes.to_vec()).map_err(|e| Error::Other(e.to_string()))
    }

    pub fn json<T: DeserializeOwned>(self) -> Result<T> {
        let status = self.status();
        let body = self.response.into_body();
        if !self.client.is_success(status) {
            return Err(self.client.error_decoder().decode(status, &Default::default(), &body));
        }
        if body.is_empty() {
            return self.client.decode_response(b"null");
        }
        self.client.decode_response(&body)
    }

    pub fn ensure_success(self) -> Result<()> {
        let status = self.status();
        if !self.client.is_success(status) {
            let body = self.response.into_body();
            return Err(self.client.error_decoder().decode(status, &Default::default(), &body));
        }
        Ok(())
    }
}
