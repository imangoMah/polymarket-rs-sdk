use clob_client_rust::client::ClobClient;
use clob_client_rust::order_builder::OrderBuilder;
use clob_client_rust::signer_adapter::EthersSigner;
use clob_client_rust::types::{ApiKeyCreds, OrderType, Side, UserMarketOrder};
use serde_json::json;
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Demo private key (deterministic for example). Replace with secure key in real usage.
    let priv_hex = "0x0123456789012345678901234567890123456789012345678901234567890123".to_string();
    println!("Using demo private key: {}", &priv_hex);

    let signer = match EthersSigner::new_from_private_key(&priv_hex) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("failed to create ethers signer: {:?}", e);
            return;
        }
    };

    let ob = OrderBuilder::new(&signer, 137, None, None);

    let user_market_order = UserMarketOrder {
        token_id: "1".to_string(),
        price: 1.23,
        amount: 0.5,
        side: Side::BUY,
        fee_rate_bps: 1.0,
        nonce: None,
        taker: None,
        order_type: OrderType::GTC,
    };

    let signed = match ob
        .build_market_order(
            "0x0000000000000000000000000000000000000001",
            &user_market_order,
            "0.01",
        )
        .await
    {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error building/signing order: {:?}", e);
            return;
        }
    };

    // If environment provides a host, attempt to POST the signed order using ClobClient.
    if let Ok(host) = env::var("CLOB_HOST") {
        println!("CLOB_HOST set: attempting post to {}", host);
        // Optionally read API creds from env
        let creds = if let (Ok(key), Ok(secret), Ok(pass)) = (
            env::var("CLOB_API_KEY"),
            env::var("CLOB_API_SECRET"),
            env::var("CLOB_API_PASSPHRASE"),
        ) {
            Some(ApiKeyCreds {
                key,
                secret,
                passphrase: pass,
            })
        } else {
            None
        };

        let signer_arc = Arc::new(signer);
        let client = ClobClient::new(&host, 137, Some(signer_arc), creds, false);

        let _order_json = json!({
            "salt": signed.salt,
            "maker": signed.maker,
            "signer": signed.signer,
            "taker": signed.taker,
            "tokenId": signed.token_id,
            "makerAmount": signed.maker_amount,
            "takerAmount": signed.taker_amount,
            "expiration": signed.expiration,
            "nonce": signed.nonce,
            "feeRateBps": signed.fee_rate_bps,
            "side": match signed.side { Side::BUY => "BUY", Side::SELL => "SELL" },
            // Only EOA supported at the moment; map directly to 0
            "signatureType": 0,
            "signature": signed.signature,
        });

        // Use the typed helper to post the signed order
        match client
            .post_signed_order(&signed, OrderType::GTC, false)
            .await
        {
            Ok(resp) => println!("post_order response: {:?}", resp),
            Err(e) => eprintln!("post_order error: {:?}", e),
        }
    } else {
        println!("Signed order: {:#?}", signed);
        println!("Set CLOB_HOST to attempt POSTing the signed order.");
    }
}
