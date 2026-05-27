use async_trait::async_trait;
use serde::Deserialize;
use summer::app::AppBuilder;
use summer::config::{ConfigRegistry, Configurable};
use summer::plugin::{MutableComponentRegistry, Plugin};

use rfeign_core::client::ClientBuilder;
use rfeign_reqwest::ReqwestTransport;

#[cfg(feature = "nacos")]
mod nacos_resolver;

#[cfg(feature = "nacos")]
pub use nacos_resolver::SummerNacosResolver;

pub use rfeign_core;
pub use summer;

/// Config read from `[rfeign]` section in app.toml
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RfeignConfig {
    pub base_url: Option<String>,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

impl Configurable for RfeignConfig {
    fn config_prefix() -> &'static str {
        "rfeign"
    }
}

fn default_timeout() -> u64 {
    30000
}

pub struct RfeignPlugin;

#[async_trait]
impl Plugin for RfeignPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let config = app.get_config::<RfeignConfig>().unwrap_or_default();

        let transport = ReqwestTransport::new();
        let mut builder = ClientBuilder::new(transport);

        if let Some(ref base_url) = config.base_url {
            builder = builder.base_url(base_url);
        }

        #[cfg(feature = "nacos")]
        {
            let resolver = build_nacos_resolver(app);
            builder = builder.url_resolver(resolver);
        }

        let client = builder.build();
        app.add_component(client);
    }

    fn name(&self) -> &str {
        "summer-rfeign"
    }

    fn dependencies(&self) -> Vec<&str> {
        #[cfg(feature = "nacos")]
        { vec!["summer-nacos"] }
        #[cfg(not(feature = "nacos"))]
        { vec![] }
    }
}

#[cfg(feature = "nacos")]
fn build_nacos_resolver(app: &AppBuilder) -> SummerNacosResolver {
    use nacos_sdk::api::naming::NamingService;
    use summer::plugin::ComponentRegistry;

    let naming = app
        .get_component::<NamingService>()
        .expect("NamingService not found. Add NacosPlugin before RfeignPlugin.");
    SummerNacosResolver::new(naming)
}
