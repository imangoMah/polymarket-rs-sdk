use clob_client_rust::client::ClobClient;
use clob_client_rust::signer_adapter::EthersSigner;
use clob_client_rust::types::{ApiKeyCreds, Trade};
use std::sync::Arc;

// Example: fetch trades (first page only by default). Requires L1 signer + L2 creds.
// Run: cargo run --example get_trades [market_id]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host =
        std::env::var("CLOB_API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let chain_id: i64 = std::env::var("CHAIN_ID")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(80002);

    let pk = std::env::var("PK").expect("PK env var required for L1 signatures");
    let signer = Arc::new(EthersSigner::new_from_private_key(&pk)?);
    let creds = ApiKeyCreds {
        key: std::env::var("CLOB_API_KEY").expect("CLOB_API_KEY"),
        secret: std::env::var("CLOB_SECRET").expect("CLOB_SECRET"),
        passphrase: std::env::var("CLOB_PASS_PHRASE").expect("CLOB_PASS_PHRASE"),
    };

    let client = ClobClient::new(&host, chain_id, Some(signer), Some(creds), false);

    let mut params = std::collections::HashMap::new();
    if let Some(mid) = std::env::args().nth(1) {
        params.insert("market_id".to_string(), mid);
    }

    let trades: Vec<Trade> = client
        .get_trades_typed(
            if params.is_empty() {
                None
            } else {
                Some(params)
            },
            true,
            None,
        )
        .await?;
    println!("trades fetched: {}", trades.len());
    for t in trades.iter().take(5) {
        println!(
            "- id={:?} token_id={:?} price={:?} size={:?}",
            t.id, t.token_id, t.price, t.size
        );
    }
    Ok(())
}
