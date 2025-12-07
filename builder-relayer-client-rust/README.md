# builder-relayer-client-rust

Utilities and helpers for building, signing and encoding transactions intended for Builder/Relayer workflows and Gnosis Safe interactions.

## Overview

This crate focuses on producing properly-formed EIP-712 typed data and transaction payloads for two main flows:

- Safe transaction signing (SafeTx): build type-packed payloads, compute struct hash, and sign with EIP-712 compatible signatures.
- Safe creation (SafeCreate): helper to build signing payloads used when deploying/initializing a Safe via the builder relayer.

The crate intentionally separates typed-data construction logic from signing implementations so that alternative signers (hardware keys, remote signers) can be integrated.

## Key features

- EIP-712 typed-data builders for Safe and SafeCreate flows.
- SafeTransaction args normalization (optional `safe_address` derivation, transaction packing, operation type handling).
- MultiSend calldata encoding and selector helpers.
- Signature packing and v-byte normalization to match TypeScript SDK behavior.

## Quick start

```bash
git clone <repo-url>
cd builder-relayer-client-rust
cargo build --release
```

Run examples (some require credentials):

```bash
cargo run --example deploy_safe
cargo run --example multisend_batch
```

## API surface

Public modules of interest:

- `builder::safe` — helpers to build safe transaction requests and safe create requests.
- `signer` — traits and test signers (e.g. `DummySigner`) used by examples and tests.
- `encode::safe` — encoding helpers for MultiSend and calldata.

## Environment variables

- `PK` — local EOA private key used by `DummySigner` examples.
- `BUILDER_API_KEY`, `BUILDER_SECRET_B64`, `BUILDER_PASSPHRASE` — used when constructing builder auth headers.

## Tests & Verification

This crate contains a set of compatibility tests (`tests/signature_compatibility_tests.rs`) which assert parity against known TypeScript reference outputs for signatures and encodings. Run `cargo test` to execute them locally.

## Contributing

- Add tests for any new signing or encoding behavior you introduce.
- Run `cargo fmt` and `cargo clippy` when filing PRs.

## License

Dual-licensed under MIT OR Apache-2.0. See LICENSE files at repository root.

## Example output

Representative output from running `deploy_safe` (values will differ):

```
Transaction request built: { type: "safe", signature: "0x...", to: "0x..." }
Signed SafeTx signature: 0x...
```

## FAQ

- Q: What signer implementations are supported?
- A: The crate defines signer traits and ships a `DummySigner` for examples/tests. You can implement `AbstractSigner` to plug hardware or remote signers.

## Examples index (auto-generated)

Short descriptions for the examples included in `examples/`:

- `approve_tokens.rs` — Approve token allowance used before submitting orders or interacting with contracts.
- `builder_auth_execute.rs` — Demonstrates building a builder auth header and executing a transaction.
- `client_execute.rs` — Example of client-side execute flow against a relayer.
- `client_get_transactions.rs` — Poll relayer for transactions associated with a request.
- `client_poll.rs` — Polling pattern example for long-running requests.
- `ctf_operations.rs` — Demo of CTF-specific operations (project-specific).
- `deploy_safe.rs` — Build and sign a Safe transaction request; uses `SafeTransactionArgs`.
- `deploy_safe_create.rs` — Build the SafeCreate typed data to initialize a new Safe.
- `monitor_transactions.rs` — Monitor transaction status via the relayer / provider.
- `multisend_batch.rs` — Build a MultiSend batch and its calldata.
- `quick_start.rs` — Short script demonstrating the basic auth + build + sign flow.
- `relayer_client_demo.rs` — End-to-end relayer client demo.

