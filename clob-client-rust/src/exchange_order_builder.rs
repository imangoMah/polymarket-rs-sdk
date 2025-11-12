use crate::errors::ClobError;
use crate::exchange_consts::{PROTOCOL_NAME, PROTOCOL_VERSION};
use crate::signing::Eip712Signer;
use crate::types::{OrderData, SignedOrder};
use ethers::core::utils::keccak256;
// rand::random used below; no Rng trait required
use serde_json::{Value, json};

pub fn generate_order_salt() -> String {
    // JavaScript 安全整数上限是 2^53 - 1 (约 9×10^15)
    // 为确保与 TS SDK 兼容且 JSON 序列化为 number 不丢失精度，限制在此范围内
    const MAX_SAFE_INT: u64 = 9007199254740991; // 2^53 - 1
    let v: u64 = rand::random::<u64>() % (MAX_SAFE_INT + 1);
    v.to_string()
}

pub struct ExchangeOrderBuilder<'a, S: Eip712Signer> {
    contract_address: &'a str,
    chain_id: i64,
    signer: &'a S,
}

impl<'a, S: Eip712Signer> ExchangeOrderBuilder<'a, S> {
    pub fn new(contract_address: &'a str, chain_id: i64, signer: &'a S) -> Self {
        Self {
            contract_address,
            chain_id,
            signer,
        }
    }

    pub async fn build_signed_order(
        &self,
        order_data: OrderData,
    ) -> Result<SignedOrder, ClobError> {
        let mut order = self.build_order(order_data)?;
        let order_typed = self.build_order_typed_data(&order);
        // 调试: 输出 typed-data 关键域（仅在设置 CLOB_DEBUG_FULL 或 CLOB_DEBUG_TYPED 时）
        if std::env::var("CLOB_DEBUG_FULL").is_ok() || std::env::var("CLOB_DEBUG_TYPED").is_ok() {
            let domain = order_typed
                .get("domain")
                .cloned()
                .unwrap_or(serde_json::Value::Null);
            let message = order_typed
                .get("message")
                .cloned()
                .unwrap_or(serde_json::Value::Null);
            let hash_preview = self.build_order_hash(&order_typed);
            eprintln!(
                "[EIP712 DEBUG] domain={} message={} hash_preview={}",
                domain, message, hash_preview
            );
        }
        let sig = self.build_order_signature(&order_typed).await?;
        // 保证签名以 0x 前缀统一（TS ethers 默认带 0x）
        order.signature = if sig.starts_with("0x") {
            sig
        } else {
            format!("0x{}", sig)
        };
        Ok(order)
    }

    /// Deterministic variant allowing caller to force a specific salt (for cross-language parity tests).
    pub async fn build_signed_order_with_salt(
        &self,
        order_data: OrderData,
        forced_salt: &str,
    ) -> Result<SignedOrder, ClobError> {
        // Build order normally then override salt before signing.
        let mut order = self.build_order(order_data.clone())?;
        order.salt = forced_salt.to_string();
        let order_typed = self.build_order_typed_data(&order);
        if std::env::var("CLOB_DEBUG_FULL").is_ok() || std::env::var("CLOB_DEBUG_TYPED").is_ok() {
            let domain = order_typed
                .get("domain")
                .cloned()
                .unwrap_or(serde_json::Value::Null);
            let message = order_typed
                .get("message")
                .cloned()
                .unwrap_or(serde_json::Value::Null);
            let hash_preview = self.build_order_hash(&order_typed);
            eprintln!(
                "[EIP712 DEBUG] (forced_salt) domain={} message={} hash_preview={}",
                domain, message, hash_preview
            );
        }
        let sig = self.build_order_signature(&order_typed).await?;
        order.signature = if sig.starts_with("0x") {
            sig
        } else {
            format!("0x{}", sig)
        };
        Ok(order)
    }

    pub fn build_order(&self, mut data: OrderData) -> Result<SignedOrder, ClobError> {
        if data.signer.is_empty() {
            data.signer = data.maker.clone();
        }
        // Validate signer equals signer address from signer implementation would require async; skip here.
        if data.expiration.is_empty() {
            data.expiration = "0".to_string();
        }
        // signatureType handling kept as-is
        let salt = generate_order_salt();
        let order = SignedOrder {
            salt,
            maker: data.maker.clone(),
            signer: data.signer.clone(),
            taker: data.taker.clone(),
            token_id: data.token_id.clone(),
            maker_amount: data.maker_amount.clone(),
            taker_amount: data.taker_amount.clone(),
            expiration: data.expiration.clone(),
            nonce: data.nonce.clone(),
            fee_rate_bps: data.fee_rate_bps.clone(),
            side: data.side.clone(),
            signature_type: data.signature_type.clone(),
            signature: String::new(),
        };
        Ok(order)
    }

    pub fn build_order_typed_data(&self, order: &SignedOrder) -> Value {
        let domain = json!({
            "name": PROTOCOL_NAME,
            "version": PROTOCOL_VERSION,
            "chainId": self.chain_id,
            "verifyingContract": self.contract_address,
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
            "side": match order.side { crate::types::Side::BUY => 0u8, crate::types::Side::SELL => 1u8 },
            "signatureType": u8::from(order.signature_type)
        });

        json!({
            "primaryType": "Order",
            "types": types,
            "domain": domain,
            "message": message,
        })
    }

    pub async fn build_order_signature(&self, typed: &Value) -> Result<String, ClobError> {
        // Remove EIP712Domain from types as in TypeScript implementation
        // For simplicity, serialize domain/types/message and ask signer to sign
        let domain = typed.get("domain").cloned().unwrap_or(Value::Null);
        let types = typed.get("types").cloned().unwrap_or(Value::Null);
        let message = typed.get("message").cloned().unwrap_or(Value::Null);

        let domain_str =
            serde_json::to_string(&domain).map_err(|e| ClobError::Other(e.to_string()))?;
        let types_str =
            serde_json::to_string(&types).map_err(|e| ClobError::Other(e.to_string()))?;
        let message_str =
            serde_json::to_string(&message).map_err(|e| ClobError::Other(e.to_string()))?;

        self.signer
            .sign_typed_data(&domain_str, &types_str, &message_str)
            .await
    }

    pub fn build_order_hash(&self, typed: &Value) -> String {
        let ser = serde_json::to_vec(typed).unwrap_or_default();
        let hash = keccak256(&ser);
        format!("0x{}", hex::encode(hash))
    }
}
