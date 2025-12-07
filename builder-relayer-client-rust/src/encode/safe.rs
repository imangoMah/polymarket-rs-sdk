use crate::types::{OperationType, SafeTransaction};
use alloy::primitives::{Address, U256};
use alloy::sol;
use alloy::sol_types::SolCall;

// pack single transaction like Solidity struct used by Gnosis Safe MultiSend
fn pack_tx(tx: &SafeTransaction) -> Vec<u8> {
    // bytes: operation (1) + to (20) + value (32) + data length (32) + data
    let mut out = Vec::new();
    out.push(tx.operation as u8);

    // address
    let addr: Address = tx.to.parse().unwrap_or(Address::ZERO);
    out.extend_from_slice(addr.as_slice());

    // value (uint256)
    let value = U256::from_str_radix(&tx.value, 10).unwrap_or(U256::ZERO);
    out.extend_from_slice(&value.to_be_bytes::<32>());

    // data length (uint256)
    let data_bytes = hex::decode(tx.data.trim_start_matches("0x")).unwrap_or_default();
    let len = U256::from(data_bytes.len());
    out.extend_from_slice(&len.to_be_bytes::<32>());

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

    // encode function selector + encoded bytes via abi
    // multiSend(bytes)
    sol! {
        function multiSend(bytes memory transactions);
    }

    let call = multiSendCall {
        transactions: packed.into(),
    };
    let calldata = call.abi_encode();

    SafeTransaction {
        to: safe_multisend_address.to_string(),
        value: "0".to_string(),
        data: format!("0x{}", hex::encode(calldata)),
        operation: OperationType::DelegateCall,
    }
}
