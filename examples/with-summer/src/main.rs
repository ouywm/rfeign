use serde::{Deserialize, Serialize};
use summer::{auto_config, App};
use summer_rfeign::RfeignPlugin;
use summer_web::{
    axum::response::IntoResponse,
    error::Result,
    extractor::Component,
    get, WebConfigurator, WebPlugin,
};

#[rfeign::http_client(service = "user-service")]
trait UserApi {
    #[rfeign::get("/users/{id}")]
    async fn get_user(&self, #[path] id: i64) -> rfeign::Result<User>;

    #[rfeign::get("/users")]
    async fn list_users(&self, #[query] page: u32, #[query] size: u32)
        -> rfeign::Result<Vec<User>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: i64,
    name: String,
}

#[get("/user/{id}")]
async fn get_user(
    Component(api): Component<UserApiClient>,
    summer_web::extractor::Path(id): summer_web::extractor::Path<i64>,
) -> Result<impl IntoResponse> {
    let user = api.get_user(id).await.map_err(|e| {
        summer_web::error::KnownWebError::internal_server_error(e.to_string())
    })?;
    Ok(summer_web::axum::Json(user))
}

#[get("/users")]
async fn list_users(
    Component(api): Component<UserApiClient>,
) -> Result<impl IntoResponse> {
    let users = api.list_users(1, 10).await.map_err(|e| {
        summer_web::error::KnownWebError::internal_server_error(e.to_string())
    })?;
    Ok(summer_web::axum::Json(users))
}

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(summer_nacos::NacosPlugin)
        .add_plugin(RfeignPlugin)
        .add_plugin(WebPlugin)
        .run()
        .await
}
