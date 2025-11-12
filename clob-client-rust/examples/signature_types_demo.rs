use clob_client_rust::client::ClobClient;
use clob_client_rust::order_builder::BuilderConfig;
use clob_client_rust::signer_adapter::EthersSigner;
use clob_client_rust::types::{ApiKeyCreds, Side, SignatureType, UserOrder};
use std::sync::Arc;

/// Example demonstrating the use of different SignatureType variants
///
/// SignatureType determines how orders are signed:
/// - EOA (0): Standard externally owned account (MetaMask, etc.) - DEFAULT
/// - PolyProxy (1): Polymarket proxy wallet system
/// - PolyGnosisSafe (2): Gnosis Safe multisig wallet
///
/// Run with:
/// cargo run --example signature_types_demo
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SignatureType Demo ===\n");

    // Setup
    let pk = std::env::var("PK").unwrap_or_else(|_| {
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string()
    });
    let signer = Arc::new(EthersSigner::new_from_private_key(&pk)?);
    let host = "http://localhost:8080";
    let chain_id: i64 = 80002; // Amoy testnet

    let creds = ApiKeyCreds {
        key: "test_key".to_string(),
        secret: "test_secret".to_string(),
        passphrase: "test_pass".to_string(),
    };

    // Example 1: Using EOA (default - most common)
    println!("1. Creating client with EOA signature type (default):");
    let _client_eoa = ClobClient::new(
        host,
        chain_id,
        Some(signer.clone()),
        Some(creds.clone()),
        false,
    )
    .with_builder_config(BuilderConfig {
        tick_size: Some("0.01".to_string()),
        neg_risk: None,
        signature_type: SignatureType::EOA,
        funder_address: None,
    });

    println!("   ✓ SignatureType::EOA (value=0)");
    println!("   Use case: Standard wallets (MetaMask, WalletConnect, etc.)\n");

    // Example 2: Using PolyProxy
    println!("2. Creating client with PolyProxy signature type:");
    let _client_proxy = ClobClient::new(
        host,
        chain_id,
        Some(signer.clone()),
        Some(creds.clone()),
        false,
    )
    .with_builder_config(BuilderConfig {
        tick_size: Some("0.01".to_string()),
        neg_risk: None,
        signature_type: SignatureType::PolyProxy,
        funder_address: None,
    });

    println!("   ✓ SignatureType::PolyProxy (value=1)");
    println!("   Use case: Polymarket's proxy wallet system\n");

    // Example 3: Using PolyGnosisSafe
    println!("3. Creating client with PolyGnosisSafe signature type:");
    let _client_safe = ClobClient::new(
        host,
        chain_id,
        Some(signer.clone()),
        Some(creds.clone()),
        false,
    )
    .with_builder_config(BuilderConfig {
        tick_size: Some("0.01".to_string()),
        neg_risk: None,
        signature_type: SignatureType::PolyGnosisSafe,
        funder_address: None,
    });

    println!("   ✓ SignatureType::PolyGnosisSafe (value=2)");
    println!("   Use case: Gnosis Safe multisig wallets\n");

    // Demonstrate serialization
    println!("4. Serialization examples:");
    println!(
        "   EOA serializes to: {}",
        serde_json::to_string(&SignatureType::EOA)?
    );
    println!(
        "   PolyProxy serializes to: {}",
        serde_json::to_string(&SignatureType::PolyProxy)?
    );
    println!(
        "   PolyGnosisSafe serializes to: {}",
        serde_json::to_string(&SignatureType::PolyGnosisSafe)?
    );
    println!();

    // Demonstrate deserialization
    println!("5. Deserialization examples:");
    let eoa_from_num: SignatureType = serde_json::from_str("0")?;
    let eoa_from_str: SignatureType = serde_json::from_str(r#""EOA""#)?;
    println!("   From number 0: {:?}", eoa_from_num);
    println!("   From string \"EOA\": {:?}", eoa_from_str);
    println!();

    // Create sample orders with different signature types
    let _user_order = UserOrder {
        token_id: "1234".to_string(),
        price: 0.5,
        size: 10.0,
        side: Side::BUY,
        fee_rate_bps: 0.0,
        nonce: None,
        expiration: None,
        taker: None,
    };

    println!("6. Creating orders with different signature types:");

    // Note: In a real scenario, you would actually create and sign orders
    // For demo purposes, we're just showing the client setup
    println!("   ✓ Client with EOA ready");
    println!("   ✓ Client with PolyProxy ready");
    println!("   ✓ Client with PolyGnosisSafe ready");
    println!();

    println!("7. Default behavior:");
    let _default_config = BuilderConfig::default();
    println!("   BuilderConfig::default() uses SignatureType::EOA");
    println!("   This matches TypeScript behavior: signatureType ?? SignatureType.EOA");
    println!();

    println!("=== Demo Complete ===");
    println!("\nKey Points:");
    println!("• SignatureType::EOA is the default and most commonly used");
    println!("• All three types serialize to numbers (0, 1, 2) for EIP-712");
    println!("• Deserialization supports both numbers and string names");
    println!("• The choice depends on your wallet type and requirements");

    Ok(())
}
