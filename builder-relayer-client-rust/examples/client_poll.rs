use builder_relayer_client_rust::client::RelayClient;
use builder_relayer_client_rust::types::RelayerTransactionState;

#[tokio::main]
async fn main() {
    let client = RelayClient::new("https://relayer.example.com", 137);
    let id = "tx_123";
    let states = [
        RelayerTransactionState::StateMined,
        RelayerTransactionState::StateConfirmed,
    ];
    let out = client
        .poll_until_state(
            id,
            &states,
            Some(RelayerTransactionState::StateFailed),
            10,
            500,
        )
        .await;
    println!("poll result: {:?}", out);
}
