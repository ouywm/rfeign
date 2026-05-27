# File Upload

rfeign supports multipart file uploads through both the imperative and declarative APIs.

## Part Type

The `Part` struct represents a file to upload:

```rust
use rfeign::part::Part;

// From raw bytes
let file = Part::from_bytes("report.pdf", "application/pdf", file_bytes);

// Plain text part
let text = Part::text("some description");
```

## Imperative API

Use `.part()` for file parts and `.text_part()` for text fields:

```rust
use rfeign::{ClientBuilder, ReqwestTransport};
use rfeign::part::Part;

let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("https://httpbin.org")
    .build();

let file = Part::from_bytes("hello.txt", "text/plain", b"Hello, rfeign!".to_vec());

let resp = client.post("/anything")
    .text_part("description", "test upload")
    .part("file", file)
    .send()
    .await?;

println!("status: {}", resp.status());
```

You can attach multiple files:

```rust
let file1 = Part::from_bytes("a.txt", "text/plain", b"aaa".to_vec());
let file2 = Part::from_bytes("b.txt", "text/plain", b"bbb".to_vec());

let resp = client.post("/upload")
    .part("files", file1)
    .part("files", file2)
    .text_part("tag", "batch")
    .send()
    .await?;
```

## Declarative API

Mark the method with `#[multipart]` and annotate parameters with `#[part]`:

```rust
use rfeign::part::Part;

#[rfeign::http_client(base_url = "http://localhost:8080")]
trait FileApi {
    #[rfeign::post("/upload")]
    #[rfeign::multipart]
    async fn upload(
        &self,
        #[part(name = "file")] file: Part,
        #[part(name = "description")] desc: String,
    ) -> rfeign::Result<UploadResult>;
}
```

- `Part`-typed parameters are sent as binary file parts
- `String`-typed parameters are sent as text form fields
- The `name` argument sets the multipart field name
