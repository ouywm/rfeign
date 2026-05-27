# 参数属性

声明式 API 中，通过参数属性标注每个方法参数的用途。

## #[path]

路径参数，替换 URL 模板中的 `{name}` 占位符：

```rust
#[rfeign::get("/users/{id}")]
async fn get_user(&self, #[path] id: i64) -> rfeign::Result<User>;

// 也可以指定名称（当参数名与占位符不同时）
#[rfeign::get("/users/{user_id}/posts/{post_id}")]
async fn get_post(
    &self,
    #[path(name = "user_id")] uid: i64,
    #[path(name = "post_id")] pid: i64,
) -> rfeign::Result<Post>;
```

## #[query]

查询参数，自动序列化并拼接到 URL：

```rust
#[rfeign::get("/users")]
async fn list_users(
    &self,
    #[query] page: u32,
    #[query] size: u32,
) -> rfeign::Result<Vec<User>>;
// 请求: GET /users?page=1&size=10
```

### 集合格式

对于集合类型参数，可指定序列化格式：

```rust
// multi 格式: ?ids=1&ids=2&ids=3
#[rfeign::get("/users")]
async fn get_by_ids(
    &self,
    #[query(format = "multi")] ids: Vec<i64>,
) -> rfeign::Result<Vec<User>>;

// csv 格式: ?ids=1,2,3
#[rfeign::get("/users")]
async fn get_by_ids_csv(
    &self,
    #[query(format = "csv")] ids: Vec<i64>,
) -> rfeign::Result<Vec<User>>;
```

也可指定查询参数名称：

```rust
#[rfeign::get("/search")]
async fn search(
    &self,
    #[query(name = "q")] keyword: String,
) -> rfeign::Result<Vec<Item>>;
// 请求: GET /search?q=hello
```

## #[body]

请求体参数，自动序列化为 JSON：

```rust
#[derive(Serialize)]
struct CreateUser { name: String, email: String }

#[rfeign::post("/users")]
async fn create_user(&self, #[body] user: CreateUser) -> rfeign::Result<User>;
```

每个方法最多一个 `#[body]` 参数。

## #[header]

动态请求头，在运行时传入值：

```rust
#[rfeign::delete("/users/{id}")]
async fn delete_user(
    &self,
    #[path] id: i64,
    #[header("Authorization")] token: String,
) -> rfeign::Result<()>;
```

调用时传入 header 值：

```rust
api.delete_user(42, "Bearer my-token".to_string()).await?;
```

## #[part]

Multipart 文件上传参数，需配合 `#[multipart]` 使用：

```rust
use rfeign::part::Part;

#[rfeign::post("/upload")]
#[multipart]
async fn upload(
    &self,
    #[part(name = "file")] file: Part,
    #[part(name = "description")] desc: String,
) -> rfeign::Result<UploadResult>;
```

`String` 类型的 `#[part]` 参数自动作为文本字段，`Part` 类型作为文件字段。

## #[derive(RequestParam)]

将结构体作为查询参数对象，生成 `to_query_pairs()` 方法：

```rust
#[derive(rfeign::RequestParam)]
struct ListParams {
    page: u32,
    size: u32,
    #[param(name = "q")]
    keyword: Option<String>,
}

// Option<T> 字段为 None 时不会生成查询参数
let params = ListParams { page: 1, size: 20, keyword: Some("rust".into()) };
let pairs = params.to_query_pairs();
// [("page", "1"), ("size", "20"), ("q", "rust")]
```

在命令式 API 中配合 `.query_pairs()` 使用：

```rust
let params = ListParams { page: 1, size: 20, keyword: None };
client.get("/users")
    .query_pairs(params.to_query_pairs())
    .send().await?;
```
