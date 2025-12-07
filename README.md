# Polymarket Rust SDK (monorepo)

This repository contains a set of Rust crates and tools that implement the core functionality of Polymarket's SDK surface in Rust. The goal is to provide parity with the TypeScript SDK for order construction, signing (EIP-712), builder relayer flows, and small utility crates that can be published independently to crates.io.

Top-level layout
---------------
- `clob-client-rust/` — CLOB client and helpers (order builders, signing, submission helpers). See `clob-client-rust/README.md` for details and examples.
- `builder-relayer-client-rust/` — Builders and relayer helpers for Gnosis Safe / Builder flows (SafeTx / SafeCreate helpers). See `builder-relayer-client-rust/README.md`.
- `builder_signing_sdk_rs/` — Small signing helper crate (HMAC, EIP-712 helper utilities). Published independently.
- `order_utils_rs/` — Small order helper utilities (normalization, small helpers). Published independently.
- `examples/`, `docs/` — collection of cross-repo examples and developer docs. Prefer per-crate `examples/` for runnable demos.

Why this repo
---------------
- Consolidation: merge TypeScript SDK responsibilities into a typed Rust implementation for services that prefer or require Rust-based integration.
- Publishable crates: small, focused crates are split so they can be published independently and consumed by other projects.
- Tests and compatibility: contains tests that assert parity (signatures, encodings) against existing TypeScript outputs.

Quick start
-----------
Build everything (recommended for development):

```bash
git clone <repo-url>
cd polymarket-rust-sdk
cargo build --all
```

Or build a single crate, for example the CLOB client:

```bash
cd clob-client-rust
cargo build --release
```

Dependency
----------
Use the published crates from crates.io to consume functionality from this repository. Two small helper crates have been published and can be added to your project with either the `cargo` helper or by adding a dependency entry to your `Cargo.toml`.

Add via cargo (recommended):

```bash
cargo add builder_signing_sdk_rs
cargo add order_utils_rs
```

Or add the following lines to your `Cargo.toml`:

```toml
[dependencies]
builder_signing_sdk_rs = "0.1.0"
order_utils_rs = "0.1.0"
```

If you want to depend on the higher-level crates after they're published, use the same pattern, for example:

```bash
cargo add clob-client-rust
cargo add builder-relayer-client-rust
```

or in `Cargo.toml`:

```toml
[dependencies]
clob-client-rust = "0.1.0"
builder-relayer-client-rust = "0.1.0"
```

During development you can continue to use path or git dependencies; switch to crates.io versions for published releases.

Contribution & development
--------------------------
- Run unit tests: `cargo test --all`.
- Run examples: `cargo run --example <name>` from the target crate directory.
- Formatting and linting: `cargo fmt` and `cargo clippy` are recommended before submitting PRs.

Publishing to crates.io
-----------------------
This repo includes a helper script to publish selected crates to crates.io. The script reads the registry token from a local `.env` file and publishes the remaining crates in dependency-safe order.

1) Create `.env` at repo root with your token:

```
CARGO_REGISTRY_TOKEN = your_crates_io_token_here
```

2) Dry run (recommended):

```bash
DRY_RUN=1 scripts/publish_crates.sh
```

3) Real publish:

```bash
scripts/publish_crates.sh
```

Notes:
- The script currently targets `builder-relayer-client-rust` and `clob-client-rust`.
- Ensure `builder_signing_sdk_rs` is available on crates.io at the version required by dependents (e.g., `0.1.0`).
- If crates.io indexing is slow, the script includes a small delay between publishes.

Where to look next
------------------
- For order-building and signing examples, open `clob-client-rust/README.md`.
- For Safe / Builder relayer helpers and examples, open `builder-relayer-client-rust/README.md`.
- The small helper crates `builder_signing_sdk_rs/` and `order_utils_rs/` contain their own README and are intended to be published to crates.io individually.

License
-------
Dual licensed under MIT OR Apache-2.0. See the root LICENSE files for details.
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
- [ ] Cancel orders: single / batch / all open
- [✅] FOK / FAK order types with boundary validation
- [ ] Order scoring status query

Markets & Data
- [ ] Market list retrieval (get_markets)
- [ ] Order book retrieval (get_order_book)
- [ ] Pricing endpoints (mid, spreads, prices)
- [ ] Trade list (get_trades)
- [ ] Notifications (get_notifications)
- [ ] Rewards (daily aggregation)

Accounts & Permissions
- [ ] Balance / allowance query & optional update (balance_allowance)
- [✅] API Keys management (get_api_keys, etc.)

Builder Redeem / Merge
- [✅] Create Redeem request (single or multi-market)
- [ ] Batch Redeem
- [ ] Merge / Unmerge share operations
- [✅] Gas / Fee estimation (Estimate Gas/Fee)
- [✅] Status polling & receipt retrieval (Redeem/Merge)
- [ ] Idempotency key & dedup control
- [✅] Retry & error categorization
- [ ] Dry-run / simulation (no on-chain broadcast)

WebSocket / Realtime
- [ ] Realtime subscriptions (spreads / trades / order events) (planned)

Engineering & Types
- [✅] Strong typed data structures (avoid raw serde_json::Value)
- [✅] HTTP helpers (typed deserialization / basic retry)
- [✅] Alias methods (camelCase + snake_case for TS migration)

Examples & Scripts
- [✅] `sign_order` / `post_order`
- [✅] `create_and_post_order` / `create_and_post_market_order`
- [✅] `market_buy_order` / `market_sell_order`
- [✅] `get_markets` / `get_order_book` / `get_open_orders`
- [ ] Cancels: `cancel_order` / `cancel_orders` / `cancel_all`
- [✅] Key management: `create_api_key` / `derive_api_key` / `get_api_keys`
- [ ] Others: `get_trades` / `get_notifications` / `rewards` / `balance_allowance` / `get_prices`

