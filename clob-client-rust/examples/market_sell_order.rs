use clob_client_rust::client::ClobClient;
use clob_client_rust::order_builder::{OrderBuilder, compute_market_price_from_book};
use clob_client_rust::signer_adapter::EthersSigner;
use clob_client_rust::types::{OrderType, Side, SignatureType, UserMarketOrder};
// no extra imports

// Example: build a market SELL order
// Run: cargo run --example market_sell_order -- <EXCHANGE_ADDRESS> <TOKEN_ID>
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

    // 外部获取订单簿并计算市价价格
    let mut client = ClobClient::new(&host, chain_id, None, None, false);
    let book = client.getOrderBook(&token_id).await?;
    let price = compute_market_price_from_book(&book, Side::SELL, 5.0, OrderType::FOK)?;

    let mso = UserMarketOrder {
        token_id,
        price,
        amount: 5.0,
        side: Side::SELL,
        fee_rate_bps: 1.0,
        nonce: None,
        taker: None,
        order_type: OrderType::FOK,
    };

    let ob = OrderBuilder::new(&signer, chain_id, Some(SignatureType::EOA), None);
    let signed = ob.build_market_order(&exchange, &mso, "0.001").await?;
    println!(
        "Market SELL signed order: salt={} maker_amount={} taker_amount={} signature={}",
        signed.salt, signed.maker_amount, signed.taker_amount, signed.signature
    );
    Ok(())
}
