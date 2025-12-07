use builder_relayer_client_rust::builder::safe::AbstractSigner;
use builder_relayer_client_rust::signer::DummySigner;
/// CTF (Conditional Token Framework) 操作示例
///
/// 演示如何执行条件代币框架操作:
/// - Split Positions (分割头寸)
/// - Merge Positions (合并头寸)
/// - Redeem Positions (赎回头寸)
use builder_relayer_client_rust::{
    OperationType, RelayClient, RelayerTransactionState, SafeTransaction,
};
use builder_signing_sdk_rs::BuilderApiKeyCreds;

// Polygon 主网合约地址
const POLYGON_CHAIN_ID: u64 = 137;
const USDC_ADDRESS: &str = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";
const CTF_ADDRESS: &str = "0x4d97dcd97ec945f40cf65f87097ace5ea0476045";
const RELAYER_URL: &str = "https://relayer-v2.polymarket.com/";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎲 CTF 操作示例\n");

    let client = initialize_client().await?;
    println!("✅ 客户端初始化成功\n");

    // 示例参数
    let parent_collection_id = "0x0000000000000000000000000000000000000000000000000000000000000000";
    let condition_id = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
    let partition = vec![1, 2]; // [YES, NO]
    let amount = "1000000"; // 1 USDC (6 decimals)
    let index_sets = vec![1, 2];

    // 示例1: Split Position (分割头寸)
    println!("1️⃣ 分割头寸 (Split Position)...");
    println!("   将抵押品代币分割为条件代币");
    split_position(
        &client,
        USDC_ADDRESS,
        parent_collection_id,
        condition_id,
        &partition,
        amount,
    )
    .await?;
    println!("✅ 分割完成\n");

    // 示例2: Merge Position (合并头寸)
    println!("2️⃣ 合并头寸 (Merge Position)...");
    println!("   将条件代币合并回抵押品");
    merge_position(
        &client,
        USDC_ADDRESS,
        parent_collection_id,
        condition_id,
        &partition,
        amount,
    )
    .await?;
    println!("✅ 合并完成\n");

    // 示例3: Redeem Position (赎回头寸)
    println!("3️⃣ 赎回头寸 (Redeem Position)...");
    println!("   赎回获胜的条件代币换回抵押品");
    redeem_position(
        &client,
        USDC_ADDRESS,
        parent_collection_id,
        condition_id,
        &index_sets,
    )
    .await?;
    println!("✅ 赎回完成\n");

    println!("🎉 所有 CTF 操作完成!");

    Ok(())
}

/// 初始化 Relayer Client
async fn initialize_client() -> Result<RelayClient, Box<dyn std::error::Error>> {
    let private_key = std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY 环境变量未设置");
    let signer = DummySigner::new(&private_key)?;
    let signer_address = signer.address().to_string();

    println!("钱包地址: {}", signer_address);

    let relay_client = RelayClient::new(RELAYER_URL, POLYGON_CHAIN_ID)
        .with_signer(Box::new(signer.clone()), Box::new(signer))
        .with_builder_api_key(BuilderApiKeyCreds {
            key: std::env::var("BUILDER_API_KEY")?,
            secret: std::env::var("BUILDER_SECRET")?,
            passphrase: std::env::var("BUILDER_PASS_PHRASE")?,
        });

    Ok(relay_client)
}

/// Split Position (分割头寸)
///
/// 将抵押品代币(如 USDC)分割为代表不同结果的条件代币
///
/// 函数签名: splitPosition(address collateralToken, bytes32 parentCollectionId, bytes32 conditionId, uint[] partition, uint amount)
/// 函数选择器: keccak256("splitPosition(address,bytes32,bytes32,uint256[],uint256)") = 0x5c382289
async fn split_position(
    client: &RelayClient,
    collateral_token: &str,
    parent_collection_id: &str,
    condition_id: &str,
    partition: &[u32],
    amount: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("   抵押品: {}", collateral_token);
    println!("   条件ID: {}", condition_id);
    println!("   分区: {:?}", partition);
    println!("   数量: {}", amount);

    let split_tx = create_split_position_transaction(
        collateral_token,
        parent_collection_id,
        condition_id,
        partition,
        amount,
    )?;

    let response = client
        .execute(
            vec![split_tx],
            Some("Split position into conditional tokens".to_string()),
        )
        .await?;

    println!("   交易已提交: {}", response.transaction_id);

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
        None => Err("Split position 失败或超时".into()),
    }
}

