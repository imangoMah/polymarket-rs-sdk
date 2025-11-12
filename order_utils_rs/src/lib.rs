use serde::{Deserialize, Serialize};
use serde_json::json;

pub const PROTOCOL_NAME: &str = "Polymarket CTF Exchange";
pub const PROTOCOL_VERSION: &str = "1";
pub const ZX: &str = "0x";
pub const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";
pub const CALL_RESULTS_PREFIX: &str = "CALL_RESULTS_";

pub fn generate_order_salt() -> String {
    let v: u128 = rand::random::<u128>();
    v.to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub salt: String,
    pub maker: String,
    pub signer: String,
    pub taker: String,
    pub token_id: String,
    pub maker_amount: String,
    pub taker_amount: String,
    pub expiration: String,
    pub nonce: String,
    pub fee_rate_bps: String,
    pub side: u8,
    pub signature_type: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderData {
    pub maker: String,
    pub taker: String,
    pub token_id: String,
    pub maker_amount: String,
    pub taker_amount: String,
    pub side: u8,
    pub fee_rate_bps: String,
    pub nonce: String,
    pub signer: String,
    pub expiration: String,
    pub signature_type: u8,
}

pub fn build_order_typed_data(
    contract_address: &str,
    chain_id: i64,
    order: &Order,
) -> serde_json::Value {
    let domain = json!({
        "name": PROTOCOL_NAME,
        "version": PROTOCOL_VERSION,
        "chainId": chain_id,
        "verifyingContract": contract_address,
    });
    let types = json!({
        "EIP712Domain": [
            {"name":"name","type":"string"},
            {"name":"version","type":"string"},
            {"name":"chainId","type":"uint256"},
            {"name":"verifyingContract","type":"address"}
        ],
        "Order": [
            {"name":"salt","type":"uint256"},
            {"name":"maker","type":"address"},
            {"name":"signer","type":"address"},
            {"name":"taker","type":"address"},
            {"name":"tokenId","type":"uint256"},
            {"name":"makerAmount","type":"uint256"},
            {"name":"takerAmount","type":"uint256"},
            {"name":"expiration","type":"uint256"},
            {"name":"nonce","type":"uint256"},
            {"name":"feeRateBps","type":"uint256"},
            {"name":"side","type":"uint8"},
            {"name":"signatureType","type":"uint8"}
        ]
    });
    let message = json!({
        "salt": order.salt,
        "maker": order.maker,
        "signer": order.signer,
        "taker": order.taker,
        "tokenId": order.token_id,
        "makerAmount": order.maker_amount,
        "takerAmount": order.taker_amount,
        "expiration": order.expiration,
        "nonce": order.nonce,
        "feeRateBps": order.fee_rate_bps,
        "side": order.side,
        "signatureType": order.signature_type,
    });
    json!({
        "primaryType": "Order",
        "types": types,
        "domain": domain,
        "message": message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn salt_not_empty() {
        assert!(!generate_order_salt().is_empty());
    }
}
