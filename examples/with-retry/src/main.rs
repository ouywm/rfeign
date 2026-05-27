use std::time::Duration;

use rfeign::{ClientBuilder, ReqwestTransport};

#[tokio::main]
async fn main() -> rfeign::Result<()> {
    let transport = ReqwestTransport::builder()
        .retry(3)
        .build();

    let client = ClientBuilder::new(transport)
        .base_url("https://httpbin.org")
        .build();

    let resp = client.get("/get").send().await?;
    println!("GET /get -> {}", resp.status());

    let resp = client.get("/status/503").send().await?;
    println!("GET /status/503 -> {} (after retries)", resp.status());

    let resp = client.get("/delay/5")
        .timeout(Duration::from_secs(1))
        .send()
        .await;
    match resp {
        Ok(r) => println!("GET /delay/5 -> {}", r.status()),
        Err(e) => println!("GET /delay/5 -> error: {}", e),
    }

    Ok(())
}
