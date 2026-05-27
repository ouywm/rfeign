use rfeign::resolver_ext::NacosResolverBuilder;
use rfeign::{ClientBuilder, ReqwestTransport};
use serde::{Deserialize, Serialize};

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

#[rfeign::http_client(service = "user_service", path = "/api")]
#[headers(
    "Authorization : Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJzdW1tZXItYWRtaW4iLCJhdWQiOiJzdW1tZXItYWRtaW4iLCJzdWIiOiIyIiwidHlwIjoiYWNjZXNzIiwiaWF0IjoxNzc5ODczNTU0LCJleHAiOjE3Nzk4ODA3NTQsImRldiI6IndlYiIsInVzZXJfbmFtZSI6IkFkbWluIiwibmlja19uYW1lIjoi566h55CG5ZGYIiwicm9sZXMiOlsiUl9BRE1JTiJdLCJwYiI6IkFFQTRBQnc9In0.2XYPHd4Nzfaym3h96QXL5QmqaFzVlMIMWqcHOsmXrbY"
)]
trait UserApi {
    #[rfeign::get("/user/{id}")]
    async fn get_user(&self, #[path] id: i64) -> rfeign::Result<User>;
}

#[tokio::main]
async fn main() -> rfeign::Result<()> {
    let resolver = NacosResolverBuilder::new("127.0.0.1:8848")
        .namespace("public")
        .group("DEFAULT_GROUP")
        .build()
        .await?;

    let client = ClientBuilder::new(ReqwestTransport::new())
        .url_resolver(resolver)
        .service_name("user_service")
        .build();

    let api = UserApiClient::new(client);
    let u = api.get_user(1).await?;
    println!("{:?}", u);
    println!(
        "NacosResolver configured for service: {:?}",
        UserApiClient::service_name()
    );

    Ok(())
}
