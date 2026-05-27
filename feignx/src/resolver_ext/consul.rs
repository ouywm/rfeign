#[cfg(feature = "consul")]
mod consul_impl {
    use std::sync::Arc;

    use async_trait::async_trait;
    use rs_consul::Consul;

    use feignx_core::error::{Error, Result};
    use feignx_core::resolver::UrlResolver;

    pub struct ConsulResolver {
        client: Arc<Consul>,
        scheme: String,
    }

    impl ConsulResolver {
        pub fn new(addr: impl Into<String>) -> Self {
            let config = rs_consul::Config {
                address: addr.into(),
                token: None,
            };
            Self {
                client: Arc::new(Consul::new(config)),
                scheme: "http".into(),
            }
        }

        pub fn with_token(addr: impl Into<String>, token: impl Into<String>) -> Self {
            let config = rs_consul::Config {
                address: addr.into(),
                token: Some(token.into()),
            };
            Self {
                client: Arc::new(Consul::new(config)),
                scheme: "http".into(),
            }
        }

        pub fn scheme(mut self, scheme: impl Into<String>) -> Self {
            self.scheme = scheme.into();
            self
        }
    }

    #[async_trait]
    impl UrlResolver for ConsulResolver {
        async fn resolve(&self, service_name: &str) -> Result<String> {
            let (services, _) = self
                .client
                .get_service_nodes(service_name, None)
                .await
                .map_err(|e| Error::Resolve(e.to_string()))?;

            let node = services
                .first()
                .ok_or_else(|| Error::Resolve(format!("no healthy instance for {}", service_name)))?;

            let addr = &node.service.address;
            let port = node.service.port;
            Ok(format!("{}://{}:{}", self.scheme, addr, port))
        }
    }
}

#[cfg(feature = "consul")]
pub use consul_impl::ConsulResolver;
