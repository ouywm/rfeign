#[cfg(feature = "nacos")]
mod nacos_impl {
    use std::sync::Arc;

    use async_trait::async_trait;
    use nacos_sdk::api::naming::{NamingService, NamingServiceBuilder};
    use nacos_sdk::api::props::ClientProps;

    use rfeign_core::error::{Error, Result};
    use rfeign_core::resolver::UrlResolver;

    pub struct NacosResolver {
        naming: Arc<NamingService>,
        group: Option<String>,
        clusters: Vec<String>,
        scheme: String,
    }

    pub struct NacosResolverBuilder {
        props: ClientProps,
        group: Option<String>,
        clusters: Vec<String>,
        scheme: String,
    }

    impl NacosResolverBuilder {
        pub fn new(server_addr: impl Into<String>) -> Self {
            Self {
                props: ClientProps::new().server_addr(server_addr),
                group: None,
                clusters: Vec::new(),
                scheme: "http".into(),
            }
        }

        pub fn namespace(mut self, ns: impl Into<String>) -> Self {
            self.props = self.props.namespace(ns);
            self
        }

        pub fn group(mut self, group: impl Into<String>) -> Self {
            self.group = Some(group.into());
            self
        }

        pub fn clusters(mut self, clusters: Vec<String>) -> Self {
            self.clusters = clusters;
            self
        }

        pub fn scheme(mut self, scheme: impl Into<String>) -> Self {
            self.scheme = scheme.into();
            self
        }

        pub fn auth(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
            self.props = self.props.auth_username(username).auth_password(password);
            self
        }

        pub async fn build(self) -> Result<NacosResolver> {
            let naming = NamingServiceBuilder::new(self.props)
                .build()
                .await
                .map_err(|e| Error::Resolve(e.to_string()))?;

            Ok(NacosResolver {
                naming: Arc::new(naming),
                group: self.group,
                clusters: self.clusters,
                scheme: self.scheme,
            })
        }
    }

    #[async_trait]
    impl UrlResolver for NacosResolver {
        async fn resolve(&self, service_name: &str) -> Result<String> {
            let instance = self
                .naming
                .select_one_healthy_instance(
                    service_name.to_string(),
                    self.group.clone(),
                    self.clusters.clone(),
                    true,
                )
                .await
                .map_err(|e| Error::Resolve(e.to_string()))?;

            Ok(format!("{}://{}:{}", self.scheme, instance.ip(), instance.port()))
        }
    }
}

#[cfg(feature = "nacos")]
pub use nacos_impl::{NacosResolver, NacosResolverBuilder};
