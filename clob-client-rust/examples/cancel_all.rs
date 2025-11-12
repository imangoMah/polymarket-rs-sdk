use clob_client_rust::client::ClobClient;
use clob_client_rust::signer_adapter::EthersSigner;
use clob_client_rust::types::ApiKeyCreds;
use std::sync::Arc;

// Example: cancel all open orders (requires L1 signer + L2 API creds)
// Run: cargo run --example cancel_all
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
    let result = client.cancel_all().await?;
    println!("cancelled {} orders", result.len());
    for o in result.iter().take(5) {
        println!("- id={:?} status={:?}", o.id, o.status);
    }
    Ok(())
}