/// Merge Position (合并头寸)
///
/// 将条件代币合并回抵押品代币
///
/// 函数签名: mergePositions(address collateralToken, bytes32 parentCollectionId, bytes32 conditionId, uint[] partition, uint amount)
/// 函数选择器: keccak256("mergePositions(address,bytes32,bytes32,uint256[],uint256)") = 0xb73f4554
async fn merge_position(
    client: &RelayClient,
    collateral_token: &str,
    parent_collection_id: &str,
    condition_id: &str,
    partition: &[u32],
    amount: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("   抵押品: {}", collateral_token);
    println!("   条件ID: {}", condition_id);
    println!("   分区: {:?}", partition);
    println!("   数量: {}", amount);

    let merge_tx = create_merge_position_transaction(
        collateral_token,
        parent_collection_id,
        condition_id,
        partition,
        amount,
    )?;

    let response = client
        .execute(
            vec![merge_tx],
            Some("Merge conditional tokens back to collateral".to_string()),
        )
        .await?;

    println!("   交易已提交: {}", response.transaction_id);

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
        None => Err("Merge position 失败或超时".into()),
    }
}

/// Redeem Position (赎回头寸)
///
/// 在市场解决后赎回获胜的条件代币换回抵押品
///
/// 函数签名: redeemPositions(address collateralToken, bytes32 parentCollectionId, bytes32 conditionId, uint[] indexSets)
/// 函数选择器: keccak256("redeemPositions(address,bytes32,bytes32,uint256[])") = 0x6d625a4e
async fn redeem_position(
    client: &RelayClient,
    collateral_token: &str,
    parent_collection_id: &str,
    condition_id: &str,
    index_sets: &[u32],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("   抵押品: {}", collateral_token);
    println!("   条件ID: {}", condition_id);
    println!("   索引集: {:?}", index_sets);

    let redeem_tx = create_redeem_position_transaction(
        collateral_token,
        parent_collection_id,
        condition_id,
        index_sets,
    )?;

    let response = client
        .execute(
            vec![redeem_tx],
            Some("Redeem winning conditional tokens".to_string()),
        )
        .await?;

    println!("   交易已提交: {}", response.transaction_id);

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
        None => Err("Redeem position 失败或超时".into()),
    }
}

/// 创建 splitPosition 交易
///
/// splitPosition(address collateralToken, bytes32 parentCollectionId, bytes32 conditionId, uint256[] partition, uint256 amount)
fn create_split_position_transaction(
    collateral_token: &str,
    parent_collection_id: &str,
    condition_id: &str,
    partition: &[u32],
    amount: &str,
) -> Result<SafeTransaction, Box<dyn std::error::Error>> {
    // 函数选择器: splitPosition(address,bytes32,bytes32,uint256[],uint256)
    let function_selector = [0x5c, 0x38, 0x22, 0x89];

    let mut data = function_selector.to_vec();

    // 参数1: collateralToken (address) - 32字节对齐
    let token_bytes = hex::decode(&collateral_token[2..])?;
    let mut token_param = vec![0u8; 12];
    token_param.extend_from_slice(&token_bytes);
    data.extend_from_slice(&token_param);

    // 参数2: parentCollectionId (bytes32)
    let parent_bytes = hex::decode(&parent_collection_id[2..])?;
    data.extend_from_slice(&parent_bytes);

    // 参数3: conditionId (bytes32)
    let condition_bytes = hex::decode(&condition_id[2..])?;
    data.extend_from_slice(&condition_bytes);

    // 参数4: partition (uint256[]) - 动态数组
    // 偏移量指向动态数据的位置 (从函数参数开始算,这里是 0xa0 = 160)
    data.extend_from_slice(&[0u8; 31]);
    data.push(0xa0);

    // 参数5: amount (uint256)
    let amount_value: u128 = amount.parse()?;
    let mut amount_bytes = [0u8; 32];
    amount_bytes[16..].copy_from_slice(&amount_value.to_be_bytes());
    data.extend_from_slice(&amount_bytes);

    // 动态数组数据: partition
    // 数组长度
    let mut length_bytes = [0u8; 32];
    length_bytes[31] = partition.len() as u8;
    data.extend_from_slice(&length_bytes);

    // 数组元素
    for &element in partition {
        let mut element_bytes = [0u8; 32];
        element_bytes[28..].copy_from_slice(&element.to_be_bytes());
        data.extend_from_slice(&element_bytes);
    }

    Ok(SafeTransaction {
        to: CTF_ADDRESS.to_string(),
        operation: OperationType::Call,
        data: format!("0x{}", hex::encode(data)),
        value: "0".to_string(),
    })
}

