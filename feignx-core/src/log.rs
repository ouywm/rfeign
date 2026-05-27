use async_trait::async_trait;
use bytes::Bytes;
use http::{Request, Response};
use std::time::Instant;

use crate::error::Result;
use crate::middleware::{Middleware, Next};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LogLevel {
    #[default]
    None,
    Basic,
    Headers,
    Full,
}

pub struct LoggingMiddleware {
    level: LogLevel,
}

impl LoggingMiddleware {
    pub fn new(level: LogLevel) -> Self {
        Self { level }
    }
}

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn handle(
        &self,
        request: Request<Bytes>,
        next: Next<'_>,
    ) -> Result<Response<Bytes>> {
        if self.level == LogLevel::None {
            return next.call(request).await;
        }

        let method = request.method().clone();
        let uri = request.uri().to_string();
        let start = Instant::now();

        if self.level >= LogLevel::Basic {
            eprintln!("--> {} {}", method, uri);
        }
        if self.level >= LogLevel::Headers {
            for (name, value) in request.headers() {
                if let Ok(v) = value.to_str() {
                    eprintln!("    {}: {}", name, v);
                }
            }
        }
        if self.level >= LogLevel::Full && !request.body().is_empty() {
            if let Ok(body_str) = std::str::from_utf8(request.body()) {
                eprintln!("    body: {}", body_str);
            }
        }

        let result = next.call(request).await;
        let elapsed = start.elapsed();

        match &result {
            Ok(resp) => {
                if self.level >= LogLevel::Basic {
                    eprintln!("<-- {} {} ({}ms)", resp.status().as_u16(), uri, elapsed.as_millis());
                }
                if self.level >= LogLevel::Headers {
                    for (name, value) in resp.headers() {
                        if let Ok(v) = value.to_str() {
                            eprintln!("    {}: {}", name, v);
                        }
                    }
                }
            }
            Err(e) => {
                if self.level >= LogLevel::Basic {
                    eprintln!("<-- ERROR {} ({}ms): {}", uri, elapsed.as_millis(), e);
                }
            }
        }

        result
    }
}

impl PartialOrd for LogLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LogLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        fn rank(l: &LogLevel) -> u8 {
            match l {
                LogLevel::None => 0,
                LogLevel::Basic => 1,
                LogLevel::Headers => 2,
                LogLevel::Full => 3,
            }
        }
        rank(self).cmp(&rank(other))
    }
}