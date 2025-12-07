use crate::builder::create::{
    AbstractSignerForCreate, CreateProxy, EIP712Domain, SAFE_FACTORY_NAME,
};
use crate::errors::{RelayClientError, Result};
use alloy::primitives::{keccak256, Address, Signature, B256, U256};
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::Signer;
use alloy::sol_types::SolStruct;
use async_trait::async_trait;
use std::str::FromStr;

/// Trait mirroring the expected abstract signer behavior in TS version.
#[async_trait]
pub trait AbstractSigner: Send + Sync {
    fn address(&self) -> Address;
    async fn sign_hash(&self, hash: B256) -> Result<Signature>;
    async fn sign_eip712_digest(&self, digest: B256) -> Result<Signature>;
}

/// DummySigner: simple local wallet wrapper from a provided private key.
#[derive(Clone)]
pub struct DummySigner {
    wallet: PrivateKeySigner,
}

impl DummySigner {
    pub fn new(priv_key_hex: &str) -> Result<Self> {
        let wallet = PrivateKeySigner::from_str(priv_key_hex)
            .map_err(|_| RelayClientError::SignerUnavailable)?;
        Ok(Self { wallet })
    }
}

#[async_trait]
impl AbstractSigner for DummySigner {
    fn address(&self) -> Address {
        self.wallet.address()
    }

    async fn sign_hash(&self, hash: B256) -> Result<Signature> {
        self.wallet
            .sign_hash(&hash)
            .await
            .map_err(|e| RelayClientError::SigningError(e.to_string()))
    }

    async fn sign_eip712_digest(&self, digest: B256) -> Result<Signature> {
        // For EIP-712 digest, we just sign the hash.
        // The digest construction happens outside.
        self.sign_hash(digest).await
    }
}

#[async_trait]
impl AbstractSignerForCreate for DummySigner {
    fn address(&self) -> Address {
        self.wallet.address()
    }

    async fn sign_typed_create_proxy(
        &self,
        safe_factory: &str,
        chain_id: u64,
        payment_token: &str,
        payment: &str,
        payment_receiver: &str,
    ) -> Result<String> {
        let domain = EIP712Domain {
            name: SAFE_FACTORY_NAME.to_string(),
            chainId: U256::from(chain_id),
            verifyingContract: Address::from_str(safe_factory)
                .map_err(|_| RelayClientError::InvalidAddress)?,
        };

        let create_proxy = CreateProxy {
            paymentToken: Address::from_str(payment_token)
                .map_err(|_| RelayClientError::InvalidAddress)?,
            payment: U256::from_str(payment).unwrap_or(U256::ZERO),
            paymentReceiver: Address::from_str(payment_receiver)
                .map_err(|_| RelayClientError::InvalidAddress)?,
        };

        let domain_separator = domain.eip712_hash_struct();
        let struct_hash = create_proxy.eip712_hash_struct();

        let mut digest_input = Vec::with_capacity(2 + 32 + 32);
        digest_input.extend_from_slice(&[0x19, 0x01]);
        digest_input.extend_from_slice(&domain_separator.0);
        digest_input.extend_from_slice(&struct_hash.0);

        let digest = keccak256(&digest_input);

        let sig = self.sign_hash(digest).await?;
        Ok(format!("0x{}", hex::encode(sig.as_bytes())))
    }
}
