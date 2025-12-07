use clob_client_rust::client::ClobClient;
// Basic example: fetch an order book and calculate the hash client-side if needed.
// Run with: cargo run --example get_order_book -- <TOKEN_ID>
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token_id = std::env::args().nth(1).unwrap_or_else(|| {
        // Sample token id fallback
        "71321045679252212594626385532706912750332728571942532289631379312455583992563".to_string()
    });
    let host =
        std::env::var("CLOB_API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let chain_id: i64 = std::env::var("CHAIN_ID")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(80002);

    let mut client = ClobClient::new(&host, chain_id, None, None, false);
    let ob = client.get_order_book(&token_id).await?;
    println!(
        "OrderBook market={} bids={} asks={} hash={}",
        ob.market,
        ob.bids.len(),
        ob.asks.len(),
        ob.hash
    );
    Ok(())
}
