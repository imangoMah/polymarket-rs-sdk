use clob_client_rust::client::ClobClient;
use clob_client_rust::signer_adapter::EthersSigner;
use clob_client_rust::types::ApiKeyCreds;
use std::sync::Arc;

// Example: cancel a list of order IDs (requires L1 signer + L2 creds)
// Run: cargo run --example cancel_orders <ORDER_ID_1> <ORDER_ID_2> ...
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args();
    let _exe = args.next();
    let ids: Vec<String> = args.collect();
    if ids.is_empty() {
        eprintln!("usage: cancel_orders <ORDER_ID_1> <ORDER_ID_2> ...");
        std::process::exit(1);
    }

    let host =
        std::env::var("CLOB_API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let chain_id: i64 = std::env::var("CHAIN_ID")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(80002);
    let pk = std::env::var("PK").expect("PK env var required");
    let signer = Arc::new(EthersSigner::new_from_private_key(&pk)?);

    let creds = ApiKeyCreds {
        key: std::env::var("CLOB_API_KEY").expect("CLOB_API_KEY"),
        secret: std::env::var("CLOB_SECRET").expect("CLOB_SECRET"),
        passphrase: std::env::var("CLOB_PASS_PHRASE").expect("CLOB_PASS_PHRASE"),
    };

    let client = ClobClient::new(&host, chain_id, Some(signer), Some(creds), false);
    let resp = client.cancel_orders(ids.clone()).await?;
    println!(
        "requested cancellation for {} orders; API returned {}",
        ids.len(),
        resp.len()
    );
    for o in resp.iter().take(5) {
        println!("- id={:?} status={:?}", o.id, o.status);
    }
    Ok(())
}
