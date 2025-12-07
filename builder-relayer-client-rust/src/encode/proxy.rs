use crate::types::ProxyTransaction;
use alloy::primitives::{Address, U256};
use alloy::sol;
use alloy::sol_types::SolCall;
use std::str::FromStr;

// ABI encode proxy(ProxyCall[]) for Proxy Wallet Factory
sol! {
    #[derive(Debug)]
    contract ProxyFactory {
        struct ProxyCall {
            uint8 typeCode;
            address to;
            uint256 value;
            bytes data;
        }
        function proxy(ProxyCall[] calls) returns (bytes[] returnValues);
    }
}

pub fn encode_proxy_transaction_data(txns: &[ProxyTransaction]) -> String {
    let calls: Vec<ProxyFactory::ProxyCall> = txns
        .iter()
        .map(|t| ProxyFactory::ProxyCall {
            typeCode: t.type_code as u8,
            to: Address::from_str(&t.to).unwrap_or(Address::ZERO),
            value: U256::from_str_radix(&t.value, 10).unwrap_or(U256::ZERO),
            data: hex::decode(t.data.trim_start_matches("0x"))
                .unwrap_or_default()
                .into(),
        })
        .collect();

    let func_call = ProxyFactory::proxyCall { calls };
    let encoded = func_call.abi_encode();
    format!("0x{}", hex::encode(encoded))
}
