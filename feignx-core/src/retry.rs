use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use http::{Request, Response};

use crate::error::Result;
use crate::middleware::{Middleware, Next};

pub struct Retry {
    max_attempts: u32,
    backoff: BackoffStrategy,
    retryable: fn(u16) -> bool,
}

#[derive(Clone)]
pub enum BackoffStrategy {
    Fixed(Duration),
    Exponential { base: Duration, max: Duration },
}

impl BackoffStrategy {
    fn delay(&self, attempt: u32) -> Duration {
        match self {
            Self::Fixed(d) => *d,
            Self::Exponential { base, max } => {
                let delay = base.saturating_mul(2u32.saturating_pow(attempt - 1));
                if delay > *max { *max } else { delay }
            }
        }
    }
}

impl Retry {
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            backoff: BackoffStrategy::Fixed(Duration::from_millis(500)),
            retryable: default_retryable,
        }
    }

    pub fn backoff(mut self, strategy: BackoffStrategy) -> Self {
        self.backoff = strategy;
        self
    }

    pub fn retryable(mut self, f: fn(u16) -> bool) -> Self {
        self.retryable = f;
        self
    }
}

fn default_retryable(status: u16) -> bool {
    matches!(status, 429 | 500 | 502 | 503 | 504)
}

#[async_trait]
impl Middleware for Retry {
    async fn handle(
        &self,
        request: Request<Bytes>,
        next: Next<'_>,
    ) -> Result<Response<Bytes>> {
        let mut attempts = 0u32;
        loop {
            let req = clone_request(&request);
            let result = next.clone().call(req).await;

            attempts += 1;

            match &result {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    if !(self.retryable)(status) || attempts >= self.max_attempts {
                        return result;
                    }
                }
                Err(_) => {
                    if attempts >= self.max_attempts {
                        return result;
                    }
                }
            }

            tokio::time::sleep(self.backoff.delay(attempts)).await;
        }
    }
}

fn clone_request(req: &Request<Bytes>) -> Request<Bytes> {
    let mut builder = Request::builder()
        .method(req.method().clone())
        .uri(req.uri().clone());
    if let Some(headers) = builder.headers_mut() {
        *headers = req.headers().clone();
    }
    match builder.body(req.body().clone()) {
        Ok(r) => r,
        Err(_) => Request::new(req.body().clone()),
    }
}