/// 创建 mergePositions 交易
///
/// mergePositions(address collateralToken, bytes32 parentCollectionId, bytes32 conditionId, uint256[] partition, uint256 amount)
fn create_merge_position_transaction(
    collateral_token: &str,
    parent_collection_id: &str,
    condition_id: &str,
    partition: &[u32],
    amount: &str,
) -> Result<SafeTransaction, Box<dyn std::error::Error>> {
    // 函数选择器: mergePositions(address,bytes32,bytes32,uint256[],uint256)
    let function_selector = [0xb7, 0x3f, 0x45, 0x54];

    let mut data = function_selector.to_vec();

    // 参数1: collateralToken (address) - 32字节对齐
    let token_bytes = hex::decode(&collateral_token[2..])?;
    let mut token_param = vec![0u8; 12];
    token_param.extend_from_slice(&token_bytes);
    data.extend_from_slice(&token_param);

    // 参数2: parentCollectionId (bytes32)
    let parent_bytes = hex::decode(&parent_collection_id[2..])?;
    data.extend_from_slice(&parent_bytes);

    // 参数3: conditionId (bytes32)
    let condition_bytes = hex::decode(&condition_id[2..])?;
    data.extend_from_slice(&condition_bytes);

    // 参数4: partition (uint256[]) - 动态数组
    // 偏移量
    data.extend_from_slice(&[0u8; 31]);
    data.push(0xa0);

    // 参数5: amount (uint256)
    let amount_value: u128 = amount.parse()?;
    let mut amount_bytes = [0u8; 32];
    amount_bytes[16..].copy_from_slice(&amount_value.to_be_bytes());
    data.extend_from_slice(&amount_bytes);

    // 动态数组数据: partition
    // 数组长度
    let mut length_bytes = [0u8; 32];
    length_bytes[31] = partition.len() as u8;
    data.extend_from_slice(&length_bytes);

    // 数组元素
    for &element in partition {
        let mut element_bytes = [0u8; 32];
        element_bytes[28..].copy_from_slice(&element.to_be_bytes());
        data.extend_from_slice(&element_bytes);
    }

    Ok(SafeTransaction {
        to: CTF_ADDRESS.to_string(),
        operation: OperationType::Call,
        data: format!("0x{}", hex::encode(data)),
        value: "0".to_string(),
    })
}

/// 创建 redeemPositions 交易
///
/// redeemPositions(address collateralToken, bytes32 parentCollectionId, bytes32 conditionId, uint256[] indexSets)
fn create_redeem_position_transaction(
    collateral_token: &str,
    parent_collection_id: &str,
    condition_id: &str,
    index_sets: &[u32],
) -> Result<SafeTransaction, Box<dyn std::error::Error>> {
    // 函数选择器: redeemPositions(address,bytes32,bytes32,uint256[])
    let function_selector = [0x6d, 0x62, 0x5a, 0x4e];

    let mut data = function_selector.to_vec();

    // 参数1: collateralToken (address) - 32字节对齐
    let token_bytes = hex::decode(&collateral_token[2..])?;
    let mut token_param = vec![0u8; 12];
    token_param.extend_from_slice(&token_bytes);
    data.extend_from_slice(&token_param);

    // 参数2: parentCollectionId (bytes32)
    let parent_bytes = hex::decode(&parent_collection_id[2..])?;
    data.extend_from_slice(&parent_bytes);

    // 参数3: conditionId (bytes32)
    let condition_bytes = hex::decode(&condition_id[2..])?;
    data.extend_from_slice(&condition_bytes);

    // 参数4: indexSets (uint256[]) - 动态数组
    // 偏移量 (从函数参数开始算,这里是 0x80 = 128)
    data.extend_from_slice(&[0u8; 31]);
    data.push(0x80);

    // 动态数组数据: indexSets
    // 数组长度
    let mut length_bytes = [0u8; 32];
    length_bytes[31] = index_sets.len() as u8;
    data.extend_from_slice(&length_bytes);

    // 数组元素
    for &element in index_sets {
        let mut element_bytes = [0u8; 32];
        element_bytes[28..].copy_from_slice(&element.to_be_bytes());
        data.extend_from_slice(&element_bytes);
    }

    Ok(SafeTransaction {
        to: CTF_ADDRESS.to_string(),
        operation: OperationType::Call,
        data: format!("0x{}", hex::encode(data)),
        value: "0".to_string(),
    })
}
