// 订单查询示例 (Query Orders)
// 演示如何查询市场信息、订单簿、未结订单
//
// 环境变量:
//   PK - 私钥
//   CLOB_API_KEY, CLOB_SECRET, CLOB_PASS_PHRASE - API 凭证
//   CLOB_API_URL (可选) - API 地址,默认 https://clob.polymarket.com
//   CHAIN_ID (可选) - 链 ID,默认 137 (Polygon)
//   TOKEN_ID (可选) - 用于查询订单簿的 Token ID
//
// 运行: cargo run --example query_orders_demo

use clob_client_rust::client::ClobClient;
use clob_client_rust::signer_adapter::EthersSigner;
use clob_client_rust::types::ApiKeyCreds;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 订单查询示例\n");

    // 1. 读取环境变量
    let pk = std::env::var("PK").expect("需要设置 PK 环境变量 (私钥)");
    let api_key = std::env::var("CLOB_API_KEY").expect("需要设置 CLOB_API_KEY");
    let api_secret = std::env::var("CLOB_SECRET").expect("需要设置 CLOB_SECRET");
    let api_passphrase = std::env::var("CLOB_PASS_PHRASE").expect("需要设置 CLOB_PASS_PHRASE");

    let host =
        std::env::var("CLOB_API_URL").unwrap_or_else(|_| "https://clob.polymarket.com".to_string());
    let chain_id: i64 = std::env::var("CHAIN_ID")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(137);

    println!("配置:");
    println!("  Host: {}", host);
    println!("  Chain ID: {}", chain_id);
    println!();

    // 2. 初始化 Client
    let signer = Arc::new(EthersSigner::new_from_private_key(&pk)?);
    let mut client = ClobClient::new(
        &host,
        chain_id,
        Some(signer.clone()),
        Some(ApiKeyCreds {
            key: api_key,
            secret: api_secret,
            passphrase: api_passphrase,
        }),
        false,
    );

    println!("✅ Client 初始化成功\n");

    // 3. 查询市场列表
    println!("=== 1. 查询市场列表 ===");
    match client.get_markets(None).await {
        Ok(markets) => {
            println!("找到 {} 个市场", markets.len());
            if let Some(market) = markets.first() {
                println!("\n示例市场:");
                println!("  ID: {}", market.id);
                if let Some(name) = &market.name {
                    println!("  名称: {}", name);
                }
                if let Some(tick_size) = &market.tick_size {
                    println!("  Tick Size: {}", tick_size);
                }
                if let Some(neg_risk) = market.neg_risk {
                    println!("  Neg Risk: {}", neg_risk);
                }
            }
        }
        Err(e) => eprintln!("❌ 查询市场列表失败: {}", e),
    }
    println!();

    // 4. 查询未结订单
    println!("=== 2. 查询未结订单 ===");
    match client.get_open_orders(None).await {
        Ok(orders) => {
            println!("找到 {} 个未结订单", orders.len());
            for (i, order) in orders.iter().take(3).enumerate() {
                println!("\n订单 #{}:", i + 1);
                println!("  Token ID: {}", order.token_id);
                println!("  Maker Amount: {}", order.maker_amount);
                println!("  Taker Amount: {}", order.taker_amount);
                println!("  Side: {:?}", order.side);
                println!("  Fee Rate: {}bps", order.fee_rate_bps);
            }
            if orders.is_empty() {
                println!("当前没有未结订单");
            }
        }
        Err(e) => eprintln!("❌ 查询未结订单失败: {}", e),
    }
    println!();

    // 5. 查询特定 Token 的订单簿 (如果提供了 TOKEN_ID)
    if let Ok(token_id) = std::env::var("TOKEN_ID") {
        println!("=== 3. 查询订单簿 (Token: {}) ===", token_id);
        match client.get_order_book(&token_id).await {
            Ok(book) => {
                println!("市场: {}", book.market);
                println!("Asset ID: {}", book.asset_id);
                println!("Tick Size: {}", book.tick_size);
                println!("\n买单 (Bids): {} 个", book.bids.len());
                for (i, bid) in book.bids.iter().take(5).enumerate() {
                    println!("  #{}: 价格={}, 数量={}", i + 1, bid.price, bid.size);
                }
                println!("\n卖单 (Asks): {} 个", book.asks.len());
                for (i, ask) in book.asks.iter().take(5).enumerate() {
                    println!("  #{}: 价格={}, 数量={}", i + 1, ask.price, ask.size);
                }
            }
            Err(e) => eprintln!("❌ 查询订单簿失败: {}", e),
        }
        println!();
    } else {
        println!("💡 提示: 设置 TOKEN_ID 环境变量以查询特定 Token 的订单簿\n");
    }

    println!("✅ 查询示例完成!");
    Ok(())
}
