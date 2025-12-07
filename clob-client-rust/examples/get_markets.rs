use clob_client_rust::client::ClobClient;
use clob_client_rust::types::Market;

// Basic example: fetch markets list and a single market summary.
// Run with: cargo run --example get_markets
// Optionally export CLOB_API_URL and CHAIN_ID environment variables.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host =
        std::env::var("CLOB_API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let chain_id: i64 = std::env::var("CHAIN_ID")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(80002);

    let client = ClobClient::new(&host, chain_id, None, None, false);
    // Fetch markets (no params)
    let markets: Vec<Market> = client.get_markets(None).await?;
    println!("Total markets: {}", markets.len());
    if let Some(first) = markets.first() {
        println!("First market id: {}", first.id);
        // Try fetching a detailed summary for the first market (may differ by API shape)
        let summary = client.get_market(&first.id, None).await?;
        println!(
            "Market summary asset_id={} bids={} asks={} hash={}",
            summary.asset_id,
            summary.bids.len(),
            summary.asks.len(),
            summary.hash
        );
    }
    Ok(())
}
