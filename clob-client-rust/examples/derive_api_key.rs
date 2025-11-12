use clob_client_rust::client::ClobClient;
use clob_client_rust::signer_adapter::EthersSigner;
use std::sync::Arc;

// Example: derive existing API key creds via L1 signer (no nonce)
// Env: PK, CLOB_API_URL, CHAIN_ID
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host =
        std::env::var("CLOB_API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let chain_id: i64 = std::env::var("CHAIN_ID")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(80002);
    let pk = std::env::var("PK").expect("PK env var required");
    let signer = Arc::new(EthersSigner::new_from_private_key(&pk)?);

    let client = ClobClient::new(&host, chain_id, Some(signer), None, false);
    let creds = client.derive_api_key(None).await?;
    println!(
        "Derived API key: key=***{} passphrase=***{}",
        &creds.key.chars().take(4).collect::<String>(),
        &creds.passphrase.chars().take(4).collect::<String>()
    );
    Ok(())
}
