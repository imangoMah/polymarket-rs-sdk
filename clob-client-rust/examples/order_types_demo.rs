//! Demonstration of orderType parameter support in Rust SDK.
//!
//! This example shows how to pass different orderType values when creating orders,
//! matching the TypeScript SDK's functionality.
//!
//! Usage:
//!   cargo run --example order_types_demo

use clob_client_rust::types::{OrderType, Side, SignatureType, SignedOrder};

fn create_mock_signed_order() -> SignedOrder {
    SignedOrder {
        salt: "12345678901234567890".to_string(),
        maker: "0x1234567890123456789012345678901234567890".to_string(),
        signer: "0x1234567890123456789012345678901234567890".to_string(),
        taker: "0x0000000000000000000000000000000000000000".to_string(),
        token_id: "123456".to_string(),
        maker_amount: "100".to_string(),
        taker_amount: "50".to_string(),
        side: Side::BUY,
        fee_rate_bps: "8".to_string(),
        nonce: "0".to_string(),
        signature_type: SignatureType::EOA,
        signature: "0xabcdef...".to_string(),
        expiration: "1234567890".to_string(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== OrderType Parameter Support Demo ===\n");

    // Create a mock signed order
    let order = create_mock_signed_order();

    // Demonstrate orderToJson transformation
    println!("1. Demonstrating orderToJson transformation:");
    println!("   Input: SignedOrder (raw order structure)");
    println!("   Output: NewOrder (wrapped with orderType, owner, deferExec)");
    println!();

    // Show different OrderType variants
    println!("2. Available OrderType variants:");
    println!("   - GTC (Good Till Cancelled) - Default for limit orders");
    println!("   - GTD (Good Till Date)");
    println!("   - FOK (Fill Or Kill) - Default for market orders");
    println!("   - FAK (Fill And Kill)");
    println!();

    // Demonstrate NewOrder structure for each type
    println!("3. NewOrder structure examples:\n");

    for order_type in [
        OrderType::GTC,
        OrderType::GTD,
        OrderType::FOK,
        OrderType::FAK,
    ] {
        let new_order = clob_client_rust::utilities::order_to_json(
            &order,
            "0x1234567890123456789012345678901234567890",
            order_type.clone(),
            false,
        );

        let order_type_str = match order_type {
            OrderType::GTC => "GTC",
            OrderType::GTD => "GTD",
            OrderType::FOK => "FOK",
            OrderType::FAK => "FAK",
        };

        println!("   OrderType::{}", order_type_str);
        println!("   JSON payload:");
        println!("{}", serde_json::to_string_pretty(&new_order)?);
        println!();
    }

    // Explain API method signatures
    println!("4. API Method Signatures:\n");
    println!("   create_and_post_order(user_order, tick, order_type: Option<OrderType>)");
    println!("   - orderType defaults to GTC if None");
    println!(
        "   - Example: client.create_and_post_order(order, None, Some(OrderType::GTD)).await?"
    );
    println!();

    println!(
        "   create_and_post_market_order(user_market_order, tick, order_type: Option<OrderType>)"
    );
    println!("   - orderType defaults to FOK if None");
    println!(
        "   - Example: client.create_and_post_market_order(order, None, Some(OrderType::FAK)).await?"
    );
    println!();

    println!("   post_signed_order(signed_order, order_type: OrderType, defer_exec: bool)");
    println!("   - Requires explicit orderType parameter");
    println!("   - Example: client.post_signed_order(&order, OrderType::GTC, false).await?");
    println!();

    // Show TypeScript parity
    println!("5. TypeScript Parity:");
    println!("   TypeScript SDK:");
    println!("     createAndPostOrder(userOrder, options, orderType = OrderType.GTC)");
    println!("     createAndPostMarketOrder(userMarketOrder, options, orderType = OrderType.FOK)");
    println!();
    println!("   Rust SDK:");
    println!("     create_and_post_order(user_order, tick, order_type: Option<OrderType>)");
    println!(
        "     create_and_post_market_order(user_market_order, tick, order_type: Option<OrderType>)"
    );
    println!();

    println!("✅ Demo complete!");
    println!("\nKey Takeaways:");
    println!("  1. Both limit and market orders now support orderType parameter");
    println!("  2. orderType is wrapped in NewOrder structure before sending to API");
    println!("  3. Default orderTypes: GTC for limit orders, FOK for market orders");
    println!("  4. Full parity with TypeScript SDK orderType handling");

    Ok(())
}
