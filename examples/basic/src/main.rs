use feignx::client::ClientBuilder;
use feignx::ReqwestTransport;
use feignx::RequestParam;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct User {
    id: i64,
    name: String,
}

#[derive(Debug, Serialize)]
struct CreateUser {
    name: String,
}

#[derive(RequestParam)]
struct ListParams {
    page: u32,
    size: u32,
    #[param(name = "q")]
    keyword: Option<String>,
}

#[feignx::http_client(base_url = "http://localhost:8080")]
#[headers("X-App: feignx", "Accept: application/json")]
trait UserApi {
    #[feignx::get("/users/{id}")]
    async fn get_user(&self, #[path] id: i64) -> feignx::Result<User>;

    #[feignx::get("/users")]
    async fn list_users(&self, #[query] page: u32, #[query] size: u32)
    -> feignx::Result<Vec<User>>;

    #[feignx::post("/users")]
    async fn create_user(&self, #[body] user: CreateUser) -> feignx::Result<User>;

    #[feignx::delete("/users/{id}")]
    async fn delete_user(
        &self,
        #[path] id: i64,
        #[header("Authorization")] token: String,
    ) -> feignx::Result<()>;
}

#[tokio::main]
async fn main() {
    let params = ListParams { page: 1, size: 20, keyword: Some("alice".into()) };
    println!("query_pairs: {:?}", params.to_query_pairs());

    let params2 = ListParams { page: 2, size: 10, keyword: None };
    println!("query_pairs: {:?}", params2.to_query_pairs());

    let client = ClientBuilder::new(ReqwestTransport::new())
        .base_url("http://localhost:8080")
        .build();

    let _api = UserApiClient::new(client);
    println!("base_url: {:?}", UserApiClient::base_url());
}
