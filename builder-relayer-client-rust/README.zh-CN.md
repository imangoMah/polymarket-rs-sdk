# builder-relayer-client-rust（中文说明）

该 crate 提供用于 Builder/Relayer 流程和 Gnosis Safe 交互的构建器与编码辅助工具，侧重于生成 EIP-712 typed-data、MultiSend 编码以及 Safe 相关的交易请求构造。

功能概览
---------
- SafeTx 构建器：生成 Safe 交易类型的 typed-data、计算 struct hash 并准备签名负载。
- SafeCreate 构建器：用于部署或初始化 Safe 时的签名数据构建。
- MultiSend 编码：批量交易的 calldata 构造与选择器处理。
- 签名打包：v 字节归一化与签名拼装，保证与 TypeScript SDK 的一致性。

快速开始
---------
```bash
git clone <repo-url>
cd builder-relayer-client-rust
cargo build --release
```

运行示例（部分示例需凭证）：

```bash
cargo run --example deploy_safe
cargo run --example multisend_batch
```

主要模块
--------
- `builder::safe` — 构建 safe 交易请求与 safe create 请求的 helper。
- `signer` — 定义签名器 trait 与示例签名器（例如 `DummySigner`）。
- `encode::safe` — MultiSend 与 calldata 的编码工具。

示例输出（代表）
----------------
运行 `deploy_safe` 可能会输出（数值仅作示例）：

```
Transaction request built: { type: "safe", signature: "0x...", to: "0x..." }
Signed SafeTx signature: 0x...
```

常见问题（FAQ）
----------------
- Q: 支持哪些 signer 实现？
- A: 本 crate 定义了 signer trait，示例中包含 `DummySigner`。你可以实现 `AbstractSigner` 来集成硬件或远程签名器。

示例索引（简短说明）
------------------
- `approve_tokens.rs` — 授权代币额度（在提交交易前常用）。
- `builder_auth_execute.rs` — 构建 builder auth header 并执行交易的示例。
- `deploy_safe.rs` — 构建并签名 Safe 交易请求（使用 `SafeTransactionArgs`）。
- `deploy_safe_create.rs` — 构建 SafeCreate typed-data，用于初始化 Safe。
- `multisend_batch.rs` — 构建 MultiSend 批次及其 calldata。

贡献
----
- 为新签名或编码逻辑添加测试。
- 在提交 PR 前运行 `cargo fmt` 与 `cargo clippy`。

许可证
----
双重许可：MIT 或 Apache-2.0。
