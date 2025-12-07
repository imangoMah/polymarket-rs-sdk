use clob_client_rust::client::ClobClient;
use clob_client_rust::signer_adapter::EthersSigner;
use std::sync::Arc;

// Example: create a new L2 API key (L1 derived)
// Env: PK, CLOB_API_URL, CHAIN_ID
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
    let client = ClobClient::new(&host, chain_id, Some(signer), None, false);

    let nonce = std::env::args().nth(1).and_then(|s| s.parse::<u64>().ok());
    let creds = client.create_api_key(nonce).await?;
    println!(
        "Created API key: key=***{} passphrase=***{} (secret hidden)",
        &creds.key.chars().take(4).collect::<String>(),
        &creds.passphrase.chars().take(4).collect::<String>()
    );
    Ok(())
}
