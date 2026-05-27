use rfeign::{ClientBuilder, ReqwestTransport};
use rfeign::part::Part;

#[tokio::main]
async fn main() -> rfeign::Result<()> {
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
    let body = resp.text()?;
    println!("response: {}", &body[..200.min(body.len())]);

    Ok(())
}
