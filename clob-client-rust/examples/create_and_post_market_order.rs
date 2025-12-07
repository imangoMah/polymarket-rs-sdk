use clob_client_rust::client::ClobClient;
use clob_client_rust::order_builder::BuilderConfig;
use clob_client_rust::signer_adapter::EthersSigner;
use clob_client_rust::types::{OrderType, Side, SignatureType, UserMarketOrder};
use std::sync::Arc;

// Example: create + post a market order using convenience method with auto tick resolution
// Env: PK, CLOB_API_KEY, CLOB_SECRET, CLOB_PASS_PHRASE (optional builder: BUILDER_API_KEY, BUILDER_SECRET_B64, BUILDER_PASSPHRASE)
// Run: cargo run --example create_and_post_market_order -- <EXCHANGE_ADDR> <TOKEN_ID>
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pk = std::env::var("PK").expect("PK env var (private key) required");
    let exchange = std::env::args()
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
        tick_size: None,
        neg_risk: None,
        signature_type: SignatureType::EOA,
        funder_address: None,
    });

    if let (Ok(b_key), Ok(b_secret), Ok(b_pass)) = (
        std::env::var("BUILDER_API_KEY"),
        std::env::var("BUILDER_SECRET_B64"),
        std::env::var("BUILDER_PASSPHRASE"),
    ) {
        client = client.with_builder_signer(b_key, b_secret, b_pass);
    }

    // 外部获取订单簿并计算市价单价格
    let book = client.getOrderBook(&token_id).await?;
    let price = clob_client_rust::order_builder::compute_market_price_from_book(
        &book,
        Side::BUY,
        2.5,
        OrderType::FOK,
    )?;

    let user_market_order = UserMarketOrder {
        token_id: token_id.clone(),
        price,
        amount: 2.5,
        side: Side::BUY,
        fee_rate_bps: 1.0,
        nonce: None,
        taker: None,
        order_type: OrderType::FOK,
    };

    // Using the alias to mirror TS naming (could also call create_and_post_market_order)
    let posted = client
        .createAndPostMarketOrder(user_market_order, None)
        .await?;
    println!(
        "Posted market order success={} order_id={}",
        posted.success, posted.order_id
    );
    if let Some(status) = &posted.status {
        println!("Status: {}", status);
    }
    if let (Some(making), Some(taking)) = (&posted.making_amount, &posted.taking_amount) {
        println!("Making amount: {}, Taking amount: {}", making, taking);
    }
    if !posted.error_msg.is_empty() {
        println!("Error message: {}", posted.error_msg);
    }
    if !posted.order_hashes.is_empty() {
        println!("Order hashes: {:?}", posted.order_hashes);
    }
    println!("exchange={} token_id={}", exchange, token_id);
    Ok(())
}
