use clob_client_rust::client::ClobClient;
use clob_client_rust::types::Reward;

// Example: fetch rewards (earnings) for user for a day.
// Depending on server, may require query params like user or date; adjust as needed.
// Run: cargo run --example rewards [user=<addr>] [date=YYYY-MM-DD]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host =
        std::env::var("CLOB_API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let chain_id: i64 = std::env::var("CHAIN_ID")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(80002);
    let client = ClobClient::new(&host, chain_id, None, None, false);

    // Parse optional params from args in form key=value
    let mut params_map = std::collections::HashMap::new();
    for arg in std::env::args().skip(1) {
        if let Some((k, v)) = arg.split_once('=') {
            params_map.insert(k.to_string(), v.to_string());
        }
    }
    let params = if params_map.is_empty() {
        None
    } else {
        Some(params_map)
    };

    let rewards: Vec<Reward> = client.get_rewards_user_for_day_typed(params).await?;
    println!("rewards entries: {}", rewards.len());
    for r in rewards.iter().take(5) {
        println!(
            "- market={:?} amount={:?} timestamp={:?}",
            r.market, r.amount, r.timestamp
        );
    }
    Ok(())
}
