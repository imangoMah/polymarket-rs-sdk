// 市价卖单示例 (Market Sell Order with Builder)
// 演示如何创建并提交市价卖单,包括使用 Builder 配置进行订单归属
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
//   cargo run --example market_sell_demo -- <TOKEN_ID> [AMOUNT]
//   例如: cargo run --example market_sell_demo -- 123456 5.0
//
// 注意: 市价单会立即以当前最佳价格成交,卖单需要有持仓
//
// 运行: cargo run --example market_sell_demo

use clob_client_rust::client::ClobClient;
use clob_client_rust::order_builder::BuilderConfig;
use clob_client_rust::signer_adapter::EthersSigner;
use clob_client_rust::types::{ApiKeyCreds, OrderType, Side, SignatureType, UserMarketOrder};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("💸 市价卖单示例\n");

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

    let amount: f64 = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(5.0); // 默认数量

    println!("配置:");
    println!("  Host: {}", host);
    println!("  Chain ID: {}", chain_id);
    println!("  Token ID: {}", token_id);
    println!("  Amount: {}", amount);
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
        tick_size: None, // 市价单自动使用市场 tick size
        neg_risk: None,
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

    // 5. 创建市价卖单
    println!("=== 创建市价卖单 ===");
    println!("=== 预获取订单簿并计算市价单价格 ===");
    let order_book = client.getOrderBook(&token_id).await?;
    let price = clob_client_rust::order_builder::compute_market_price_from_book(
        &order_book,
        Side::SELL,
        amount,
        OrderType::FOK,
    )?;
    println!("  计算得到市价成交参考价: {:.6}", price);

    let user_market_order = UserMarketOrder {
        token_id: token_id.clone(),
        price, // 必填外部计算
        amount,
        side: Side::SELL,           // 注意这里是 SELL
        fee_rate_bps: 0.0,          // 示例费率
        nonce: None,                // 自动生成
        taker: None,                // 任何人可成交
        order_type: OrderType::FOK, // 必填
    };

    println!("订单信息:");
    println!("  Token ID: {}", user_market_order.token_id);
    println!("  方向: 卖单 (SELL)");
    println!("  类型: 市价单 - FOK (Fill or Kill)");
    println!("  数量: {}", user_market_order.amount);
    println!("  费率: {} bps", user_market_order.fee_rate_bps);
    println!("\n⚠️  市价单将以当前最佳买价立即成交");
    println!("⚠️  确保您有足够的持仓进行卖出");
    println!("⚠️  FOK 订单类型: 必须全部成交,否则取消");
    println!();

    // 6. 提交订单
    println!("=== 提交订单 ===");
    match client
        .createAndPostMarketOrder(user_market_order, None)
        .await
    {
        Ok(posted) => {
            println!("✅ 市价卖单提交成功!");
            println!("  Success: {}", posted.success);
            println!("  Order ID: {}", posted.order_id);

            if let Some(status) = &posted.status {
                println!("  Status: {}", status);
            }

            if !posted.error_msg.is_empty() {
                println!("  Error Message: {}", posted.error_msg);
            }

            if !posted.order_hashes.is_empty() {
                println!("  Order Hashes: {:?}", posted.order_hashes);
            }

            // 计算实际成交价格 (如果有 making_amount 和 taking_amount)
            if let (Some(making_str), Some(taking_str)) =
                (&posted.making_amount, &posted.taking_amount)
            {
                println!("  Making Amount: {}", making_str);
                println!("  Taking Amount: {}", taking_str);

                if let (Ok(making_amt), Ok(taking_amt)) =
                    (making_str.parse::<f64>(), taking_str.parse::<f64>())
                {
                    if taking_amt > 0.0 {
                        let avg_price = making_amt / taking_amt;
                        println!("  实际成交价: ~{:.4}", avg_price);
                    }
                }
            }

            println!("\n💡 可以使用以下命令查询订单状态:");
            println!("   cargo run --example get_order -- {}", posted.order_id);
        }
        Err(e) => {
            eprintln!("❌ 订单提交失败: {}", e);
            eprintln!("\n可能的原因:");
            eprintln!("  1. TOKEN_ID 无效");
            eprintln!("  2. 持仓不足 (卖单需要有持仓)");
            eprintln!("  3. 订单簿深度不足 (没有足够的买单)");
            eprintln!("  4. API 凭证无效");
        }
    }

    println!("\n✅ 市价卖单示例完成!");
    Ok(())
}
