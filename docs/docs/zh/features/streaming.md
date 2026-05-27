# 流式响应

rfeign 支持流式读取响应体，适用于大文件下载、SSE 事件流等场景。

## ByteStream 类型

`ByteStream` 是响应体流的类型别名：

```rust
// 定义
pub type ByteStream = Pin<Box<dyn Stream<Item = Result<Bytes, Error>> + Send>>;
```

## .send_streaming() 方法

使用 `.send_streaming()` 代替 `.send()` 获取流式响应：

```rust
use futures_util::StreamExt;
use rfeign::{ClientBuilder, ReqwestTransport};

let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("https://api.example.com")
    .build();

let streaming_resp = client.get("/large-file")
    .send_streaming()
    .await?;

println!("status: {}", streaming_resp.status);
println!("content-type: {:?}", streaming_resp.headers.get("content-type"));

// 逐块读取响应体
let mut stream = streaming_resp.body;
while let Some(chunk) = stream.next().await {
    let bytes = chunk?;
    // 处理每个数据块
    println!("received {} bytes", bytes.len());
}
```

## StreamingResponse

`StreamingResponse` 包含三个字段：

```rust
pub struct StreamingResponse {
    pub status: u16,           // HTTP 状态码
    pub headers: HeaderMap,    // 响应头
    pub body: ByteStream,      // 响应体流
}
```

## SSE 事件流示例

```rust
use futures_util::StreamExt;

let resp = client.get("/events")
    .header("Accept", "text/event-stream")
    .send_streaming()
    .await?;

let mut stream = resp.body;
while let Some(chunk) = stream.next().await {
    let bytes = chunk?;
    let text = String::from_utf8_lossy(&bytes);
    for line in text.lines() {
        if line.starts_with("data:") {
            println!("event: {}", &line[5..]);
        }
    }
}
```
