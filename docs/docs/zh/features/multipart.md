# 文件上传

rfeign 支持 multipart/form-data 文件上传，提供声明式和命令式两种方式。

## Part 类型

`Part` 是文件上传的核心类型：

```rust
use rfeign::part::Part;

// 从字节数组创建
let file = Part::from_bytes(
    "report.pdf",           // 文件名
    "application/pdf",      // Content-Type
    std::fs::read("report.pdf")?,  // 文件内容
);

// 创建纯文本 Part
let text = Part::text("hello world");
```

## 命令式上传

使用 `.part()` 和 `.text_part()` 方法：

```rust
use rfeign::{ClientBuilder, ReqwestTransport};
use rfeign::part::Part;

let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("https://api.example.com")
    .build();

let file = Part::from_bytes("photo.jpg", "image/jpeg", image_bytes);

let resp = client.post("/upload")
    .text_part("description", "My photo")
    .part("file", file)
    .send()
    .await?;

println!("status: {}", resp.status());
```

`.text_part(name, value)` 添加文本字段，`.part(name, Part)` 添加文件字段。

## 声明式上传

在 trait 方法上使用 `#[multipart]` 和 `#[part]`：

```rust
use rfeign::part::Part;

#[rfeign::http_client(base_url = "http://localhost:8080")]
trait FileApi {
    #[rfeign::post("/upload")]
    #[multipart]
    async fn upload(
        &self,
        #[part(name = "file")] file: Part,
        #[part(name = "description")] desc: String,
    ) -> rfeign::Result<UploadResult>;
}
```

- `Part` 类型参数作为文件字段上传
- `String` 类型参数作为文本字段
- `#[part(name = "x")]` 指定表单字段名称
