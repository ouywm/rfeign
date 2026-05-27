#[cfg(feature = "circuit-breaker")]
mod inner {
    use std::time::Duration;

    use async_trait::async_trait;
    use bytes::Bytes;
    use http::{Request, Response};
    use recloser::{AsyncRecloser, Recloser};

    use crate::error::{Error, Result};
    use crate::middleware::{Middleware, Next};

    pub struct CircuitBreakerMiddleware {
        recloser: AsyncRecloser,
    }

    impl CircuitBreakerMiddleware {
        pub fn new(error_rate: f32, open_wait: Duration) -> Self {
            let recloser = Recloser::custom()
                .error_rate(error_rate)
                .open_wait(open_wait)
                .build();
            Self {
                recloser: AsyncRecloser::from(recloser),
            }
        }

        pub fn default_with_wait(open_wait: Duration) -> Self {
            Self::new(0.5, open_wait)
        }
    }

    #[async_trait]
    impl Middleware for CircuitBreakerMiddleware {
        async fn handle(
            &self,
            request: Request<Bytes>,
            next: Next<'_>,
        ) -> Result<Response<Bytes>> {
            let result = self
                .recloser
                .call(async { next.call(request).await.map_err(CircuitError) })
                .await;

            match result {
                Ok(resp) => Ok(resp),
                Err(recloser::Error::Inner(e)) => Err(e.0),
                Err(recloser::Error::Rejected) => Err(Error::CircuitOpen),
            }
        }
    }

    struct CircuitError(Error);

    impl std::fmt::Display for CircuitError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::fmt::Debug for CircuitError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }

    impl std::error::Error for CircuitError {}
}

#[cfg(feature = "circuit-breaker")]
pub use inner::CircuitBreakerMiddleware;
