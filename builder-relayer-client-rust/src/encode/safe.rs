use crate::types::{OperationType, SafeTransaction};
use ethers::abi::{Token, encode};
use hex::ToHex;

// pack single transaction like Solidity struct used by Gnosis Safe MultiSend
fn pack_tx(tx: &SafeTransaction) -> Vec<u8> {
    // bytes: operation (1) + to (20) + value (32) + data length (32) + data
    let mut out = Vec::new();
    out.push(tx.operation as u8);
    // address
    let addr_bytes = hex::decode(tx.to.trim_start_matches("0x")).unwrap_or_default();
    let mut addr_fixed = vec![0u8; 20];
    if addr_bytes.len() == 20 {
        addr_fixed.copy_from_slice(&addr_bytes);
    } else if addr_bytes.len() > 20 {
        addr_fixed.copy_from_slice(&addr_bytes[addr_bytes.len() - 20..]);
    }
    out.extend_from_slice(&addr_fixed);
    // value (uint256)
    let value = ethers::types::U256::from_dec_str(&tx.value).unwrap_or_default();
    let mut value_be = [0u8; 32];
    value.to_big_endian(&mut value_be);
    out.extend_from_slice(&value_be);
    // data length (uint256)
    let data_bytes = hex::decode(tx.data.trim_start_matches("0x")).unwrap_or_default();
    let mut len_be = [0u8; 32];
    ethers::types::U256::from(data_bytes.len()).to_big_endian(&mut len_be);
    out.extend_from_slice(&len_be);
    // data
    out.extend_from_slice(&data_bytes);
    out
}

pub fn create_safe_multisend_transaction(
    txns: &[SafeTransaction],
    safe_multisend_address: &str,
) -> SafeTransaction {
    if txns.len() == 1 {
        return txns[0].clone();
    }
    let mut packed: Vec<u8> = Vec::new();
    for t in txns {
        packed.extend_from_slice(&pack_tx(t));
    }
    // encode function selector + encoded bytes via ethers abi
    // multiSend(bytes)
    let calldata = encode(&[Token::Bytes(packed)]);
    let selector = &keccak256(b"multiSend(bytes)")[..4];
    let mut final_data = Vec::from(selector);
    final_data.extend_from_slice(&calldata);
    SafeTransaction {
        to: safe_multisend_address.to_string(),
        value: "0".to_string(),
        data: format!("0x{}", final_data.encode_hex::<String>()),
        operation: OperationType::DelegateCall,
    }
}

fn keccak256(data: &[u8]) -> [u8; 32] {
    use sha3::{Digest, Keccak256};
    let mut h = Keccak256::new();
    h.update(data);
    let out = h.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&out);
    arr
}
