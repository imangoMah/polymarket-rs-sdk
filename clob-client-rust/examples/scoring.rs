use clob_client_rust::client::ClobClient;

// Example: check if orders are scoring (builder endpoints). If builder auth is required, ensure client has builder signer.
// Run: cargo run --example scoring [order_id1,order_id2,...]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host =
        std::env::var("CLOB_API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let chain_id: i64 = std::env::var("CHAIN_ID")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(80002);

    let mut client = ClobClient::new(&host, chain_id, None, None, false);
    // Optional builder auth from env
    if let (Ok(b_key), Ok(b_secret), Ok(b_pass)) = (
        std::env::var("BUILDER_API_KEY"),
        std::env::var("BUILDER_SECRET_B64"),
        std::env::var("BUILDER_PASSPHRASE"),
    ) {
        client = client.with_builder_signer(b_key, b_secret, b_pass);
    }

    // If a comma-separated list is provided, call areOrdersScoring; otherwise call isOrderScoring without params
    if let Some(list) = std::env::args().nth(1) {
        let ids: Vec<String> = list
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let resp = client.areOrdersScoring(Some(ids)).await?;
        println!("areOrdersScoring: {:?}", resp);
    } else {
        let resp = client.isOrderScoring(None).await?;
        println!("isOrderScoring: {:?}", resp);
    }
    Ok(())
}
