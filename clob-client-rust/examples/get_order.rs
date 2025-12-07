use clob_client_rust::ClobClient;
use clob_client_rust::errors::ClobError;

// Example: Get order details by order ID
// Run with: cargo run --example get_order -- <ORDER_ID>
//
// This corresponds to TypeScript SDK's getOrder() method:
// const order = await client.getOrder(orderId);

#[tokio::main]
async fn main() -> Result<(), ClobError> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run --example get_order -- <ORDER_ID>");
        eprintln!("\nExample:");
        eprintln!("  cargo run --example get_order -- 0x1234567890abcdef...");
        std::process::exit(1);
    }

    let order_id = &args[1];

    println!("🔍 Getting order details...");
    println!("   Order ID: {}\n", order_id);

    // Create client (no auth required for public read)
    let host =
        std::env::var("CLOB_API_URL").unwrap_or_else(|_| "https://clob.polymarket.com".to_string());

    let client = ClobClient::new(&host, 137, None, None, false);

    // Get order details
    match client.get_order(order_id).await {
        Ok(order) => {
            println!("✅ Order found!");
            println!("\n📋 Order Details:");

            println!("   ID: {}", order.id);
            println!("   Status: {}", order.status);
            println!("   Owner (API Key): {}", order.owner);
            println!("   Maker Address: {}", order.maker_address);
            println!("   Market: {}", order.market);
            println!("   Asset ID: {}", order.asset_id);
            println!("   Side: {}", order.side);
            println!("   Outcome: {}", order.outcome);
            println!("   Price: {}", order.price);
            println!("   Original Size: {}", order.original_size);
            println!("   Size Matched: {}", order.size_matched);
            println!("   Order Type: {}", order.order_type);
            println!("   Expiration: {}", order.expiration);
            println!("   Created At: {}", order.created_at);

            if !order.associate_trades.is_empty() {
                println!("   Associated Trades: {}", order.associate_trades.len());
                for (i, trade_id) in order.associate_trades.iter().enumerate() {
                    println!("     {}. {}", i + 1, trade_id);
                }
            }

            println!("\n💡 Usage in code:");
            println!("   let order = client.get_order(\"{}\").await?;", order_id);
            println!("   // Or use TypeScript-style naming:");
            println!("   let order = client.getOrder(\"{}\").await?;", order_id);
        }
        Err(e) => {
            eprintln!("❌ Error getting order: {}", e);
            eprintln!("\n💡 Possible reasons:");
            eprintln!("   - Order ID does not exist");
            eprintln!("   - Invalid order ID format");
            eprintln!("   - Network connection issues");
            return Err(e);
        }
    }

    Ok(())
}
