use serde::{Deserialize, Serialize};
use summer::{auto_config, App};
use summer_rfeign::RfeignPlugin;
use summer_web::{
    axum::response::IntoResponse,
    error::Result,
    extractor::Component,
    get, WebConfigurator, WebPlugin,
};

#[rfeign::http_client(service = "user_service", path = "/api")]
trait UserApi {
    #[rfeign::get("/user/{id}")]
    async fn get_user(&self, #[path] id: i64) -> rfeign::Result<User>;
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(flatten)]
    pub user: UserVo,
    pub roles: Vec<RoleDetailVo>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserVo {
    pub id: i64,
    pub avatar: String,
    pub status: i16,
    pub user_name: String,
    pub user_gender: String,
    pub nick_name: String,
    pub user_phone: String,
    pub user_email: String,
    pub create_by: String,
    pub create_time: String,
    pub update_by: String,
    pub update_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleDetailVo {
    pub role_id: i64,
    pub role_name: String,
    pub role_code: String,
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