Parity
------
- [✅] Signature parity with TS (salt, signature, maker/taker amounts)
- [✅] TickSize strategy parity (no implicit backend fetch; default or explicit)

Installation & Build
--------------------
```bash
git clone <repo-url>
cd clob-client-rust
cargo build
```

Usage in your project (see INTEGRATION.md):
```toml
# Add dependency stanza here; path or git variant.
```

Environment Variables
---------------------
| Name | Description | Example |
|------|-------------|---------|
| PK | L1 private key (0x + 64 hex) | 0x... |
| CLOB_API_URL | CLOB backend base URL | https://api.polymarket.com |
| CHAIN_ID | Chain ID | 80002 / 137 |
| CLOB_API_KEY / CLOB_SECRET / CLOB_PASS_PHRASE | L2 API credentials | provisioned |
| BUILDER_API_KEY / BUILDER_SECRET_B64 / BUILDER_PASSPHRASE | Builder auth (optional) | provisioned |

Quick Example
-------------
Sign without submitting (build only when no API credentials):
```bash
cargo run --example sign_order
```

Example Index
-------------
| Example | Purpose | Env Vars |
|---------|---------|----------|
| `sign_order` | Build & sign limit order | PK |
| `post_order` | Build then (demo skip real submit) | PK |
| `create_and_post_order` | Limit: create + submit | PK, CLOB_API_KEY/SECRET/PASS_PHRASE |
| `create_and_post_market_order` | Market: create + submit (external price calc) | PK, CLOB_API_KEY/SECRET/PASS_PHRASE |
| `market_buy_order` / `market_sell_order` | Construct market buy/sell | PK |
| `get_markets` / `get_order_book` | Market list / order book | (opt) CLOB_API_URL |
| `get_open_orders` | Current open orders | (L2 if auth endpoint) |
| `cancel_order` | Cancel single | PK + L2 creds |
| `cancel_orders` | Batch cancel | PK + L2 creds |
| `cancel_all` | Cancel all open | PK + L2 creds |
| `create_api_key` / `derive_api_key` | Create or derive L2 key | PK |
| `get_api_keys` | List existing API keys | PK + L2 creds |
| `get_trades` | Trades (first page) | PK + L2 creds |
| `get_notifications` | Notifications | PK + L2 creds |
| `rewards` | Daily rewards data | (backend dependent) |
| `scoring` | Order scoring status | (Builder auth optional) |
| `balance_allowance` | Balance / allowance query (+ optional update) | PK + L2 creds |
| `get_prices` | Mid / spreads / prices | public / semi-public |

TypeScript Parity Notes
-----------------------
* camelCase aliases (e.g. `getMarkets`) coexist with snake_case for smoother TS migration.
* EIP-712 salt/structure ensures identical signatures vs TS (tested).
* TickSize behavior: no implicit backend fetch; call `get_tick_size` externally if you need the true value, otherwise default "0.01" applies.
* `OrderBuilder` supports limit & market orders, current extension points reserved.

Architecture
------------
| Module | Purpose |
|--------|---------|
| `order_builder` | Build/sign orders; price & amount normalization |
| `client` | HTTP wrapper; caches tick/fee/neg_risk; aliases & auth headers |
| `headers` | L1/L2/Builder header generation (HMAC / EIP-712 derived) |
| `types` | Strong typed data structures (avoid raw JSON) |
| `http_helpers` | Basic request wrapper + typed deserialization |

Dev Tips
--------
1. Use a local mock/dev backend and set `CLOB_API_URL` accordingly.
2. For TS parity checks, run the original SDK in parallel and diff signature outputs.
3. Add `RUST_LOG=debug` for verbose tracing (future migration to structured tracing possible).

Required Fields (Breaking Changes)
---------------------------------
| Struct | Field | Description |
|--------|-------|-------------|
| `UserOrder` | `fee_rate_bps: f64` | Limit order fee (bps) |
| `UserMarketOrder` | `price: f64` | Market order external computed price |
| `UserMarketOrder` | `fee_rate_bps: f64` | Market order fee |
| `UserMarketOrder` | `order_type: OrderType` | FOK / FAK required |

Signature Parity Test (Rust vs TypeScript)
-----------------------------------------
1. Generate `SignedOrder` using identical inputs in TS and Rust.
2. Compare: `maker_amount`, `taker_amount`, `salt`, `signature`.
3. Builder auth only alters request headers, not the signature body.

Market Order Pricing Summary
* BUY: accumulate asks (size * price) top-down until target USDC met. FOK errors if insufficient; FAK uses best partial.
* SELL: accumulate bids high-to-low until target shares met. FOK errors if insufficient; FAK uses best partial.

Roadmap / TODO
--------------
* [ ] More Builder features (expanded scoring query fields)
* [ ] WebSocket / realtime subscriptions (spreads / trades / order events)
* [ ] Richer error taxonomy (HTTP code -> domain error)
* [ ] Pluggable signature adapter trait
* [ ] CI: GitHub Actions (build + test + clippy + fmt)

FAQ
---
**Q:** Why are some fields `Option<String>`?
**A:** Backend responses vary; optional keeps deserialization resilient.

**Q:** Why both camelCase and snake_case methods?
**A:** Migration convenience; snake_case is canonical internally, camelCase is a thin alias.

**Q:** What if Tick Size isn't provided?
**A:** Default "0.01" is used; call `get_tick_size` beforehand for the authoritative value.

License
-------
Same as original SDK (formal LICENSE not yet embedded; confirm before production use).

Contributing
------------
PRs welcome: keep diffs minimal, add corresponding tests, ensure `cargo clippy -D warnings` passes.


