use std::time::Duration;

use feignx::{BackoffStrategy, ClientBuilder, LogLevel, LoggingMiddleware, Retry, ReqwestTransport};

#[tokio::main]
async fn main() -> feignx::Result<()> {
    let client = ClientBuilder::new(ReqwestTransport::new())
        .base_url("https://httpbin.org")
        .middleware(LoggingMiddleware::new(LogLevel::Basic))
        .middleware(
            Retry::new(3).backoff(BackoffStrategy::Fixed(Duration::from_millis(200)))
        )
        .build();

    let resp = client.get("/get").send().await?;
    println!("status: {}", resp.status());

    let resp = client.get("/status/500").send().await?;
    println!("status: {}", resp.status());

    let resp = client.get("/delay/5")
        .timeout(Duration::from_secs(1))
        .send()
        .await;
    match resp {
        Ok(r) => println!("status: {}", r.status()),
        Err(e) => println!("expected timeout: {}", e),
    }

    Ok(())
}
