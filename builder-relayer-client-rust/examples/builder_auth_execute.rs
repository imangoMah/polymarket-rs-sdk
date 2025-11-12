use base64::Engine;
use builder_relayer_client_rust::client::RelayClient;
use builder_relayer_client_rust::signer::DummySigner;
use builder_relayer_client_rust::types::{OperationType, SafeTransaction};
use builder_signing_sdk_rs::BuilderApiKeyCreds; // bring trait into scope for .encode()

#[tokio::main]
async fn main() {
    // 占位私钥与 API Key，需替换为真实值
    let signer =
        DummySigner::new("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
            .unwrap();
    let creds = BuilderApiKeyCreds {
        key: "demo_key".into(),
        secret: base64::engine::general_purpose::STANDARD.encode("demo_secret"),
        passphrase: "demo_pass".into(),
    };
    let client = RelayClient::new("https://relayer.example.com", 137)
        .with_signer(Box::new(signer.clone()), Box::new(signer))
        .with_builder_api_key(creds);

    let tx = SafeTransaction {
        to: "0x000000000000000000000000000000000000dead".into(),
        value: "0".into(),
        data: "0x".into(),
        operation: OperationType::Call,
    };
    let res = client
        .execute(vec![tx], Some("with-builder-auth".into()))
        .await;
    println!("builder authed execute response: {:?}", res);
}
