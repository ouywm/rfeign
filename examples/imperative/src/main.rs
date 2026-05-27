use rfeign::ReqwestTransport;
use rfeign::client::ClientBuilder;
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

#[derive(Debug, Serialize)]
struct ListParams {
    page: u32,
    size: u32,
}

#[tokio::main]
async fn main() -> rfeign::Result<()> {
    let client = ClientBuilder::new(ReqwestTransport::new())
        .base_url("https://httpbin.org")
        .build();

    let resp = client.get("/get")
        .query_pair("page", "1")
        .header("X-App", "rfeign")
        .send()
        .await?;
    println!("status: {}", resp.status());

    let resp = client.get("/get")
        .query(&ListParams { page: 1, size: 10 })
        .send()
        .await?;
    println!("status: {}", resp.status());

    let resp = client.post("/post")
        .json(&CreateUser { name: "alice".into() })?
        .send()
        .await?;
    println!("status: {}", resp.status());

    Ok(())
}
