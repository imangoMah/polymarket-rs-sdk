# Polymarket Rust SDK（中文说明）

本仓库为 Polymarket 提供了一套 Rust 实现的 SDK 集合（monorepo）。目标是与现有 TypeScript SDK 在订单构建、EIP-712 签名、Builder/Relayer 流程以及若干独立工具上达到功能与兼容性，便于在 Rust 环境中集成与发布可复用的 crate。

仓库结构概览
----------------
- `clob-client-rust/` — CLOB 客户端与工具（订单构建、签名、提交帮助器）。详见 `clob-client-rust/README.md`。
- `builder-relayer-client-rust/` — 与 Builder/Relayer、Gnosis Safe 相关的构建器与辅助工具（SafeTx / SafeCreate）。详见 `builder-relayer-client-rust/README.md`。
- `builder_signing_sdk_rs/` — 小型签名工具库（HMAC、EIP-712 助手），可独立发布到 crates.io。
- `order_utils_rs/` — 订单相关的小工具（归一化、辅助函数），可独立发布。
- `examples/`、`docs/` — 跨仓库或面向开发者的示例与文档；更详细的可在各子包的 `examples/` 下查看可运行示例。

为什么使用这个仓库
---------------------
- 整合：将 TypeScript SDK 的责任逐步以类型安全的 Rust 实现复刻，便于后端或工具链使用 Rust。
- 可发布的子包：小而专注的 crate 被拆分，便于单独发布与复用。
- 兼容性测试：含有对签名与编码的兼容性测试，以确保与 TS 实现一致。

快速开始
---------
构建整个工作区（开发时推荐）：

```bash
git clone <repo-url>
cd polymarket-rust-sdk
cargo build --all
```

构建单个 crate（例如 CLOB 客户端）：

```bash
cd clob-client-rust
cargo build --release
```

依赖说明
-------
可以直接从 crates.io 添加已发布的 crate 来使用仓库提供的功能。两个已发布的小型 helper crate 可通过 `cargo` 命令或直接在 `Cargo.toml` 中添加依赖：

通过 cargo 添加（推荐）：

```bash
cargo add builder_signing_sdk_rs
cargo add order_utils_rs
```

或者在 `Cargo.toml` 中添加：

```toml
[dependencies]
builder_signing_sdk_rs = "0.1.0"
order_utils_rs = "0.1.0"
```

若要依赖更高层的 crate（例如 clob-client-rust 或 builder-relayer-client-rust），使用相同方式：

```bash
cargo add clob-client-rust
cargo add builder-relayer-client-rust
```

或在 `Cargo.toml` 中：

```toml
[dependencies]
clob-client-rust = "0.1.0"
builder-relayer-client-rust = "0.1.0"
```

开发期间仍可使用 path 或 git 依赖；发布时建议切换为 crates.io 版本。

贡献与开发
-----------
- 运行单元测试：`cargo test --all`。
- 运行示例：在对应 crate 目录下执行 `cargo run --example <name>`。
- 格式与静态检查：请使用 `cargo fmt` 与 `cargo clippy`。

查看哪一部分
-------------
- 要查看订单构建与签名示例，请打开 `clob-client-rust/README.md`。
- 要查看 Safe / Builder relayer 的 helper 与示例，请打开 `builder-relayer-client-rust/README.md`。
- 小工具 crate（`builder_signing_sdk_rs/`、`order_utils_rs/`）包含各自 README，适合直接消费和发布。

许可证
-----
双重许可：MIT 或 Apache-2.0。详细信息请参阅根目录的 LICENSE 文件。
