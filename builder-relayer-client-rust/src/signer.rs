use ethers::signers::{LocalWallet, Signer};
use ethers::types::{Signature, H256};
use sha3::{Digest, Keccak256};

use crate as builder_relayer_client_rust;
use crate::errors::{RelayClientError, Result};

/// Trait mirroring the expected abstract signer behavior in TS version.
pub trait AbstractSigner {
    fn address(&self) -> ethers::types::Address;
    fn sign_hash(&self, hash: H256) -> std::result::Result<Signature, RelayClientError>;
    fn sign_eip712_digest(&self, digest: H256) -> std::result::Result<Signature, RelayClientError>;
}

/// Trait for EIP-712 typed data signing (create proxy specific typed data). Placeholder for future struct generation.
pub trait TypedDataSigner {
    fn sign_typed_create_proxy(
        &self,
        safe_factory: &str,
        chain_id: u64,
        payment_token: &str,
        payment: &str,
        payment_receiver: &str,
    ) -> std::result::Result<String, RelayClientError>;
}

/// DummySigner: simple local wallet wrapper from a provided private key (hex, no 0x needed or with 0x).
#[derive(Clone)]
pub struct DummySigner {
    wallet: LocalWallet,
}

impl DummySigner {
    pub fn new(priv_key_hex: &str) -> std::result::Result<Self, RelayClientError> {
        let prefixed = if priv_key_hex.starts_with("0x") {
            priv_key_hex.to_string()
        } else {
            format!("0x{}", priv_key_hex)
        };
        let wallet: LocalWallet = prefixed
            .parse()
            .map_err(|_| RelayClientError::SignerUnavailable)?;
        Ok(Self { wallet })
    }
}

impl AbstractSigner for DummySigner {
    fn address(&self) -> ethers::types::Address {
        self.wallet.address()
    }
    fn sign_hash(&self, hash: H256) -> std::result::Result<Signature, RelayClientError> {
        let sig = self
            .wallet
            .sign_hash(hash)
            .map_err(|_| RelayClientError::SignerUnavailable)?;
        Ok(sig)
    }
    fn sign_eip712_digest(&self, digest: H256) -> std::result::Result<Signature, RelayClientError> {
        // For RelayClient (builder/relayer), sign the EIP-712 digest directly
        // WITHOUT adding Ethereum message prefix
        // This is different from on-chain Safe execution which uses signMessage
        use ethers::core::k256::ecdsa::SigningKey;

        let key_bytes = &self.wallet.signer().to_bytes();
        let signing_key =
            SigningKey::from_slice(key_bytes).map_err(|_| RelayClientError::SignerUnavailable)?;
        let (sig, recovery_id) = signing_key
            .sign_prehash_recoverable(digest.as_bytes())
            .map_err(|_| RelayClientError::SignerUnavailable)?;

        let r_bytes = sig.r().to_bytes();
        let s_bytes = sig.s().to_bytes();
        Ok(Signature {
            r: ethers::types::U256::from_big_endian(r_bytes.as_ref()),
            s: ethers::types::U256::from_big_endian(s_bytes.as_ref()),
            v: recovery_id.to_byte() as u64,
        })
    }
}

impl TypedDataSigner for DummySigner {
    fn sign_typed_create_proxy(
        &self,
        _safe_factory: &str,
        _chain_id: u64,
        _payment_token: &str,
        _payment: &str,
        _payment_receiver: &str,
    ) -> std::result::Result<String, RelayClientError> {
        // Placeholder: sign zero hash for now; to be replaced with full typed data encoding.
        let zero = H256::zero();
        let sig = self.sign_hash(zero)?;
        Ok(sig_to_hex(sig))
    }
}

// Implement the builder module traits for convenience so examples can pass DummySigner directly
impl builder_relayer_client_rust::builder::safe::AbstractSigner for DummySigner {
    fn get_address(&self) -> Result<String> {
        Ok(format!("0x{:x}", self.address()))
    }
    fn sign_message(&self, hash32_hex: &str) -> Result<String> {
        // Sign using Ethereum Signed Message prefix (EIP-191) over the 32-byte input
        let bytes = hex::decode(hash32_hex.trim_start_matches("0x"))
            .map_err(|_| RelayClientError::SignerUnavailable)?;
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        // Compute keccak("\x19Ethereum Signed Message:\n32" || hash32)
        let mut msg = b"\x19Ethereum Signed Message:\n32".to_vec();
        msg.extend_from_slice(&arr);
        let mut hasher = Keccak256::new();
        hasher.update(&msg);
        let digest = hasher.finalize();
        let mut dig = [0u8; 32];
        dig.copy_from_slice(&digest);
        let sig = self
            .sign_hash(H256::from(dig))
            .map_err(|_| RelayClientError::SignerUnavailable)?;
        Ok(sig_to_hex(sig))
    }
    fn sign_eip712_digest(&self, digest_hex: &str) -> Result<String> {
        // For EIP-712 digest, use the trait method that signs without prefix
        let bytes = hex::decode(digest_hex.trim_start_matches("0x"))
            .map_err(|_| RelayClientError::SignerUnavailable)?;
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        let sig = AbstractSigner::sign_eip712_digest(self, H256::from(arr))
            .map_err(|_| RelayClientError::SignerUnavailable)?;
        Ok(sig_to_hex(sig))
    }
}

impl builder_relayer_client_rust::builder::create::TypedDataSigner for DummySigner {
    fn sign_typed_create_proxy(
        &self,
        _safe_factory: &str,
        _chain_id: u64,
        _payment_token: &str,
        _payment: &str,
        _payment_receiver: &str,
    ) -> Result<String> {
        // Placeholder same as our local trait: sign zero hash
        let sig = self
            .sign_hash(H256::zero())
            .map_err(|_| RelayClientError::SignerUnavailable)?;
        Ok(sig_to_hex(sig))
    }
}

impl builder_relayer_client_rust::builder::create::AbstractSignerForCreate for DummySigner {
    fn get_address(&self) -> Result<String> {
        Ok(format!("0x{:x}", self.address()))
    }
    fn sign_eip712_digest(&self, digest_hex: &str) -> Result<String> {
        // For EIP-712 digest, use the trait method that signs without prefix
        let bytes = hex::decode(digest_hex.trim_start_matches("0x"))
            .map_err(|_| RelayClientError::SignerUnavailable)?;
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        let sig = AbstractSigner::sign_eip712_digest(self, H256::from(arr))
            .map_err(|_| RelayClientError::SignerUnavailable)?;
        Ok(sig_to_hex(sig))
    }
}

fn sig_to_hex(sig: Signature) -> String {
    // r,s are U256; convert to 32-byte big endian
    let mut r_bytes = [0u8; 32];
    sig.r.to_big_endian(&mut r_bytes);
    let mut s_bytes = [0u8; 32];
    sig.s.to_big_endian(&mut s_bytes);
    format!(
        "0x{}{}{:02x}",
        hex::encode(r_bytes),
        hex::encode(s_bytes),
        sig.v
    )
}
