use builder_relayer_client_rust::builder::create::build_safe_create_transaction_request;
use builder_relayer_client_rust::signer::{AbstractSigner, DummySigner};
use builder_relayer_client_rust::types::SafeCreateTransactionArgs;

#[tokio::main]
async fn main() {
    let signer =
        DummySigner::new("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
            .expect("signer");

    let args = SafeCreateTransactionArgs {
        from: format!("0x{:x}", signer.address()),
        chain_id: 137,                                                      // Polygon
        payment_token: "0x0000000000000000000000000000000000000000".into(), // Zero address for no payment
        payment: "0".into(),
        payment_receiver: "0x0000000000000000000000000000000000000000".into(),
    };

    // Gnosis Safe Factory on Polygon (from docs)
    let factory = "0xaacFeEa03eb1561C4e67d661e40682Bd20E3541b";

    let req = build_safe_create_transaction_request(&signer, factory, args)
        .await
        .expect("request");

    println!("Built SAFE CREATE tx request: {:?}", req);
}
