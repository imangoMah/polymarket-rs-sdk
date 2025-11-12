use builder_relayer_client_rust::signer::DummySigner;
/// 交易监控示例
///
/// 演示如何监控和追踪 Relayer 交易状态
use builder_relayer_client_rust::{
    OperationType, RelayClient, RelayerTransactionState, SafeTransaction,
};
use builder_signing_sdk_rs::BuilderApiKeyCreds;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 交易监控示例\n");

    let relayer_client = initialize_client().await?;

    // 示例1: 使用 poll_until_state 自动等待
    println!("1️⃣ 自动等待交易确认...");
    auto_wait_example(&relayer_client).await?;
    println!();

    // 示例2: 手动轮询状态
    println!("2️⃣ 手动轮询交易状态...");
    manual_polling_example(&relayer_client).await?;
    println!();

    // 示例3: 带超时的监控
    println!("3️⃣ 带超时的交易监控...");
    timeout_monitoring_example(&relayer_client).await?;
    println!();

    println!("🎉 所有监控示例完成!");

    Ok(())
}

async fn initialize_client() -> Result<RelayClient, Box<dyn std::error::Error>> {
    let private_key = std::env::var("PRIVATE_KEY")?;
    let signer = DummySigner::new(&private_key)?;

    let relayer_client = RelayClient::new("https://relayer-v2.polymarket.com/", 137)
        .with_signer(Box::new(signer.clone()), Box::new(signer))
        .with_builder_api_key(BuilderApiKeyCreds {
            key: std::env::var("BUILDER_API_KEY")?,
            secret: std::env::var("BUILDER_SECRET")?,
            passphrase: std::env::var("BUILDER_PASS_PHRASE")?,
        });

    Ok(relayer_client)
}

/// 示例1: 自动等待 (推荐方式)
async fn auto_wait_example(client: &RelayClient) -> Result<(), Box<dyn std::error::Error>> {
    println!("   部署 Safe 钱包...");

    let response = client.deploy().await?;
    println!("   交易已提交,ID: {}", response.transaction_id);

    // poll_until_state 会自动轮询直到交易完成
    let result = client
        .poll_until_state(
            &response.transaction_id,
            &[RelayerTransactionState::StateConfirmed],
            Some(RelayerTransactionState::StateFailed),
            30,
            2000,
        )
        .await?;

    match result {
        Some(receipt) => {
            println!("   ✅ 交易已确认!");
            println!("      状态: {}", receipt.state);
            println!("      哈希: {}", receipt.transaction_hash);
            println!("      Safe 地址: {}", receipt.proxy_address);
        }
        None => {
            println!("   ❌ 交易失败或超时");
        }
    }

    Ok(())
}

/// 示例2: 手动轮询状态
async fn manual_polling_example(client: &RelayClient) -> Result<(), Box<dyn std::error::Error>> {
    println!("   部署 Safe 钱包...");

    let response = client.deploy().await?;
    let tx_id = response.transaction_id.clone();

    println!("   交易已提交,ID: {}", tx_id);
    println!("   开始手动轮询状态...\n");

    let mut poll_count = 0;
    loop {
        poll_count += 1;

        // 获取交易状态
        let txns = client.get_transaction(&tx_id).await?;

        if let Some(status) = txns.first() {
            println!(
                "   [{:2}] 状态: {} (更新于: {})",
                poll_count, status.state, status.updated_at
            );

            match status.state.as_str() {
                "STATE_NEW" => {
                    println!("        → Relayer 已接收交易");
                }
                "STATE_EXECUTED" => {
                    println!("        → 交易已在链上执行");
                }
                "STATE_MINED" => {
                    println!("        → 交易已被包含在区块中");
                }
                "STATE_CONFIRMED" => {
                    println!("        → 交易已确认!");
                    println!("\n   ✅ 交易成功!");
                    println!("      交易哈希: {}", status.transaction_hash);
                    break;
                }
                "STATE_FAILED" => {
                    println!("        → 交易失败");
                    println!("\n   ❌ 交易失败!");
                    break;
                }
                "STATE_INVALID" => {
                    println!("        → 交易无效");
                    println!("\n   ❌ 交易被拒绝!");
                    break;
                }
                _ => {
                    println!("        → 未知状态");
                }
            }
        }

        // 等待3秒后重新查询
        sleep(Duration::from_secs(3)).await;
    }

    Ok(())
}

/// 示例3: 带超时的监控
async fn timeout_monitoring_example(
    client: &RelayClient,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("   创建测试交易...");

    // 创建一个简单的交易
    let tx = create_dummy_transaction();

    let response = client
        .execute(vec![tx], Some("Test transaction with timeout".to_string()))
        .await?;

    let tx_id = response.transaction_id.clone();
    println!("   交易已提交,ID: {}", tx_id);

    // 设置超时时间
    let timeout_secs = 60;
    let poll_interval = Duration::from_secs(3);
    let max_polls = timeout_secs / 3;

    println!("   监控交易 (超时: {} 秒)...\n", timeout_secs);

    for i in 0..max_polls {
        let txns = client.get_transaction(&tx_id).await?;

        if let Some(status) = txns.first() {
            println!("   [{}/{}] 状态: {}", i + 1, max_polls, status.state);

            match status.state.as_str() {
                "STATE_CONFIRMED" => {
                    println!("\n   ✅ 交易已确认!");
                    return Ok(());
                }
                "STATE_FAILED" | "STATE_INVALID" => {
                    println!("\n   ❌ 交易失败!");
                    return Err("Transaction failed".into());
                }
                _ => {
                    sleep(poll_interval).await;
                }
            }
        } else {
            sleep(poll_interval).await;
        }
    }

    println!("\n   ⏱️ 监控超时!");
    Err("Transaction monitoring timeout".into())
}

/// 创建一个测试交易
fn create_dummy_transaction() -> SafeTransaction {
    // 创建一个简单的 ETH 转账交易
    SafeTransaction {
        to: "0x0000000000000000000000000000000000000001".to_string(),
        operation: OperationType::Call,
        data: "0x".to_string(),
        value: "0".to_string(),
    }
}

/// 交易状态说明
#[allow(dead_code)]
fn print_state_descriptions() {
    println!("\n📋 交易状态说明:");
    println!("   STATE_NEW       - Relayer 已接收交易");
    println!("   STATE_EXECUTED  - 交易已在链上执行");
    println!("   STATE_MINED     - 交易已被包含在区块中");
    println!("   STATE_CONFIRMED - 交易已确认 (最终状态)");
    println!("   STATE_FAILED    - 交易失败 (终止状态)");
    println!("   STATE_INVALID   - 交易无效 (终止状态)");
}
