use clob_client_rust::client::ClobClient;
use clob_client_rust::types::MarketPrice;

// Example: fetch midpoint, prices, spreads for a (possibly filtered) market set
// Run: cargo run --example get_prices [market_id]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host =
        std::env::var("CLOB_API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let chain_id: i64 = std::env::var("CHAIN_ID")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(80002);
    let client = ClobClient::new(&host, chain_id, None, None, false);

    let market_filter = std::env::args().nth(1);
    let mut params = std::collections::HashMap::new();
    if let Some(mid) = market_filter.clone() {
        params.insert("market_id".to_string(), mid);
    }

    let midpoint: Vec<MarketPrice> = client
        .get_midpoint(if params.is_empty() {
            None
        } else {
            Some(params.clone())
        })
        .await
        .unwrap_or_default();
    let spreads: Vec<MarketPrice> = client
        .get_spreads(if params.is_empty() {
            None
        } else {
            Some(params.clone())
        })
        .await
        .unwrap_or_default();
    let prices: Vec<MarketPrice> = client
        .get_prices(if params.is_empty() {
            None
        } else {
            Some(params)
        })
        .await
        .unwrap_or_default();

    println!(
        "midpoint entries={} spreads entries={} prices entries={}",
        midpoint.len(),
        spreads.len(),
        prices.len()
    );
    if let Some(m) = midpoint.first() {
        println!("sample midpoint: t={} p={}", m.t, m.p);
    }
    Ok(())
}
