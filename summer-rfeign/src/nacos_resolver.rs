use async_trait::async_trait;
use nacos_sdk::api::naming::NamingService;

use rfeign_core::error::{Error, Result};
use rfeign_core::resolver::UrlResolver;

pub struct SummerNacosResolver {
    naming: NamingService,
}

impl SummerNacosResolver {
    pub fn new(naming: NamingService) -> Self {
        Self { naming }
    }
}

#[async_trait]
impl UrlResolver for SummerNacosResolver {
    async fn resolve(&self, service_name: &str) -> Result<String> {
        let instances = self
            .naming
            .get_all_instances(
                service_name.to_string(),
                Some("DEFAULT_GROUP".to_string()),
                Vec::new(),
                false,
            )
            .await
            .map_err(|e| Error::Resolve(e.to_string()))?;

        let healthy: Vec<_> = instances.iter().filter(|i| i.healthy).collect();
        let instance = if let Some(h) = healthy.first() {
            *h
        } else {
            instances.first().ok_or_else(|| {
                Error::Resolve(format!("no instance for {}", service_name))
            })?
        };

        let scheme = if instance.metadata.contains_key("secure") {
            "https"
        } else {
            "http"
        };

        Ok(format!("{}://{}:{}", scheme, instance.ip, instance.port))
    }
}
