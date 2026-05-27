use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use summer::app::AppBuilder;
use summer::config::{ConfigRegistry, Configurable};
use summer::plugin::{MutableComponentRegistry, Plugin};

use rfeign_core::auth::Auth;
use rfeign_core::client::ClientBuilder;
use rfeign_core::interceptor::{RequestInterceptor, ResponseInterceptor};
use rfeign_reqwest::ReqwestTransport;

#[cfg(feature = "nacos")]
mod nacos_resolver;

#[cfg(feature = "nacos")]
pub use nacos_resolver::SummerNacosResolver;

pub use rfeign_core;
pub use summer;

#[derive(Debug, Clone, Default, Configurable, Deserialize)]
#[config_prefix = "rfeign"]
pub struct RfeignConfig {
    pub base_url: Option<String>,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

fn default_timeout() -> u64 {
    30000
}

pub struct RfeignPlugin {
    auth: Option<Arc<dyn Auth>>,
    request_interceptors: Vec<Arc<dyn RequestInterceptor>>,
    response_interceptors: Vec<Arc<dyn ResponseInterceptor>>,
}

impl RfeignPlugin {
    pub fn new() -> Self {
        Self {
            auth: None,
            request_interceptors: Vec::new(),
            response_interceptors: Vec::new(),
        }
    }

    pub fn auth(mut self, auth: impl Auth) -> Self {
        self.auth = Some(Arc::new(auth));
        self
    }

    pub fn request_interceptor(mut self, interceptor: impl RequestInterceptor) -> Self {
        self.request_interceptors.push(Arc::new(interceptor));
        self
    }

    pub fn response_interceptor(mut self, interceptor: impl ResponseInterceptor) -> Self {
        self.response_interceptors.push(Arc::new(interceptor));
        self
    }
}

impl Default for RfeignPlugin {
    fn default() -> Self {
        Self::new()
    }
}

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

        if let Some(ref auth) = self.auth {
            builder = builder.auth(auth.clone());
        }

        for interceptor in &self.request_interceptors {
            builder = builder.interceptor(interceptor.clone());
        }

        for interceptor in &self.response_interceptors {
            builder = builder.response_interceptor(interceptor.clone());
        }

        let client = builder.build();
        app.add_component(client);
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
