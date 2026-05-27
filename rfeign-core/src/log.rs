#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LogLevel {
    #[default]
    None,
    Basic,
    Headers,
    Full,
}

impl PartialOrd for LogLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LogLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.rank()).cmp(&other.rank())
    }
}

impl LogLevel {
    fn rank(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Basic => 1,
            Self::Headers => 2,
            Self::Full => 3,
        }
    }
}

pub struct LoggingMiddleware {
    level: LogLevel,
}

impl LoggingMiddleware {
    pub fn new(level: LogLevel) -> Self {
        Self { level }
    }
}

use async_trait::async_trait;
use bytes::Bytes;
use http::{Request, Response};
use std::time::Instant;

use crate::error::Result;
use crate::middleware::{Middleware, Next};

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
            if let Ok(s) = std::str::from_utf8(request.body()) {
                eprintln!("    body: {}", s);
            }
        }

        let result = next.call(request).await;
        let elapsed = start.elapsed();

        match &result {
            Ok(resp) => {
                if self.level >= LogLevel::Basic {
                    eprintln!("<-- {} ({}ms) {}", resp.status().as_u16(), elapsed.as_millis(), uri);
                }
                if self.level >= LogLevel::Headers {
                    for (name, value) in resp.headers() {
                        if let Ok(v) = value.to_str() {
                            eprintln!("    {}: {}", name, v);
                        }
                    }
                }
                if self.level >= LogLevel::Full && !resp.body().is_empty() {
                    if let Ok(s) = std::str::from_utf8(resp.body()) {
                        eprintln!("    body: {}", s);
                    }
                }
            }
            Err(e) => {
                if self.level >= LogLevel::Basic {
                    eprintln!("<-- ERROR ({}ms) {}: {}", elapsed.as_millis(), uri, e);
                }
            }
        }

        result
    }
}
