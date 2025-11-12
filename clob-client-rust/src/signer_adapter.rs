use crate::errors::ClobError;
use crate::signing::Eip712Signer;
use async_trait::async_trait;
use ethers::core::types::transaction::eip712::TypedData;
use ethers::signers::{LocalWallet, Signer};
use std::str::FromStr;
use std::sync::Arc;

/// A simple ethers-rs based signer adapter that implements Eip712Signer.
///
/// NOTE: For full EIP-712 compliance you may want to hook into ethers' typed-data
/// signing APIs; this adapter uses keccak256(domain+types+value) as the digest
/// and signs that digest. It's a pragmatic adapter to integrate with the
/// existing `ExchangeOrderBuilder::build_order_signature` which sends JSON strings.
pub struct EthersSigner {
    wallet: Arc<LocalWallet>,
}

impl EthersSigner {
    /// Create a new EthersSigner from a hex private key string (with or without 0x).
    pub fn new_from_private_key(hex_priv: &str) -> Result<Self, ClobError> {
        let w = LocalWallet::from_str(hex_priv)
            .map_err(|e| ClobError::Other(format!("invalid private key: {}", e)))?;
        Ok(Self {
            wallet: Arc::new(w),
        })
    }

    /// Return inner wallet (cloneable Arc) for advanced usage
    pub fn wallet(&self) -> Arc<LocalWallet> {
        self.wallet.clone()
    }
}

#[async_trait]
impl Eip712Signer for EthersSigner {
    async fn get_address(&self) -> Result<String, ClobError> {
        // Produce a full hex address explicitly (0x-prefixed) to avoid any trimmed/debug representation
        Ok(format!("{:#x}", self.wallet.address()))
    }

    async fn sign_typed_data(
        &self,
        domain: &str,
        types: &str,
        value: &str,
    ) -> Result<String, ClobError> {
        // Try to construct a TypedData from the provided domain/types/value JSON strings.
        // The ExchangeOrderBuilder provides domain, types, and message separately. We'll assemble
        // a full TypedData JSON and attempt to deserialize it into ethers' TypedData, then
        // call the Signer::sign_typed_data API which returns an EIP-712 compliant signature.

        // Build a full typed-data JSON object: { "types": ..., "domain": ..., "primaryType": "Order", "message": ... }
        let types_val: serde_json::Value = serde_json::from_str(types)
            .map_err(|e| ClobError::Other(format!("invalid types json: {}", e)))?;
        let domain_val: serde_json::Value = serde_json::from_str(domain)
            .map_err(|e| ClobError::Other(format!("invalid domain json: {}", e)))?;
        let message_val: serde_json::Value = serde_json::from_str(value)
            .map_err(|e| ClobError::Other(format!("invalid message json: {}", e)))?;

        // Determine primaryType from provided types (prefer ClobAuth)
        let types_obj = types_val.clone();
        let primary_type = if let Some(map) = types_obj.as_object() {
            if map.contains_key("ClobAuth") {
                "ClobAuth".to_string()
            } else if map.contains_key("Order") {
                "Order".to_string()
            } else {
                map.keys()
                    .next()
                    .cloned()
                    .unwrap_or_else(|| "Order".to_string())
            }
        } else {
            "Order".to_string()
        };

        // IMPORTANT: Do NOT inject EIP712Domain into types here - keep types exactly as the caller passed
        // (TypeScript SDK sends types without an explicit EIP712Domain key). ethers-rs TypedData will accept
        // a domain object separately.
        let full = serde_json::json!({
            "types": types_obj,
            "domain": domain_val,
            "primaryType": primary_type,
            "message": message_val
        });

        // Debug output to compare with TypeScript inputs when requested
        if std::env::var("CLOB_DEBUG_FULL").is_ok() || std::env::var("CLOB_DEBUG_RAW").is_ok() {
            let pretty = serde_json::to_string_pretty(&full)
                .unwrap_or_else(|_| "<failed to pretty print>".to_string());
            eprintln!("[EIP712 DEBUG] full typed-data JSON: {}", pretty);
        }

        let full_str = serde_json::to_string(&full).map_err(|e| ClobError::Other(e.to_string()))?;

        // Deserialize into TypedData with improved error logging
        let typed_res: Result<TypedData, _> = serde_json::from_str(&full_str);
        let typed: TypedData = match typed_res {
            Ok(t) => t,
            Err(e) => {
                eprintln!(
                    "[EIP712 ERROR] Failed to parse TypedData. full_json={}",
                    full_str
                );
                return Err(ClobError::Other(format!(
                    "failed to parse TypedData: {}",
                    e
                )));
            }
        };

        // Use ethers Signer API to sign typed data (async)
        let sig = match self.wallet.sign_typed_data(&typed).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[EIP712 ERROR] typed-data sign error: {}", e);
                return Err(ClobError::Other(format!("typed-data sign error: {}", e)));
            }
        };

        if std::env::var("CLOB_DEBUG_FULL").is_ok() || std::env::var("CLOB_DEBUG_RAW").is_ok() {
            eprintln!("[EIP712 DEBUG] produced signature: {}", sig);
        }

        Ok(sig.to_string())
    }
}

// Implement headers::Signer for compatibility with header helpers
// NOTE: headers::Signer trait removed in favor of using Eip712Signer directly.
