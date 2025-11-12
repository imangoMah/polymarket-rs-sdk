//! Test address case sensitivity in EIP-712 signing
//!
//! This test verifies whether address case (checksum vs lowercase) affects EIP-712 signatures

use clob_client_rust::exchange_order_builder::ExchangeOrderBuilder;
use clob_client_rust::signer_adapter::EthersSigner;
use clob_client_rust::signing::Eip712Signer;
use clob_client_rust::types::{OrderData, Side, SignatureType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Address Case Sensitivity Test ===\n");

    let private_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let signer = EthersSigner::new_from_private_key(private_key)?;

    let address = signer.get_address().await?;
    println!("Original address from signer: {}", address);
    println!();

    let chain_id = 137;
    let verifying_contract = "0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E";
    let builder = ExchangeOrderBuilder::new(verifying_contract, chain_id, &signer);
    let fixed_salt = "12345678901234567890";

    // Test 1: 使用全小写地址
    println!("=== Test 1: 全小写地址 ===");
    let lowercase_address = address.to_lowercase();
    println!("Address: {}", lowercase_address);

    let order_data_lowercase = OrderData {
        maker: lowercase_address.clone(),
        taker: "0x0000000000000000000000000000000000000000".to_string(),
        token_id: "123456".to_string(),
        maker_amount: "100000000".to_string(),
        taker_amount: "50000000".to_string(),
        side: Side::BUY,
        fee_rate_bps: "8".to_string(),
        nonce: "0".to_string(),
        signer: lowercase_address.clone(),
        expiration: "2000000000".to_string(),
        signature_type: SignatureType::EOA,
    };

    let mut order1 = builder.build_order(order_data_lowercase)?;
    order1.salt = fixed_salt.to_string();
    let typed_data1 = builder.build_order_typed_data(&order1);
    let signature1 = builder.build_order_signature(&typed_data1).await?;
    println!("Signature: {}", signature1);
    println!();

    // Test 2: 使用 Checksum 地址 (EIP-55)
    println!("=== Test 2: Checksum 地址 (混合大小写) ===");
    let checksum_address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"; // EIP-55 checksum
    println!("Address: {}", checksum_address);

    let order_data_checksum = OrderData {
        maker: checksum_address.to_string(),
        taker: "0x0000000000000000000000000000000000000000".to_string(),
        token_id: "123456".to_string(),
        maker_amount: "100000000".to_string(),
        taker_amount: "50000000".to_string(),
        side: Side::BUY,
        fee_rate_bps: "8".to_string(),
        nonce: "0".to_string(),
        signer: checksum_address.to_string(),
        expiration: "2000000000".to_string(),
        signature_type: SignatureType::EOA,
    };

    let mut order2 = builder.build_order(order_data_checksum)?;
    order2.salt = fixed_salt.to_string();
    let typed_data2 = builder.build_order_typed_data(&order2);
    let signature2 = builder.build_order_signature(&typed_data2).await?;
    println!("Signature: {}", signature2);
    println!();

    // Test 3: 使用混合大小写但保持 0x 前缀小写
    println!("=== Test 3: 混合大小写 (0x 小写) ===");
    let mixed_case_address = "0xF39FD6E51AAD88F6F4CE6AB8827279CFFFB92266";
    println!("Address: {}", mixed_case_address);

    let order_data_mixed = OrderData {
        maker: mixed_case_address.to_string(),
        taker: "0x0000000000000000000000000000000000000000".to_string(),
        token_id: "123456".to_string(),
        maker_amount: "100000000".to_string(),
        taker_amount: "50000000".to_string(),
        side: Side::BUY,
        fee_rate_bps: "8".to_string(),
        nonce: "0".to_string(),
        signer: mixed_case_address.to_string(),
        expiration: "2000000000".to_string(),
        signature_type: SignatureType::EOA,
    };

    let mut order3 = builder.build_order(order_data_mixed)?;
    order3.salt = fixed_salt.to_string();
    let typed_data3 = builder.build_order_typed_data(&order3);
    let signature3 = builder.build_order_signature(&typed_data3).await?;
    println!("Signature: {}", signature3);
    println!();

    // 对比结果
    println!("=== 签名对比 ===");
    println!("签名1 (全小写):      {}", signature1);
    println!("签名2 (Checksum):    {}", signature2);
    println!("签名3 (混合大小写):  {}", signature3);
    println!();

    if signature1 == signature2 && signature2 == signature3 {
        println!("✅ 结论: 地址大小写不影响签名!");
        println!("   EIP-712 实现会自动规范化地址格式");
    } else {
        println!("⚠️ 警告: 地址大小写影响签名!");
        if signature1 == signature2 {
            println!("   ✅ 小写 == Checksum");
        } else {
            println!("   ❌ 小写 != Checksum");
        }
        if signature2 == signature3 {
            println!("   ✅ Checksum == 混合大小写");
        } else {
            println!("   ❌ Checksum != 混合大小写");
        }
        if signature1 == signature3 {
            println!("   ✅ 小写 == 混合大小写");
        } else {
            println!("   ❌ 小写 != 混合大小写");
        }
    }

    Ok(())
}
