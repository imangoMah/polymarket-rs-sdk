# 示例代码说明

⚠️ **注意**: 当前示例代码需要根据实际 API 调整。

## 快速开始

基本使用示例请参考 `tests/` 目录中的测试代码,这些是经过验证可以工作的。

## RelayClient API

```rust
use builder_relayer_client_rust::RelayClient;
use builder_relayer_client_rust::signer::DummySigner;
use builder_signing_sdk_rs::BuilderApiKeyCreds;

// 1. 创建签名器
let signer = DummySigner::new("your-private-key")?;

// 2. 创建客户端
let client = RelayClient::new("https://relayer-v2.polymarket.com/", 137)
    .with_signer(Box::new(signer.clone()), Box::new(signer))
    .with_builder_api_key(BuilderApiKeyCreds {
        key: "your_api_key".to_string(),
        secret: "your_secret".to_string(),
        passphrase: "your_passphrase".to_string(),
    });

// 3. 部署 Safe
let response = client.deploy().await?;

// 4. 等待确认
let result = client.poll_until_state(
    &response.transaction_id,
    &[RelayerTransactionState::Confirmed],
    Some(RelayerTransactionState::Failed),
    30,
    2000,
).await?;
```

## 执行交易

```rust
use builder_relayer_client_rust::{SafeTransaction, OperationType};

// 创建交易
let tx = SafeTransaction {
    to: "0x...".to_string(),
    operation: OperationType::Call,
    data: "0x...".to_string(),
    value: "0".to_string(),
};

// 执行
let response = client.execute(
    vec![tx],
    Some("交易描述".to_string()),
).await?;

// 等待确认
let result = client.poll_until_state(
    &response.transaction_id,
    &[RelayerTransactionState::Confirmed],
    Some(RelayerTransactionState::Failed),
    30,
    2000,
).await?;
```

## 环境变量

创建 `.env` 文件:

```bash
PRIVATE_KEY=0x...
BUILDER_API_KEY=...
BUILDER_SECRET=...
BUILDER_PASS_PHRASE=...
```

## 更多示例

查看 `tests/` 目录获取更多可运行的测试示例:

```bash
cargo test -p builder-relayer-client-rust
```

---

**待办**: 示例文件需要重写以匹配当前 API。请参考测试代码和 `RELAYER_CLIENT_GUIDE.md` 获取最新用法。
