use builder_relayer_client_rust::client::RelayClient;
use builder_relayer_client_rust::signer::DummySigner;
use builder_relayer_client_rust::types::{OperationType, SafeTransaction};

#[tokio::main]
async fn main() {
    let signer =
        DummySigner::new("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
            .unwrap();
    let client = RelayClient::new("https://relayer.example.com", 137)
        .with_signer(Box::new(signer.clone()), Box::new(signer));

    let tx = SafeTransaction {
        to: "0x000000000000000000000000000000000000dead".into(),
        value: "0".into(),
        data: "0x".into(),
        operation: OperationType::Call,
    };
    let res = client.execute(vec![tx], None).await;
    println!("execute response: {:?}", res);
}
