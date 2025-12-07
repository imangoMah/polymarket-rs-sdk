use crate::errors::Result;
use alloy::primitives::{keccak256, Address, B256};
use alloy::sol_types::SolValue;
use std::str::FromStr;

// SAFE_INIT_CODE_HASH from TS constants
pub const SAFE_INIT_CODE_HASH: &str =
    "0x2bce2127ff07fb632d16c8347c4ebf501f4841168bed00d9e6ef715ddb6fcecf";
// PROXY_INIT_CODE_HASH from TS constants
pub const PROXY_INIT_CODE_HASH: &str =
    "0xd21df8dc65880a8606f09fe0ce3df9b8869287ab0b058be05aa9e8af6330a00b";

pub fn derive_safe(factory: Address, owner: Address) -> Result<Address> {
    let init_code_hash = B256::from_str(SAFE_INIT_CODE_HASH).expect("invalid hash");

    // salt = keccak256(abi.encode(owner))
    let encoded = owner.abi_encode();
    let salt = keccak256(&encoded);

    let addr = factory.create2(salt, init_code_hash);
    Ok(addr)
}

pub fn derive_proxy(factory: Address, owner: Address) -> Result<Address> {
    let init_code_hash = B256::from_str(PROXY_INIT_CODE_HASH).expect("invalid hash");

    // TS 使用 encodePacked(address) => keccak(address 20 bytes)
    let salt = keccak256(owner.as_slice());

    Ok(factory.create2(salt, init_code_hash))
}
