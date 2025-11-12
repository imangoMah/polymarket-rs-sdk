use clob_client_rust::client::ClobClient;
use clob_client_rust::order_builder::BuilderConfig;
use clob_client_rust::signer_adapter::EthersSigner;
use clob_client_rust::types::{Side, SignatureType, UserOrder};
use std::sync::Arc;

// Example: create + post a limit order using BuilderConfig convenience method.
// Requires L1 PK and L2 API creds env vars.
// PK, CLOB_API_KEY, CLOB_SECRET, CLOB_PASS_PHRASE must be set.
// Optional: BUILDER_API_KEY, BUILDER_SECRET_B64, BUILDER_PASSPHRASE for builder auth.
// Run: cargo run --example create_and_post_order -- <EXCHANGE_ADDR> <TOKEN_ID>
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pk = std::env::var("PK").expect("PK env var (private key) required");
    let _exchange = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "0xdFE02Eb6733538f8Ea35D585af8DE5958AD99E40".to_string());
    let token_id = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "1234".to_string());

    let l2_key = std::env::var("CLOB_API_KEY").expect("CLOB_API_KEY env var");
    let l2_secret = std::env::var("CLOB_SECRET").expect("CLOB_SECRET env var");
    let l2_pass = std::env::var("CLOB_PASS_PHRASE").expect("CLOB_PASS_PHRASE env var");

    let signer = Arc::new(EthersSigner::new_from_private_key(&pk)?);
    let host =
        std::env::var("CLOB_API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let chain_id: i64 = std::env::var("CHAIN_ID")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(80002);

    let mut client = ClobClient::new(
        &host,
        chain_id,
        Some(signer.clone()),
        Some(clob_client_rust::types::ApiKeyCreds {
            key: l2_key,
            secret: l2_secret,
            passphrase: l2_pass,
        }),
        false,
    )
    .with_builder_config(BuilderConfig {
        tick_size: Some("0.001".to_string()),
        neg_risk: None,
        signature_type: SignatureType::EOA,
        funder_address: None,
    });

    // Optional builder auth
    if let (Ok(b_key), Ok(b_secret), Ok(b_pass)) = (
        std::env::var("BUILDER_API_KEY"),
        std::env::var("BUILDER_SECRET_B64"),
        std::env::var("BUILDER_PASSPHRASE"),
    ) {
        client = client.with_builder_signer(b_key, b_secret, b_pass);
    }

    let user_order = UserOrder {
        token_id: token_id.clone(),
        price: 0.01,
        size: 5.0,
        side: Side::BUY,
        fee_rate_bps: 1.0,
        nonce: None,
        expiration: None,
        taker: None,
    };

    let posted = client.create_and_post_order(user_order, None, None).await?;
    println!(
        "Posted order success={} order_id={}",
        posted.success, posted.order_id
    );
    if let Some(status) = &posted.status {
        println!("Status: {}", status);
    }
    if !posted.error_msg.is_empty() {
        println!("Error message: {}", posted.error_msg);
    }
    if !posted.order_hashes.is_empty() {
        println!("Order hashes: {:?}", posted.order_hashes);
    }
    Ok(())
}
