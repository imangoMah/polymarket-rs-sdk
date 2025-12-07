use crate::errors::ClobError;
use crate::signing::Eip712Signer;
use alloy::primitives::{Address, U256};
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::Signer;
use alloy::sol;
use alloy::sol_types::{Eip712Domain, SolStruct};
use async_trait::async_trait;
use std::str::FromStr;
use std::sync::Arc;

// Define Sol structs matching the JSON structure
sol! {
    #[derive(serde::Deserialize, Debug)]
    struct ClobAuth {
        address address;
        string timestamp;
        uint256 nonce;
        string message;
    }

    #[derive(serde::Deserialize, Debug)]
    struct Order {
        uint256 salt;
        address maker;
        address signer;
        address taker;
        uint256 tokenId;
        uint256 makerAmount;
        uint256 takerAmount;
        uint256 expiration;
        uint256 nonce;
        uint256 feeRateBps;
        uint8 side;
        uint8 signatureType;
    }
}

/// A simple alloy-based signer adapter that implements Eip712Signer.
pub struct EthersSigner {
    wallet: Arc<PrivateKeySigner>,
}

impl EthersSigner {
    /// Create a new EthersSigner from a hex private key string (with or without 0x).
    pub fn new_from_private_key(hex_priv: &str) -> Result<Self, ClobError> {
        let w = PrivateKeySigner::from_str(hex_priv)
            .map_err(|e| ClobError::Other(format!("invalid private key: {}", e)))?;
        Ok(Self {
            wallet: Arc::new(w),
        })
    }

    /// Return inner wallet (cloneable Arc) for advanced usage
    pub fn wallet(&self) -> Arc<PrivateKeySigner> {
        self.wallet.clone()
    }
}

#[async_trait]
impl Eip712Signer for EthersSigner {
    async fn get_address(&self) -> Result<String, ClobError> {
        // Produce a full hex address explicitly (0x-prefixed)
        Ok(format!("{:#x}", self.wallet.address()))
    }

    async fn sign_typed_data(
        &self,
        domain: &str,
        types: &str,
        value: &str,
    ) -> Result<String, ClobError> {
        // Parse domain JSON
        let domain_val: serde_json::Value = serde_json::from_str(domain)
            .map_err(|e| ClobError::Other(format!("invalid domain json: {}", e)))?;
        
        let name = domain_val.get("name").and_then(|v| v.as_str()).map(|s| s.to_string().into());
        let version = domain_val.get("version").and_then(|v| v.as_str()).map(|s| s.to_string().into());
        let chain_id = domain_val.get("chainId").and_then(|v| v.as_u64()).map(|v| U256::from(v));
        let verifying_contract = domain_val.get("verifyingContract")
            .and_then(|v| v.as_str())
            .map(|s| Address::from_str(s).unwrap_or_default());

        let domain_struct = Eip712Domain {
            name,
            version,
            chain_id,
            verifying_contract,
            salt: None,
        };

        // Determine primary type from types JSON
        let types_val: serde_json::Value = serde_json::from_str(types)
            .map_err(|e| ClobError::Other(format!("invalid types json: {}", e)))?;
        
        let primary_type = if let Some(map) = types_val.as_object() {
            if map.contains_key("ClobAuth") {
                "ClobAuth"
            } else if map.contains_key("Order") {
                "Order"
            } else {
                "Order"
            }
        } else {
            "Order"
        };

        let sig = match primary_type {
            "ClobAuth" => {
                let auth: ClobAuth = serde_json::from_str(value)
                    .map_err(|e| ClobError::Other(format!("failed to parse ClobAuth: {}", e)))?;
                let hash = auth.eip712_signing_hash(&domain_struct);
                self.wallet.sign_hash(&hash).await
            },
            "Order" => {
                let order: Order = serde_json::from_str(value)
                    .map_err(|e| ClobError::Other(format!("failed to parse Order: {}", e)))?;
                let hash = order.eip712_signing_hash(&domain_struct);
                self.wallet.sign_hash(&hash).await
            },
            _ => return Err(ClobError::Other(format!("unknown primary type: {}", primary_type))),
        };

        match sig {
            Ok(s) => {
                // Alloy signature as_bytes() returns 65 bytes [r, s, v]
                // Check v. If it's 0/1, convert to 27/28 for compatibility if needed.
                // Ethers usually produces 27/28.
                let mut bytes = s.as_bytes();
                if bytes[64] < 27 {
                    bytes[64] += 27;
                }
                Ok(hex::encode(bytes))
            },
            Err(e) => Err(ClobError::Other(format!("sign error: {}", e))),
        }
    }
}
