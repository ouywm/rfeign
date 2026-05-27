# Streaming

rfeign supports streaming responses for large payloads or server-sent events.

## ByteStream Type

`ByteStream` is a pinned, boxed async stream of `Result<Bytes, Error>`:

```rust
pub type ByteStream = Pin<Box<dyn Stream<Item = Result<Bytes, Error>> + Send>>;
```

## StreamingResponse

The `StreamingResponse` struct provides access to status, headers, and the body stream:

```rust
pub struct StreamingResponse {
    pub status: u16,
    pub headers: http::HeaderMap,
    pub body: ByteStream,
}
```

## Sending a Streaming Request

Use `.send_streaming()` instead of `.send()` to get a streaming response:

```rust
use futures_util::StreamExt;
use rfeign::{ClientBuilder, ReqwestTransport};

let client = ClientBuilder::new(ReqwestTransport::new())
    .base_url("https://api.example.com")
    .build();

let stream_resp = client.get("/large-file")
    .send_streaming()
    .await?;

println!("status: {}", stream_resp.status);

// Consume the stream chunk by chunk
let mut body = stream_resp.body;
while let Some(chunk) = body.next().await {
    let bytes = chunk?;
    println!("received {} bytes", bytes.len());
}
```

## Server-Sent Events Example

```rust
use futures_util::StreamExt;

let stream_resp = client.get("/events")
    .header("Accept", "text/event-stream")
    .send_streaming()
    .await?;

let mut body = stream_resp.body;
while let Some(chunk) = body.next().await {
    let bytes = chunk?;
    let text = String::from_utf8_lossy(&bytes);
    println!("event: {}", text);
}
```
