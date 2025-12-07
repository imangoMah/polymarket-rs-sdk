use clob_client_rust::client::ClobClient;
use clob_client_rust::types::SignedOrder;

// Example: fetch open orders (optionally filter by token_id)
// Run: cargo run --example get_open_orders [token_id]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host =
        std::env::var("CLOB_API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let chain_id: i64 = std::env::var("CHAIN_ID")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(80002);

    let token_id_opt = std::env::args().nth(1);

    let client = ClobClient::new(&host, chain_id, None, None, false);

    let mut params = std::collections::HashMap::new();
    if let Some(token_id) = token_id_opt {
        params.insert("token_id".to_string(), token_id);
    }

    let orders: Vec<SignedOrder> = client
        .get_open_orders(if params.is_empty() {
            None
        } else {
            Some(params)
        })
        .await?;

    println!("open orders count = {}", orders.len());
    for o in orders.iter().take(3) {
        println!(
            "- salt={} token_id={} side={:?} maker={} taker={}",
            o.salt, o.token_id, o.side, o.maker_amount, o.taker_amount
        );
    }
    Ok(())
}
