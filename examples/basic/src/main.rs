use rfeign::client::ClientBuilder;
use rfeign::ReqwestTransport;
use rfeign::RequestParam;
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

#[rfeign::http_client(base_url = "http://localhost:8080")]
#[headers("X-App: rfeign", "Accept: application/json")]
trait UserApi {
    #[rfeign::get("/users/{id}")]
    #[rfeign::header("X-Trace-Id", "abc-123")]
    async fn get_user(&self, #[path] id: i64) -> rfeign::Result<User>;

    #[rfeign::get("/users")]
    async fn list_users(&self, #[query] page: u32, #[query] size: u32)
    -> rfeign::Result<Vec<User>>;

    #[rfeign::post("/users")]
    async fn create_user(&self, #[body] user: CreateUser) -> rfeign::Result<User>;

    #[rfeign::delete("/users/{id}")]
    async fn delete_user(
        &self,
        #[path] id: i64,
        #[header("Authorization")] token: String,
    ) -> rfeign::Result<()>;
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
