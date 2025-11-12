use clob_client_rust::client::ClobClient;
use clob_client_rust::order_builder::OrderBuilder;
use clob_client_rust::signer_adapter::EthersSigner;
use clob_client_rust::types::{Side, SignatureType, UserOrder};

// Example: build and post a limit order.
// Requirements: set PK (private key hex), optionally CLOB_API_URL and CHAIN_ID.
// Run: cargo run --example post_order -- <EXCHANGE_ADDRESS> <TOKEN_ID>
// NOTE: This will attempt a real post; for local dev you may want to stub network or use a mock server.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pk = std::env::var("PK").unwrap_or_else(|_| {
        eprintln!("PK env var not set; using a deterministic dev key (DO NOT USE IN PROD)");
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string()
    });
    let signer = EthersSigner::new_from_private_key(&pk)?;
    let exchange = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "0xdFE02Eb6733538f8Ea35D585af8DE5958AD99E40".to_string());
    let token_id = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "1234".to_string());

    let host =
        std::env::var("CLOB_API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let chain_id: i64 = std::env::var("CHAIN_ID")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(80002);

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

    let ob = OrderBuilder::new(&signer, chain_id, Some(SignatureType::EOA), None);
    let signed = ob.build_order(&exchange, &user_order, "0.001").await?;
    println!(
        "Built order salt={} maker_amount={} taker_amount={} signature={}",
        signed.salt, signed.maker_amount, signed.taker_amount, signed.signature
    );

    // Post order (requires API creds configured in client; omitted here)
    let _client = ClobClient::new(&host, chain_id, None, None, false);
    println!(
        "(Skipping network post in example; integrate client.post_order(&signed) when creds ready)"
    );
    // let posted = client.post_order(&signed).await?;
    // println!("Posted order id={:?} status={:?}", posted.id, posted.status);
    Ok(())
}
