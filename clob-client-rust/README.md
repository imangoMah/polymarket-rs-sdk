Polymarket Rust SDK
===================

This repository provides an integrated Rust SDK that unifies the functionality of the TypeScript SDK components: `clob-client`, `builder-signing`, `order-utils`, and builder relayer redeem/merge operations. It delivers a consolidated dependency and documentation surface so that order construction/signing, builder relayer workflows, and common utility calls can be performed within a single codebase.

Overview
--------
```
src/                Core library implementation
examples/           Usage examples aligned with typical TS scenarios
tests/              Unit / integration tests (more EIP-712 & flow cases to be added)
```

Feature Matrix
--------------
This is a modular feature checklist. Items that have been validated by automated tests or manual verification are marked with ✅; unmarked items are planned or pending additional work.

Signing & Keys
- [✅] Deterministic EIP-712 signatures (parity with TS SDK)
- [✅] EOA signature type support (SignatureType: EOA, etc.)
- [✅] L1 private key derive / create L2 API Key (Create / Derive)
- [✅] Builder API Key auth header generation (HMAC / Base64)
- [✅] Unified header construction: mixed L1/L2/Builder fallback

Orders
- [✅] Limit order build / sign / submit (create + post)
- [✅] Market order build / sign / submit (price computed externally)
- [✅] Explicit Tick Size (default fallback "0.01")
# clob-client-rust

A production-oriented Rust client for interacting with Polymarket's central limit order book (CLOB) services. This crate focuses on building, signing and submitting orders with an emphasis on EIP-712 parity with the existing TypeScript SDK.

## What this crate provides

- High-quality order builders (limit, market) with validation and tick-size handling.
- EIP-712 typed-data signing compatible with the TypeScript reference implementation.
- Typed HTTP helpers for interacting with CLOB endpoints (submit orders, query orders, etc.).
- Utilities for signature packing, order normalization and MultiSend encoding.
- Examples that demonstrate common flows and migration guidance from the TypeScript SDK.

## Quick start

```bash
git clone <repo-url>
cd clob-client-rust
# clob-client-rust

A production-oriented Rust client for interacting with Polymarket's central limit order book (CLOB) services. This crate focuses on building, signing and submitting orders with an emphasis on EIP-712 parity with the existing TypeScript SDK.

## What this crate provides

- High-quality order builders (limit, market) with validation and tick-size handling.
- EIP-712 typed-data signing compatible with the TypeScript reference implementation.
- Typed HTTP helpers for interacting with CLOB endpoints (submit orders, query orders, etc.).
- Utilities for signature packing, order normalization and MultiSend encoding.
- Examples that demonstrate common flows and migration guidance from the TypeScript SDK.

## Quick start

```bash
git clone <repo-url>
cd clob-client-rust
cargo build --release
```

Run an example that builds and signs an order (no network credentials required):

```bash
cargo run --example sign_order
```

## Examples

- `examples/sign_order.rs` — Build and sign a limit order locally.
- `examples/post_order.rs` — Build and POST an order to a configured CLOB endpoint (requires L2 credentials).
- `examples/create_and_post_order.rs` — Full create + submit flow used in integration tests.

## Environment variables

Configuration and credentials are injected via environment variables in examples and some runtime helpers:

- `PK` — L1 private key (hex, e.g. `0x...`).
- `CLOB_API_URL` — CLOB backend base URL (e.g. `https://api.polymarket.com`).
- `CHAIN_ID` — Chain ID (e.g. `137` or `80002`).
- `CLOB_API_KEY`, `CLOB_SECRET`, `CLOB_PASS_PHRASE` — L2 API credentials when required.
- `BUILDER_API_KEY`, `BUILDER_SECRET_B64`, `BUILDER_PASSPHRASE` — optional Builder auth credentials used by builder relayer flows.

## Integration

During development you can reference this crate as a path or git dependency. For publishing, depend on the crates.io release. See `INTEGRATION.md` for suggested dependency snippets and migration notes from the TypeScript SDK.

## Contributing

