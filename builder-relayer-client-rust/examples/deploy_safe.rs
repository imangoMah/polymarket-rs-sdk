use builder_relayer_client_rust::builder::safe::{
    build_safe_transaction_request, SafeContractConfig, SignatureMode,
};
use builder_relayer_client_rust::signer::{AbstractSigner, DummySigner};
use builder_relayer_client_rust::types::{SafeTransaction, SafeTransactionArgs};

#[tokio::main]
async fn main() {
    let signer =
        DummySigner::new("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
            .expect("signer");
    let args = SafeTransactionArgs {
        from: format!("0x{:x}", signer.address()),
        nonce: "1".to_string(),
        chain_id: 137, // polygon example
        transactions: vec![SafeTransaction {
            to: "0x000000000000000000000000000000000000dead".into(),
            value: "0".into(),
            data: "0x".into(),
            operation: builder_relayer_client_rust::types::OperationType::Call,
        }],
        safe_address: None,
    };
    let cfg = SafeContractConfig {
        safe_factory: "0xFactoryAddress".into(),
        safe_multisend: "0xMultiSendAddress".into(),
    };
    let req = build_safe_transaction_request(&signer, args, cfg, None, SignatureMode::Eip712Digest)
        .await
        .expect("request");
    println!("Built SAFE tx request: {:?}", req);
}
