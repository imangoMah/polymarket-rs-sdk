use builder_relayer_client_rust::builder::safe::AbstractSigner;
use builder_relayer_client_rust::signer::DummySigner;
/// Relayer Client 示例
///
/// 演示如何使用 Polymarket Relayer Client 执行各种操作
use builder_relayer_client_rust::{
    OperationType, RelayClient, RelayerTransactionState, SafeTransaction,
};
use builder_signing_sdk_rs::BuilderApiKeyCreds;

// Polygon 主网合约地址
const POLYGON_CHAIN_ID: u64 = 137;
const USDC_ADDRESS: &str = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";
const CTF_ADDRESS: &str = "0x4d97dcd97ec945f40cf65f87097ace5ea0476045";
const CTF_EXCHANGE: &str = "0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E";
const RELAYER_URL: &str = "https://relayer-v2.polymarket.com/";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Polymarket Relayer Client 示例\n");

    // 示例1: 初始化客户端
    println!("1️⃣ 初始化 Relayer Client...");
    let client = initialize_client().await?;
    println!("✅ 客户端初始化成功\n");

    // 示例2: 部署 Safe 钱包
    println!("2️⃣ 部署 Safe 钱包...");
    let safe_address = deploy_safe(&client).await?;
    println!("✅ Safe 部署成功: {}\n", safe_address);

    // 示例3: 授权代币
    println!("3️⃣ 授权 USDC 代币...");
    approve_token(&client, USDC_ADDRESS, CTF_ADDRESS).await?;
    println!("✅ 代币授权成功\n");

    // 示例4: 批量交易
    println!("4️⃣ 执行批量交易...");
    execute_batch_transactions(&client).await?;
    println!("✅ 批量交易完成\n");

    println!("🎉 所有示例执行完成!");

    Ok(())
}

/// 初始化 Relayer Client
async fn initialize_client() -> Result<RelayClient, Box<dyn std::error::Error>> {
    // 获取配置
    let private_key = std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY 环境变量未设置");
    let signer = DummySigner::new(&private_key)?;
    let signer_address = signer.get_address()?;

    println!("   钱包地址: {}", signer_address);

    // 获取 Builder 凭证
    let api_key = std::env::var("BUILDER_API_KEY").expect("BUILDER_API_KEY 环境变量未设置");
    let secret = std::env::var("BUILDER_SECRET").expect("BUILDER_SECRET 环境变量未设置");
    let passphrase =
        std::env::var("BUILDER_PASS_PHRASE").expect("BUILDER_PASS_PHRASE 环境变量未设置");

    // 创建 RelayClient
    let relay_client = RelayClient::new(RELAYER_URL, POLYGON_CHAIN_ID)
        .with_signer(Box::new(signer.clone()), Box::new(signer))
        .with_builder_api_key(BuilderApiKeyCreds {
            key: api_key,
            secret,
            passphrase,
        });

    Ok(relay_client)
}

/// 部署 Safe 钱包
async fn deploy_safe(client: &RelayClient) -> Result<String, Box<dyn std::error::Error>> {
    println!("   发送部署请求...");

    let response = client.deploy().await?;
    println!("   交易已提交,等待确认...");

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
            println!("   交易哈希: {}", receipt.transaction_hash);
            println!("   Safe 地址: {}", receipt.proxy_address);
            Ok(receipt.proxy_address)
        }
        None => Err("Safe 部署失败或超时".into()),
    }
}

/// 授权代币
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

    println!("   交易已提交,等待确认...");
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
            println!("   交易哈希: {}", receipt.transaction_hash);
            Ok(())
        }
        None => Err("代币授权失败或超时".into()),
    }
}

/// 创建 ERC20 approve 交易
fn create_approval_transaction(token_address: &str, spender_address: &str) -> SafeTransaction {
    // approve(address spender, uint256 amount) 函数选择器
    // keccak256("approve(address,uint256)") 的前4字节: 0x095ea7b3
    let mut data = vec![0x09, 0x5e, 0xa7, 0xb3];

    // 解析地址
    let spender_bytes = hex::decode(&spender_address[2..]).unwrap();

    // 编码 spender 参数 (address,填充到32字节)
    let mut spender_param = vec![0u8; 12];
    spender_param.extend_from_slice(&spender_bytes);
    data.extend_from_slice(&spender_param);

    // 编码 amount 参数 (uint256 max)
    data.extend_from_slice(&[0xffu8; 32]);

    SafeTransaction {
        to: token_address.to_string(),
        operation: OperationType::Call,
        data: format!("0x{}", hex::encode(data)),
        value: "0".to_string(),
    }
}

/// 执行批量交易
async fn execute_batch_transactions(
    client: &RelayClient,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("   创建批量交易...");

    // 创建多个授权交易
    let transactions = vec![
        create_approval_transaction(USDC_ADDRESS, CTF_ADDRESS),
        create_approval_transaction(USDC_ADDRESS, CTF_EXCHANGE),
    ];

    println!("   批量交易包含 {} 个操作", transactions.len());

    let response = client
        .execute(
            transactions,
            Some("Batch approvals: USDC for CTF and Exchange".to_string()),
        )
        .await?;

    println!("   交易已提交,等待确认...");
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
            println!("   交易哈希: {}", receipt.transaction_hash);
            println!("   状态: {}", receipt.state);
            Ok(())
        }
        None => Err("批量交易失败或超时".into()),
    }
}

// 注意: CTF 操作示例需要完整的 ABI 编码实现
// 以下是占位符,实际使用时需要使用 ethers-rs 的 ABI 编码功能

/// 示例: 创建转账交易
#[allow(dead_code)]
fn create_transfer_tx(to: &str, value: &str) -> SafeTransaction {
    SafeTransaction {
        to: to.to_string(),
        operation: OperationType::Call,
        data: "0x".to_string(),
        value: value.to_string(),
    }
}
