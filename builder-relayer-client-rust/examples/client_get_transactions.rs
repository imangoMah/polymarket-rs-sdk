use builder_relayer_client_rust::client::RelayClient;

#[tokio::main]
async fn main() {
    let client = RelayClient::new("https://relayer.example.com", 137);
    let list = client.get_transactions().await;
    println!("transactions: {:?}", list);
}
