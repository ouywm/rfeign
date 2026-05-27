use feignx::{ClientBuilder, ReqwestTransport};
use feignx::resolver_ext::NacosResolverBuilder;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct User {
    id: i64,
    name: String,
}

#[feignx::http_client(service = "user-service", path = "/api/v1")]
trait UserApi {
    #[feignx::get("/users/{id}")]
    async fn get_user(&self, #[path] id: i64) -> feignx::Result<User>;
}

#[tokio::main]
async fn main() -> feignx::Result<()> {
    let resolver = NacosResolverBuilder::new("127.0.0.1:8848")
        .namespace("public")
        .group("DEFAULT_GROUP")
        .build()
        .await?;

    let client = ClientBuilder::new(ReqwestTransport::new())
        .url_resolver(resolver)
        .service_name("user-service")
        .build();

    let _api = UserApiClient::new(client);
    println!("NacosResolver configured for service: {:?}", UserApiClient::service_name());

    Ok(())
}
