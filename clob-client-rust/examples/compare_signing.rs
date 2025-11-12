//! Compare signing process between Rust and TypeScript SDKs
//!
//! This example generates detailed logs of the signing process to compare
//! with TypeScript SDK output.
//!
//! Usage: cargo run --example compare_signing

use clob_client_rust::exchange_order_builder::ExchangeOrderBuilder;
use clob_client_rust::signer_adapter::EthersSigner;
use clob_client_rust::signing::Eip712Signer;
use clob_client_rust::types::{OrderData, Side, SignatureType};
use ethers::core::utils::keccak256;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Rust SDK Signing Process Comparison ===\n");

    // 使用固定的测试数据以便与 TypeScript 对比
    let private_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"; // Hardhat 测试账户 #0
    let signer = EthersSigner::new_from_private_key(private_key)?;

    let address = signer.get_address().await?;
    println!("Signer Address: {}", address);
    println!();

    // 创建测试订单数据（与 TypeScript 使用相同的参数）
    let order_data = OrderData {
        maker: address.clone(),
        taker: "0x0000000000000000000000000000000000000000".to_string(),
        token_id: "123456".to_string(),
        maker_amount: "100000000".to_string(), // 100 with 6 decimals
        taker_amount: "50000000".to_string(),  // 50 with 6 decimals
        side: Side::BUY,
        fee_rate_bps: "8".to_string(),
        nonce: "0".to_string(),
        signer: address.clone(),
        expiration: "2000000000".to_string(), // 固定过期时间
        signature_type: SignatureType::EOA,
    };

    println!("Order Data:");
    println!("  maker: {}", order_data.maker);
    println!("  taker: {}", order_data.taker);
    println!("  tokenId: {}", order_data.token_id);
    println!("  makerAmount: {}", order_data.maker_amount);
    println!("  takerAmount: {}", order_data.taker_amount);
    println!("  side: {:?}", order_data.side);
    println!("  feeRateBps: {}", order_data.fee_rate_bps);
    println!("  nonce: {}", order_data.nonce);
    println!("  signer: {}", order_data.signer);
    println!("  expiration: {}", order_data.expiration);
    println!("  signatureType: {:?}", order_data.signature_type);
    println!();

    // 创建 ExchangeOrderBuilder
    let chain_id = 137; // Polygon mainnet
    let verifying_contract = "0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E"; // NEG_RISK_CTF_EXCHANGE

    let builder = ExchangeOrderBuilder::new(verifying_contract, chain_id, &signer);

    println!("EIP-712 Domain:");
    println!("  name: Polymarket CTF Exchange");
    println!("  version: 1");
    println!("  chainId: {}", chain_id);
    println!("  verifyingContract: {}", verifying_contract);
    println!();

    // 使用固定的 salt 以便与 TypeScript 对比
    let fixed_salt = "12345678901234567890";

    // 构建订单（使用固定 salt）
    let mut order = builder.build_order(order_data)?;
    order.salt = fixed_salt.to_string();

    println!("Built Order (with fixed salt):");
    println!("  salt: {}", order.salt);
    println!("  maker: {}", order.maker);
    println!("  signer: {}", order.signer);
    println!("  taker: {}", order.taker);
    println!("  tokenId: {}", order.token_id);
    println!("  makerAmount: {}", order.maker_amount);
    println!("  takerAmount: {}", order.taker_amount);
    println!("  side: {:?}", order.side);
    println!("  feeRateBps: {}", order.fee_rate_bps);
    println!("  nonce: {}", order.nonce);
    println!("  expiration: {}", order.expiration);
    println!("  signatureType: {:?}", order.signature_type);
    println!();

    // 构建 typed data
    let typed_data = builder.build_order_typed_data(&order);

    println!("=== Typed Data Structure ===");
    println!("{}", serde_json::to_string_pretty(&typed_data)?);
    println!();

    // 提取各部分
    let domain = typed_data.get("domain").unwrap();
    let types = typed_data.get("types").unwrap();
    let message = typed_data.get("message").unwrap();

    println!("=== Domain JSON ===");
    println!("{}", serde_json::to_string_pretty(domain)?);
    println!();

    println!("=== Types JSON ===");
    println!("{}", serde_json::to_string_pretty(types)?);
    println!();

    println!("=== Message JSON ===");
    println!("{}", serde_json::to_string_pretty(message)?);
    println!();

    // 计算各部分的 hash
    println!("=== Hash Calculations ===");

    // Domain separator hash
    let domain_str = serde_json::to_string(domain)?;
    let domain_bytes = domain_str.as_bytes();
    let domain_hash = keccak256(domain_bytes);
    println!("Domain String: {}", domain_str);
    println!("Domain Hash: 0x{}", hex::encode(domain_hash));
    println!();

    // Types hash
    let types_str = serde_json::to_string(types)?;
    let types_bytes = types_str.as_bytes();
    let types_hash = keccak256(types_bytes);
    println!("Types String: {}", types_str);
    println!("Types Hash: 0x{}", hex::encode(types_hash));
    println!();

    // Message hash
    let message_str = serde_json::to_string(message)?;
    let message_bytes = message_str.as_bytes();
    let message_hash = keccak256(message_bytes);
    println!("Message String: {}", message_str);
    println!("Message Hash: 0x{}", hex::encode(message_hash));
    println!();

    // 生成签名
    println!("=== Signature Generation ===");
    let signature = builder.build_order_signature(&typed_data).await?;
    println!("Signature: {}", signature);
    println!("Signature length: {}", signature.len());

    // 解析签名部分
    if signature.starts_with("0x") && signature.len() == 132 {
        let sig_bytes = &signature[2..];
        let r = &sig_bytes[0..64];
        let s = &sig_bytes[64..128];
        let v = &sig_bytes[128..130];

        println!();
        println!("Signature Components:");
        println!("  r: 0x{}", r);
        println!("  s: 0x{}", s);
        println!("  v: 0x{}", v);
    }
    println!();

    // 计算订单 hash
    let order_hash = builder.build_order_hash(&typed_data);
    println!("Order Hash: {}", order_hash);
    println!();

    println!("=== Summary ===");
    println!("✓ Generated typed data structure");
    println!("✓ Calculated domain separator");
    println!("✓ Calculated struct hash");
    println!("✓ Generated signature");
    println!("✓ Calculated order hash");
    println!();
    println!("Please compare these values with TypeScript SDK output!");
    println!();
    println!("To run TypeScript comparison:");
    println!("  cd ../clob-order-utils");
    println!("  npx ts-node compare-signing.ts");

    Ok(())
}
