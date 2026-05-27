use std::time::Duration;

use feignx::{ClientBuilder, ReqwestTransport};
use feignx::part::Part;

#[tokio::test]
async fn test_get_request() {
    let client = ClientBuilder::new(ReqwestTransport::new())
        .base_url("https://httpbin.org")
        .build();

    let resp = client.get("/get").send().await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_post_json() {
    let client = ClientBuilder::new(ReqwestTransport::new())
        .base_url("https://httpbin.org")
        .build();

    let resp = client.post("/anything")
        .json(&serde_json::json!({"name": "feignx"}))
        .unwrap()
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_query_params() {
    let client = ClientBuilder::new(ReqwestTransport::new())
        .base_url("https://httpbin.org")
        .build();

    let resp = client.get("/get")
        .query_pair("page", "1")
        .query_pair("size", "10")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.text().unwrap();
    assert!(body.contains("page"));
}

#[tokio::test]
async fn test_custom_header() {
    let client = ClientBuilder::new(ReqwestTransport::new())
        .base_url("https://httpbin.org")
        .build();

    let resp = client.get("/headers")
        .header("X-Custom", "feignx-test")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.text().unwrap();
    assert!(body.contains("X-Custom"));
}

#[tokio::test]
async fn test_timeout() {
    let client = ClientBuilder::new(ReqwestTransport::new())
        .base_url("https://httpbin.org")
        .build();

    let result = client.get("/delay/5")
        .timeout(Duration::from_millis(500))
        .send()
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_multipart_upload() {
    let client = ClientBuilder::new(ReqwestTransport::new())
        .base_url("https://httpbin.org")
        .build();

    let file = Part::from_bytes("test.txt", "text/plain", b"hello".to_vec());
    let resp = client.post("/anything")
        .text_part("name", "feignx")
        .part("file", file)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_to_curl() {
    let client = ClientBuilder::new(ReqwestTransport::new())
        .base_url("https://httpbin.org")
        .build();

    let curl = client.get("/users")
        .header("Authorization", "Bearer token123")
        .query_pair("page", "1")
        .to_curl();
    assert!(curl.contains("curl -X GET"));
    assert!(curl.contains("authorization: Bearer token123"));
    assert!(curl.contains("page=1"));
}
