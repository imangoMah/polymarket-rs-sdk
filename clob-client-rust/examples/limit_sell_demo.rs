// 限价卖单示例 (Limit Sell Order with Builder)
// 演示如何创建并提交限价卖单,包括使用 Builder 配置进行订单归属
//
// 环境变量:
//   必需:
//     PK - 私钥
//     CLOB_API_KEY, CLOB_SECRET, CLOB_PASS_PHRASE - API 凭证
//   可选:
//     CLOB_API_URL - API 地址,默认 https://clob.polymarket.com
//     CHAIN_ID - 链 ID,默认 137 (Polygon)
//     BUILDER_API_KEY, BUILDER_SECRET_B64, BUILDER_PASSPHRASE - Builder 凭证
//
// 参数:
//   cargo run --example limit_sell_demo -- <TOKEN_ID> [PRICE] [SIZE]
//   例如: cargo run --example limit_sell_demo -- 123456 0.58 10.0
//
// 运行: cargo run --example limit_sell_demo

use clob_client_rust::client::ClobClient;
use clob_client_rust::order_builder::BuilderConfig;
use clob_client_rust::signer_adapter::EthersSigner;
use clob_client_rust::types::{ApiKeyCreds, Side, SignatureType, UserOrder};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📉 限价卖单示例\n");

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

    // 2. 解析命令行参数
    let token_id = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("TOKEN_ID").ok())
        .expect("需要提供 TOKEN_ID (作为参数或环境变量)");

    let price: f64 = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.58); // 默认价格

    let size: f64 = std::env::args()
        .nth(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(10.0); // 默认数量

    println!("配置:");
    println!("  Host: {}", host);
    println!("  Chain ID: {}", chain_id);
    println!("  Token ID: {}", token_id);
    println!("  Price: {}", price);
    println!("  Size: {}", size);
    println!();

    // 3. 初始化 Client
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
    )
    .with_builder_config(BuilderConfig {
        tick_size: Some("0.01".to_string()),
        neg_risk: Some(false),
        signature_type: SignatureType::EOA,
        funder_address: None,
    });

    // 4. 可选: 添加 Builder 签名器 (用于订单归属)
    if let (Ok(b_key), Ok(b_secret), Ok(b_pass)) = (
        std::env::var("BUILDER_API_KEY"),
        std::env::var("BUILDER_SECRET_B64"),
        std::env::var("BUILDER_PASSPHRASE"),
    ) {
        println!("✅ 检测到 Builder 配置,将添加订单归属");
        client = client.with_builder_signer(b_key, b_secret, b_pass);
    } else {
        println!("ℹ️  未配置 Builder (可选)");
    }
    println!();

    println!("✅ Client 初始化成功\n");

    // 5. 创建限价卖单
    println!("=== 创建限价卖单 ===");
    let user_order = UserOrder {
        token_id: token_id.clone(),
        price,
        size,
        side: Side::SELL,  // 注意这里是 SELL
        fee_rate_bps: 1.0, // 0.01% 费率 (必填)
        nonce: None,       // 自动生成
        expiration: None,  // 使用默认过期时间
        taker: None,       // 任何人可成交
    };

    println!("订单信息:");
    println!("  Token ID: {}", user_order.token_id);
    println!("  方向: 卖单 (SELL)");
    println!("  价格: {}", user_order.price);
    println!("  数量: {}", user_order.size);
    println!("  费率: {}bps", user_order.fee_rate_bps);
    println!();

    // 6. 提交订单
    println!("=== 提交订单 ===");
    match client.create_and_post_order(user_order, None, None).await {
        Ok(posted) => {
            println!("✅ 订单提交成功!");
            println!("  Success: {}", posted.success);
            println!("  Order ID: {}", posted.order_id);

            if let Some(status) = &posted.status {
                println!("  Status: {}", status);
            }
            if let Some(taking) = &posted.taking_amount {
                println!("  Taking Amount: {}", taking);
            }
            if let Some(making) = &posted.making_amount {
                println!("  Making Amount: {}", making);
            }
            if !posted.order_hashes.is_empty() {
                println!("  Order Hashes: {:?}", posted.order_hashes);
            }

            if !posted.error_msg.is_empty() {
                println!("  Error Message: {}", posted.error_msg);
            }

            println!("\n💡 可以使用以下命令查询订单状态:");
            println!("   cargo run --example get_order -- {}", posted.order_id);
        }
        Err(e) => {
            eprintln!("❌ 订单提交失败: {}", e);
            eprintln!("\n可能的原因:");
            eprintln!("  1. TOKEN_ID 无效");
            eprintln!("  2. 价格超出范围 (0-1)");
            eprintln!("  3. 持仓不足 (卖单需要有持仓)");
            eprintln!("  4. API 凭证无效");
        }
    }

    println!("\n✅ 限价卖单示例完成!");
    Ok(())
}
