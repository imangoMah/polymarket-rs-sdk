use clob_client_rust::client::ClobClient;
use clob_client_rust::signer_adapter::EthersSigner;
use clob_client_rust::types::ApiKeyCreds;
use std::sync::Arc;

// Example: list L2 API keys (requires L1 signer + existing L2 creds)
// Env: PK, CLOB_API_URL, CHAIN_ID, CLOB_API_KEY, CLOB_SECRET, CLOB_PASS_PHRASE
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
    let keys = client.get_api_keys().await?;
    println!("{} API keys:", keys.len());
    for (i, k) in keys.iter().enumerate() {
        println!(
            "{}: key=***{} passphrase=***{}",
            i,
            &k.key.chars().take(4).collect::<String>(),
            &k.passphrase.chars().take(4).collect::<String>()
        );
    }
    Ok(())
}
