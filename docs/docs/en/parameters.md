# Parameter Attributes

Parameter attributes tell rfeign how to map method arguments to HTTP request components.

## #[path]

Substitutes the argument into a `{placeholder}` in the URL path.

```rust
#[rfeign::get("/users/{id}")]
async fn get_user(&self, #[path] id: i64) -> rfeign::Result<User>;
```

Use `#[path("name")]` when the parameter name differs from the placeholder:

```rust
#[rfeign::get("/users/{user_id}/posts/{post_id}")]
async fn get_post(
    &self,
    #[path("user_id")] uid: i64,
    #[path("post_id")] pid: i64,
) -> rfeign::Result<Post>;
```

## #[query]

Appends the argument as a query parameter.

```rust
#[rfeign::get("/users")]
async fn list_users(
    &self,
    #[query] page: u32,
    #[query] size: u32,
) -> rfeign::Result<Vec<User>>;
// GET /users?page=1&size=20
```

### Custom name

```rust
#[rfeign::get("/search")]
async fn search(&self, #[query(name = "q")] keyword: String) -> rfeign::Result<Vec<Item>>;
// GET /search?q=rust
```

### Collection formats

For parameters that accept multiple values:

```rust
// format = "multi" -> ?id=1&id=2&id=3
#[rfeign::get("/users")]
async fn get_by_ids(
    &self,
    #[query(format = "multi")] id: Vec<i64>,
) -> rfeign::Result<Vec<User>>;

// format = "csv" -> ?ids=1,2,3
#[rfeign::get("/users")]
async fn get_by_ids_csv(
    &self,
    #[query(name = "ids", format = "csv")] ids: Vec<i64>,
) -> rfeign::Result<Vec<User>>;
```

## #[body]

Serializes the argument as the JSON request body.

```rust
#[rfeign::post("/users")]
async fn create_user(&self, #[body] user: CreateUser) -> rfeign::Result<User>;
```

Only one `#[body]` parameter is allowed per method.

## #[header("name")]

Passes the argument as a dynamic request header.

```rust
#[rfeign::delete("/users/{id}")]
async fn delete_user(
    &self,
    #[path] id: i64,
    #[header("Authorization")] token: String,
) -> rfeign::Result<()>;
```

## #[part]

Used with `#[multipart]` methods for file upload (see [File Upload](./multipart.md)):

```rust
#[rfeign::post("/upload")]
#[rfeign::multipart]
async fn upload(
    &self,
    #[part(name = "file")] file: rfeign::part::Part,
    #[part(name = "description")] desc: String,
) -> rfeign::Result<UploadResult>;
```

String-typed `#[part]` arguments are sent as text parts; `Part`-typed arguments are sent as file parts.

## #[derive(RequestParam)]

For complex query parameter objects, derive `RequestParam` to generate a `to_query_pairs()` method:

```rust
use rfeign::RequestParam;

#[derive(RequestParam)]
struct ListParams {
    page: u32,
    size: u32,
    #[param(name = "q")]
    keyword: Option<String>,
}
```

`Option<T>` fields are skipped when `None`. Use `#[param(name = "...")]` to customize the query key.

This is used with the imperative API:

```rust
let params = ListParams { page: 1, size: 20, keyword: Some("alice".into()) };
let resp = client.get("/users")
    .query_pairs(params.to_query_pairs())
    .send()
    .await?;
// GET /users?page=1&size=20&q=alice
```
