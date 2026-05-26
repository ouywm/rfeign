use feignx::ReqwestTransport;
use feignx::client::ClientBuilder;
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

#[feignx::http_client(base_url = "http://localhost:8080")]
trait UserApi {
    #[feignx::get("/users/{id}")]
    async fn get_user(&self, #[path] id: i64) -> feignx::Result<User>;

    #[feignx::get("/users")]
    async fn list_users(&self, #[query] page: u32, #[query] size: u32) -> feignx::Result<Vec<User>>;

    #[feignx::post("/users")]
    async fn create_user(&self, #[body] user: CreateUser) -> feignx::Result<User>;

    #[feignx::delete("/users/{id}")]
    async fn delete_user(&self, #[path] id: i64, #[header("Authorization")] token: String) -> feignx::Result<()>;
}

#[tokio::main]
async fn main() {
    let client = ClientBuilder::new(ReqwestTransport::new())
        .base_url("http://localhost:8080")
        .build();

    let _api = UserApiClient::new(client);
    println!("UserApiClient created with all methods");
}