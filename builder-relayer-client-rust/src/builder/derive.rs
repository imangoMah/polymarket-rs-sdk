use ethers::types::{Address, H256};
use ethers::utils::{get_create2_address_from_hash, keccak256};

// SAFE_INIT_CODE_HASH from TS constants
pub const SAFE_INIT_CODE_HASH: &str =
    "0x2bce2127ff07fb632d16c8347c4ebf501f4841168bed00d9e6ef715ddb6fcecf";

pub fn derive_safe(owner: &str, safe_factory: &str) -> String {
    let factory: Address = safe_factory.parse().expect("invalid factory");
    let init_code_hash: H256 = SAFE_INIT_CODE_HASH.parse().expect("invalid hash");
    let owner_addr: Address = owner.parse().expect("invalid owner");

    // salt = keccak256(abi.encode(address))
    use ethers::abi::{encode, Token};
    let salt_bytes = keccak256(&encode(&[Token::Address(owner_addr)]));
    let salt = H256::from_slice(&salt_bytes);
    let addr = get_create2_address_from_hash(factory, salt, init_code_hash);
    format!("0x{:x}", addr)
}
