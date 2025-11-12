use builder_relayer_client_rust::builder::safe::{
    build_safe_transaction_request, SafeContractConfig, SignatureMode,
};
use builder_relayer_client_rust::signer::{AbstractSigner, DummySigner};
use builder_relayer_client_rust::types::{OperationType, SafeTransaction, SafeTransactionArgs};

#[tokio::main]
async fn main() {
    let signer =
        DummySigner::new("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
            .unwrap();
    let base_from = format!("0x{:x}", signer.address());
    let tx1 = SafeTransaction {
        to: "0x000000000000000000000000000000000000dead".into(),
        value: "0".into(),
        data: "0x".into(),
        operation: OperationType::Call,
    };
    let tx2 = SafeTransaction {
        to: "0x000000000000000000000000000000000000beef".into(),
        value: "0".into(),
        data: "0x1234".into(),
        operation: OperationType::Call,
    };
    let args = SafeTransactionArgs {
        from: base_from,
        nonce: "2".into(),
        chain_id: 137,
        transactions: vec![tx1, tx2],
        safe_address: None,
    };
    let cfg = SafeContractConfig {
        safe_factory: "0xFactoryAddress".into(),
        safe_multisend: "0xMultiSendAddress".into(),
    };
    let req = build_safe_transaction_request(&signer, args, cfg, None, SignatureMode::Eip712Digest)
        .await
        .unwrap();
    println!("Multisend SAFE tx request: {:?}", req.data);
}
