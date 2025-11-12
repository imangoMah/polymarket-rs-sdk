pub mod client;
pub mod constants;
pub mod endpoints;
pub mod errors;
pub mod exchange_consts;
pub mod exchange_order_builder;
pub mod headers;
pub mod http_helpers;
pub mod order_builder;
pub mod signer_adapter;
pub mod signing;
pub mod types;
pub mod utilities;

pub use client::*;
pub use constants::*;
pub use endpoints::*;
pub use errors::*;
pub use exchange_consts::*;
pub use exchange_order_builder::*;
pub use headers::*;
pub use http_helpers::*;
pub use order_builder::*;
pub use signer_adapter::*;
pub use signing::*;
pub use types::*;
pub use utilities::*;

/// Simple smoke function to verify the crate is usable
pub fn client_version() -> &'static str {
    "0.1.0-rust-translation"
}
