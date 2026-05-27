use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use summer::{auto_config, App};
use summer_rfeign::RfeignPlugin;
use summer_web::{
    axum::response::IntoResponse,
    error::Result,
    extractor::Component,
    get, WebConfigurator, WebPlugin,
};

// ========== 通用响应 ==========

#[derive(Debug, Serialize, Deserialize)]
pub struct Page<T> {
    pub content: Vec<T>,
    pub total_elements: i64,
    pub size: i64,
    pub page: i64,
    pub total_pages: i64,
}

// ========== 数据模型 ==========

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDetailVo {
    pub id: i64,
    pub user_name: String,
    pub nick_name: String,
    pub roles: Vec<RoleVo>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserVo {
    pub id: i64,
    pub user_name: String,
    pub nick_name: String,
    pub status: i16,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleVo {
    pub role_id: i64,
    pub role_name: String,
    pub role_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictDataSimpleVo {
    pub label: String,
    pub value: String,
}

// ========== 功能测试：rfeign 客户端声明 ==========

// 测试: GET + path param + query param + DynamicAuth + service discovery
#[rfeign::http_client(service = "user-service", path = "/api")]
trait TestApi {
    // 功能1: GET + #[path]
    #[rfeign::get("/user/{id}")]
    async fn get_user(&self, #[path] id: i64) -> rfeign::Result<UserDetailVo>;

    // 功能2: GET + #[query] 多参数
    #[rfeign::get("/user/list")]
    async fn list_users(&self, #[query] page: i64, #[query] size: i64)
        -> rfeign::Result<Page<UserVo>>;

    // 功能3: GET + #[path] String 类型
    #[rfeign::get("/dict/data/by-type/{dict_type}")]
    async fn get_dict_by_type(&self, #[path] dict_type: String)
        -> rfeign::Result<Vec<DictDataSimpleVo>>;

    // 功能4: GET 无参数
    #[rfeign::get("/dict/all")]
    async fn get_all_dicts(&self)
        -> rfeign::Result<HashMap<String, Vec<DictDataSimpleVo>>>;

    // 功能5: GET 返回动态 JSON
    #[rfeign::get("/monitor/server")]
    async fn server_info(&self) -> rfeign::Result<Value>;

    // 功能6: GET + #[query] 分页 + 列表
    #[rfeign::get("/role/list")]
    async fn list_roles(&self, #[query] page: i64, #[query] size: i64)
        -> rfeign::Result<Page<RoleVo>>;

    // 功能7: POST + #[body]
    #[rfeign::post("/role")]
    async fn create_role(&self, #[body] role: CreateRoleDto)
        -> rfeign::Result<Value>;

    // 功能8: PUT + #[path] + #[body]
    #[rfeign::put("/role/{role_id}")]
    async fn update_role(&self, #[path] role_id: i64, #[body] role: UpdateRoleDto)
        -> rfeign::Result<Value>;

    // 功能9: DELETE + #[path]
    #[rfeign::delete("/role/{role_id}")]
    async fn delete_role(&self, #[path] role_id: i64) -> rfeign::Result<Value>;

    // 功能10: PATCH
    #[rfeign::patch("/user/{id}")]
    async fn patch_user(&self, #[path] id: i64, #[body] data: Value)
        -> rfeign::Result<Value>;

    // 功能11: 方法级别 #[header]
    #[rfeign::get("/user/info")]
    async fn get_user_info_with_header(
        &self,
        #[header("X-Request-Id")] request_id: String,
    ) -> rfeign::Result<Value>;

    // 功能12: #[timeout] 超时控制
    #[rfeign::get("/monitor/server")]
    #[timeout(5000)]
    async fn server_info_with_timeout(&self) -> rfeign::Result<Value>;

    // 功能13: #[query(multi)] 多值参数
    #[rfeign::get("/user/list")]
    async fn list_users_by_ids(
        &self,
        #[query(multi)] id: Vec<i64>,
    ) -> rfeign::Result<Value>;

    // 功能14: #[query(csv)] 逗号分隔
    #[rfeign::get("/user/list")]
    async fn list_users_csv(
        &self,
        #[query(csv)] ids: Vec<i64>,
    ) -> rfeign::Result<Value>;
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoleDto {
    pub role_name: String,
    pub role_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRoleDto {
    pub role_name: String,
    pub role_code: String,
}

// 测试: 登录接口 (POST + body, 无需认证)
#[rfeign::http_client(service = "user-service", path = "/api")]
trait AuthApi {
    #[rfeign::post("/auth/login")]
    async fn login(&self, #[body] dto: LoginDto) -> rfeign::Result<LoginVo>;

    #[rfeign::post("/auth/refresh")]
    async fn refresh_token(&self, #[body] dto: RefreshTokenDto)
        -> rfeign::Result<LoginVo>;
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginDto {
    pub user_name: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshTokenDto {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginVo {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

// ========== Web Handlers ==========

macro_rules! err {
    ($e:expr) => {
        summer_web::error::KnownWebError::internal_server_error($e.to_string())
    };
}

#[get("/test/user/{id}")]
async fn test_get_user(
    summer_web::extractor::Path(id): summer_web::extractor::Path<i64>,
    Component(api): Component<TestApiClient>,
) -> Result<impl IntoResponse> {
    let r = api.get_user(id).await.map_err(|e| err!(e))?;
    Ok(summer_web::axum::Json(r))
}

#[get("/test/users")]
async fn test_list_users(
    Component(api): Component<TestApiClient>,
) -> Result<impl IntoResponse> {
    let r = api.list_users(1, 10).await.map_err(|e| err!(e))?;
    Ok(summer_web::axum::Json(r))
}

#[get("/test/dict/{dict_type}")]
async fn test_dict_by_type(
    summer_web::extractor::Path(dt): summer_web::extractor::Path<String>,
    Component(api): Component<TestApiClient>,
) -> Result<impl IntoResponse> {
    let r = api.get_dict_by_type(dt).await.map_err(|e| err!(e))?;
    Ok(summer_web::axum::Json(r))
}

#[get("/test/dict/all")]
async fn test_all_dicts(
    Component(api): Component<TestApiClient>,
) -> Result<impl IntoResponse> {
    let r = api.get_all_dicts().await.map_err(|e| err!(e))?;
    Ok(summer_web::axum::Json(r))
}

#[get("/test/monitor")]
async fn test_monitor(
    Component(api): Component<TestApiClient>,
) -> Result<impl IntoResponse> {
    let r = api.server_info().await.map_err(|e| err!(e))?;
    Ok(summer_web::axum::Json(r))
}

#[get("/test/roles")]
async fn test_roles(
    Component(api): Component<TestApiClient>,
) -> Result<impl IntoResponse> {
    let r = api.list_roles(1, 10).await.map_err(|e| err!(e))?;
    Ok(summer_web::axum::Json(r))
}

// 功能10: 方法级别 header
#[get("/test/user-info")]
async fn test_user_info_header(
    Component(api): Component<TestApiClient>,
) -> Result<impl IntoResponse> {
    let r = api.get_user_info_with_header("req-12345".to_string())
        .await.map_err(|e| err!(e))?;
    Ok(summer_web::axum::Json(r))
}

// 功能11: timeout
#[get("/test/monitor-timeout")]
async fn test_monitor_timeout(
    Component(api): Component<TestApiClient>,
) -> Result<impl IntoResponse> {
    let r = api.server_info_with_timeout().await.map_err(|e| err!(e))?;
    Ok(summer_web::axum::Json(r))
}

// 功能12: query multi
#[get("/test/users-multi")]
async fn test_users_multi(
    Component(api): Component<TestApiClient>,
) -> Result<impl IntoResponse> {
    let r = api.list_users_by_ids(vec![1, 2, 3]).await.map_err(|e| err!(e))?;
    Ok(summer_web::axum::Json(r))
}

// 功能13: query csv
#[get("/test/users-csv")]
async fn test_users_csv(
    Component(api): Component<TestApiClient>,
) -> Result<impl IntoResponse> {
    let r = api.list_users_csv(vec![1, 2, 3]).await.map_err(|e| err!(e))?;
    Ok(summer_web::axum::Json(r))
}

// 功能测试: POST + #[body] 登录
#[get("/test/login")]
async fn test_login(
    Component(api): Component<AuthApiClient>,
) -> Result<impl IntoResponse> {
    let dto = LoginDto {
        user_name: "Admin".to_string(),
        password: "123456".to_string(),
    };
    let r = api.login(dto).await.map_err(|e| err!(e))?;
    Ok(summer_web::axum::Json(r))
}

// 功能测试: POST + #[body] 刷新 token
#[get("/test/refresh/{token}")]
async fn test_refresh(
    summer_web::extractor::Path(token): summer_web::extractor::Path<String>,
    Component(api): Component<AuthApiClient>,
) -> Result<impl IntoResponse> {
    let dto = RefreshTokenDto { refresh_token: token };
    let r = api.refresh_token(dto).await.map_err(|e| err!(e))?;
    Ok(summer_web::axum::Json(r))
}

// ========== DynamicAuth: 自动登录获取 token ==========

struct AdminTokenProvider;

impl rfeign::auth::TokenLike for LoginVo {
    fn access_token(&self) -> &str { &self.access_token }
    fn refresh_token(&self) -> Option<&str> { Some(&self.refresh_token) }
    fn expires_in_secs(&self) -> u64 { self.expires_in as u64 }
}

#[rfeign::async_trait]
impl rfeign::auth::TokenProvider for AdminTokenProvider {
    type Token = LoginVo;
    type Refresh = LoginVo;

    async fn fetch_token(&self) -> rfeign::Result<LoginVo> {
        let client = rfeign::ClientBuilder::new(rfeign::ReqwestTransport::new())
            .base_url("http://localhost:8080")
            .build();
        let resp = client.post("/api/auth/login")
            .json(&LoginDto {
                user_name: "Admin".to_string(),
                password: "123456".to_string(),
            })?
            .send()
            .await?;
        resp.json()
    }

    async fn refresh(&self, refresh_token: &str) -> rfeign::Result<LoginVo> {
        let client = rfeign::ClientBuilder::new(rfeign::ReqwestTransport::new())
            .base_url("http://localhost:8080")
            .build();
        let resp = client.post("/api/auth/refresh")
            .json(&RefreshTokenDto {
                refresh_token: refresh_token.to_string(),
            })?
            .send()
            .await?;
        resp.json()
    }
}

// ========== Main ==========

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(summer_nacos::NacosPlugin)
        .add_plugin(
            RfeignPlugin::new()
                .auth(rfeign::auth::DynamicAuth::new(AdminTokenProvider))
        )
        .add_plugin(WebPlugin)
        .run()
        .await
}
