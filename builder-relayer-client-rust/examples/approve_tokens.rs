use builder_relayer_client_rust::signer::DummySigner;
/// 代币授权示例
///
/// 演示如何授权 USDC 给 CTF 和 Exchange
use builder_relayer_client_rust::{
    OperationType, RelayClient, RelayerTransactionState, SafeTransaction,
};
use builder_signing_sdk_rs::BuilderApiKeyCreds;

// Polygon 合约地址
const USDC_ADDRESS: &str = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";
const CTF_ADDRESS: &str = "0x4d97dcd97ec945f40cf65f87097ace5ea0476045";
const CTF_EXCHANGE: &str = "0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("💰 代币授权示例\n");

    // 初始化客户端
    let relayer_client = initialize_client().await?;

    // 授权 USDC 给 CTF
    println!("1️⃣ 授权 USDC 给 CTF...");
    approve_token(&relayer_client, USDC_ADDRESS, CTF_ADDRESS).await?;
    println!("✅ 授权完成\n");

    // 授权 USDC 给 Exchange
    println!("2️⃣ 授权 USDC 给 CTF Exchange...");
    approve_token(&relayer_client, USDC_ADDRESS, CTF_EXCHANGE).await?;
    println!("✅ 授权完成\n");

    // 批量授权
    println!("3️⃣ 批量授权...");
    batch_approve(&relayer_client).await?;
    println!("✅ 批量授权完成\n");

    println!("🎉 所有授权完成!");

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

async fn approve_token(
    client: &RelayClient,
    token_address: &str,
    spender_address: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("   代币: {}", token_address);
    println!("   授权给: {}", spender_address);

    let approval_tx = create_approval_transaction(token_address, spender_address);

    let response = client
        .execute(
            vec![approval_tx],
            Some(format!("Approve {} for {}", token_address, spender_address)),
        )
        .await?;

    println!("   交易已提交: {}", response.transaction_id);

    // 等待确认
    let result = client
        .poll_until_state(
            &response.transaction_id,
            &[RelayerTransactionState::StateConfirmed],
            Some(RelayerTransactionState::StateFailed),
            30,
            2000,
        )
        .await?;

    if let Some(receipt) = result {
        println!("   交易哈希: {}", receipt.transaction_hash);
    }

    Ok(())
}

async fn batch_approve(client: &RelayClient) -> Result<(), Box<dyn std::error::Error>> {
    // 创建批量交易
    let transactions = vec![
        create_approval_transaction(USDC_ADDRESS, CTF_ADDRESS),
        create_approval_transaction(USDC_ADDRESS, CTF_EXCHANGE),
    ];

    println!("   提交 {} 个授权交易...", transactions.len());

    let response = client
        .execute(
            transactions,
            Some("Batch approve: USDC for CTF and Exchange".to_string()),
        )
        .await?;

    println!("   交易已提交: {}", response.transaction_id);

    // 等待确认
    let result = client
        .poll_until_state(
            &response.transaction_id,
            &[RelayerTransactionState::StateConfirmed],
            Some(RelayerTransactionState::StateFailed),
            30,
            2000,
        )
        .await?;

    if let Some(receipt) = result {
        println!("   交易哈希: {}", receipt.transaction_hash);
    }

    Ok(())
}

/// 创建 ERC20 approve 交易
///
/// approve(address spender, uint256 amount)
/// 函数选择器: 0x095ea7b3
fn create_approval_transaction(token_address: &str, spender_address: &str) -> SafeTransaction {
    let mut data = vec![0x09, 0x5e, 0xa7, 0xb3]; // approve selector

    // 解析地址
    let spender_bytes = hex::decode(&spender_address[2..]).unwrap();

    // 编码 spender (32字节)
    let mut spender_param = vec![0u8; 12];
    spender_param.extend_from_slice(&spender_bytes);
    data.extend_from_slice(&spender_param);

    // 编码 amount (uint256 max)
    data.extend_from_slice(&[0xffu8; 32]);

    SafeTransaction {
        to: token_address.to_string(),
        operation: OperationType::Call,
        data: format!("0x{}", hex::encode(data)),
        value: "0".to_string(),
    }
}
