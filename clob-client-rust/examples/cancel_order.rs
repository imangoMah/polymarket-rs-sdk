use clob_client_rust::client::{ClobClient, OrderPayloadParity};
use clob_client_rust::signer_adapter::EthersSigner;
use clob_client_rust::types::ApiKeyCreds;
use std::sync::Arc;

// Example: cancel a single order by id using parity DELETE body { orderID }
// Run: cargo run --example cancel_order -- <ORDER_ID>
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let order_id = std::env::args()
        .nth(1)
        .expect("usage: cancel_order <ORDER_ID>");

    let host =
        std::env::var("CLOB_API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let chain_id: i64 = std::env::var("CHAIN_ID")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(80002);

    let pk = std::env::var("PK").expect("env PK private key required for L1 signatures");
    let signer = Arc::new(EthersSigner::new_from_private_key(&pk)?);

    let creds = ApiKeyCreds {
        key: std::env::var("CLOB_API_KEY").expect("CLOB_API_KEY"),
        secret: std::env::var("CLOB_SECRET").expect("CLOB_SECRET"),
        passphrase: std::env::var("CLOB_PASS_PHRASE").expect("CLOB_PASS_PHRASE"),
    };

    let client = ClobClient::new(&host, chain_id, Some(signer), Some(creds), true);
    let payload = OrderPayloadParity::new(order_id.clone());
    let resp = client.cancelOrder(payload).await?;
    // SDK parity: cancelOrder now returns raw JSON `Value` (like TS `any`). Print full response.
    println!("Cancelled response: {:?}", resp);
    Ok(())
}
