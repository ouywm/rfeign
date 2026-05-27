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

#[derive(Clone)]
struct UserApiCClient {
    client: feignx::client::Client,
}

impl UserApiCClient {
    pub fn new(client: feignx::client::Client) -> Self {
        Self { client }
    }

    pub fn base_url() -> String {
        "http://localhost:8080".to_string()
    }

    pub fn service_name() -> String {
        "user_service".to_string()
    }

    pub fn path() -> String {
        "/api/v1".to_string()
    }
}

#[feignx::async_trait]
impl UserApi for UserApiCClient {
    async fn get_user(&self, id: i64) -> feignx::Result<User> {
        let mut url = self
            .client
            .resolve_url(&::alloc::__export::must_use({
                ::alloc::fmt::format(::alloc::__export::format_args!("/users/{}", id))
            }))
            .await?;
        let mut builder = feignx::http::Request::builder()
            .method(feignx::http::Method::GET)
            .uri(&url);
        let body = feignx::bytes::Bytes::new();
        let request = match builder.body(body) {
            Ok(r) => r,
            Err(e) => return Err(feignx::Error::Other(e.to_string())),
        };
        self.client.send_and_decode(request).await
    }
    async fn list_users(&self, page: u32, size: u32) -> feignx::Result<Vec<User>> {
        let mut url = self.client.resolve_url(&"/users").await?;
        let qs = [
            (
                "page",
                feignx::serde_urlencoded::to_string(&page).unwrap_or_default(),
            ),
            (
                "size",
                feignx::serde_urlencoded::to_string(&size).unwrap_or_default(),
            ),
        ]
        .iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| {
            ::alloc::__export::must_use({
                ::alloc::fmt::format(::alloc::__export::format_args!("{}={}", k, v))
            })
        })
        .collect::<Vec<_>>()
        .join("&");
        if !qs.is_empty() {
            url.push('?');
            url.push_str(&qs);
        }
        let mut builder = feignx::http::Request::builder()
            .method(feignx::http::Method::GET)
            .uri(&url);
        let body = feignx::bytes::Bytes::new();
        let request = match builder.body(body) {
            Ok(r) => r,
            Err(e) => return Err(feignx::Error::Other(e.to_string())),
        };
        self.client.send_and_decode(request).await
    }
    async fn create_user(&self, user: CreateUser) -> feignx::Result<User> {
        let mut url = self.client.resolve_url(&"/users").await?;
        let mut builder = feignx::http::Request::builder()
            .method(feignx::http::Method::POST)
            .uri(&url);
        let body = self.client.encode_body(&user)?;
        let request = match builder.body(body) {
            Ok(r) => r,
            Err(e) => return Err(feignx::Error::Other(e.to_string())),
        };
        self.client.send_and_decode(request).await
    }
    async fn delete_user(&self, id: i64, token: String) -> feignx::Result<()> {
        let mut url = self
            .client
            .resolve_url(&::alloc::__export::must_use({
                ::alloc::fmt::format(::alloc::__export::format_args!("/users/{}", id))
            }))
            .await?;
        let mut builder = feignx::http::Request::builder()
            .method(feignx::http::Method::DELETE)
            .uri(&url);
        builder = builder.header("Authorization", token.to_string());
        let body = feignx::bytes::Bytes::new();
        let request = match builder.body(body) {
            Ok(r) => r,
            Err(e) => return Err(feignx::Error::Other(e.to_string())),
        };
        self.client.send_and_decode(request).await
    }
}
