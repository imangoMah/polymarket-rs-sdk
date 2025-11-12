use builder_relayer_client_rust::builder::safe::AbstractSigner;
use builder_relayer_client_rust::signer::DummySigner;
use builder_relayer_client_rust::{RelayClient, RelayerTransactionState};
use builder_signing_sdk_rs::BuilderApiKeyCreds;

/// 快速开始示例
///
/// 最简单的 Relayer Client 使用方式

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 快速开始: Polymarket Relayer Client\n");

    // 1. 配置
    let relayer_url = "https://relayer-v2.polymarket.com/";
    let chain_id = 137u64; // Polygon 主网

    // 2. 创建签名器
    let private_key = std::env::var("PRIVATE_KEY").expect("需要设置 PRIVATE_KEY 环境变量");
    let signer = DummySigner::new(&private_key)?;
    let signer_address = signer.get_address()?;

    println!("钱包地址: {}", signer_address);

    // 3. 创建 Relayer Client
    let relayer_client = RelayClient::new(relayer_url, chain_id)
        .with_signer(Box::new(signer.clone()), Box::new(signer))
        .with_builder_api_key(BuilderApiKeyCreds {
            key: std::env::var("BUILDER_API_KEY")?,
            secret: std::env::var("BUILDER_SECRET")?,
            passphrase: std::env::var("BUILDER_PASS_PHRASE")?,
        });

    println!("✅ Relayer Client 初始化成功!\n");

    // 4. 部署 Safe 钱包
    println!("部署 Safe 钱包...");
    let response = relayer_client.deploy().await?;

    println!("   交易已提交: {}", response.transaction_id);
    println!("   等待确认...");

    // 5. 等待交易确认
    let result = relayer_client
        .poll_until_state(
            &response.transaction_id,
            &[RelayerTransactionState::StateConfirmed],
            Some(RelayerTransactionState::StateFailed),
            30,   // 最多轮询 30 次
            2000, // 每 2 秒轮询一次
        )
        .await?;

    if let Some(receipt) = result {
        println!("✅ Safe 部署成功!");
        println!("   交易状态: {}", receipt.state);
        println!("   交易哈希: {}", receipt.transaction_hash);
        println!("   Safe 地址: {}", receipt.proxy_address);
    } else {
        println!("❌ Safe 部署失败或超时");
    }

    Ok(())
}