- Run tests with `cargo test` and ensure examples still run.
- Open issues for bugs or feature requests; submit pull requests for fixes and enhancements.

## License

Dual licensed under MIT OR Apache-2.0. See top-level LICENSE files for details.

For more developer notes and a complete example index see the repository `README.md` and `docs/` directory.

## Example output

Below are representative, minimal outputs you should expect when running a few local examples. Actual values (addresses, signatures, nonces) will vary.

- sign_order.rs

```
Signed order: 0x... (EIP-712 signature, packed)
Order payload: {"maker": "0x...", "taker": "0x...", "amount": "..."}
```

- post_order.rs (with valid credentials)

```
HTTP 200
{"result":"ok","order_id":"12345"}
```

## FAQ

- Q: Do I need an L2 API key to run examples?
- A: Examples that POST to the CLOB backend require L2 credentials (`CLOB_API_KEY`, `CLOB_SECRET`, `CLOB_PASS_PHRASE`). Local signing examples (e.g. `sign_order.rs`) do not.

- Q: How do I test signing parity with the TypeScript SDK?
- A: See `tests/` for compatibility tests. If you need a reference vector, run the TypeScript fixture and compare the packed signature hex.

## Examples index (auto-generated)

Below is a short description (purpose and common env vars) for each example file in `examples/`.

- `balance_allowance.rs` — Query token balance and allowance for a wallet. (Env: `PK`, `CLOB_API_URL`)
- `cancel_all.rs` — Cancel all open orders for an account (requires L2 creds). (Env: `CLOB_API_KEY`, `CLOB_SECRET`)
- `cancel_order.rs` — Cancel a single order by id. (Env: `CLOB_API_KEY`, `CLOB_SECRET`)
- `cancel_orders.rs` — Batch cancel orders by ids. (Env: `CLOB_API_KEY`, `CLOB_SECRET`)
- `compare_signing.rs` — Compare local EIP-712 signatures against a reference vector.
- `create_and_post_market_order.rs` — Build and submit a market order (external price required). (Env: `PK`, `CLOB_API_*`)
- `create_and_post_order.rs` — Create and submit a limit order end-to-end. (Env: `PK`, `CLOB_API_*`)
- `create_api_key.rs` — Create a new L2 API key (requires privileged credentials).
- `derive_api_key.rs` — Derive an L2 API key from an L1 key (demo utility).
- `get_api_keys.rs` — List API keys for the account (requires L2 creds).
- `get_markets.rs` — Fetch available markets and their metadata. (Env: `CLOB_API_URL`)
- `get_notifications.rs` — Retrieve notifications for an account.
- `get_open_orders.rs` — List open orders for a trader. (Env: `CLOB_API_URL`, `PK`)
- `get_order.rs` — Fetch details for a single order id.
- `get_order_book.rs` — Download order book for a market. (Env: `CLOB_API_URL`)
- `get_prices.rs` — Fetch pricing endpoints (mid, spreads). (Env: `CLOB_API_URL`)
- `get_trades.rs` — Retrieve recent trades for a market.
- `limit_buy_demo.rs` — Demo: construct and sign a limit buy order.
- `limit_sell_demo.rs` — Demo: construct and sign a limit sell order.
- `market_buy_demo.rs` — Demo: market buy using local price helpers.
- `market_buy_order.rs` — Build and sign a market buy order.
- `market_sell_demo.rs` — Demo: market sell using local price helpers.
- `market_sell_order.rs` — Build and sign a market sell order.
- `order_types_demo.rs` — Show available order types and validation rules.
- `post_order.rs` — Build and POST an order to the CLOB service. (Env: `CLOB_API_*`)
- `query_orders_demo.rs` — Example for querying orders with filters.
- `rewards.rs` — Fetch/recompute rewards summary (if supported by backend).
- `scoring.rs` — Demo of order scoring endpoint usage.
- `sign_order.rs` — Build and locally sign an order (no network). (Env: `PK`)
- `signature_types_demo.rs` — Explore different signature packing formats.
- `test_address_case.rs` — Utility to test address normalization.

