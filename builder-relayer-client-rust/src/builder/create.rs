use crate::builder::derive::derive_safe;
use crate::errors::Result;
use crate::types::{
    SafeCreateTransactionArgs, SignatureParams, TransactionRequest, TransactionType,
};
use crate::utils::split_and_pack_sig;
use ethers::abi::{encode, Token};
use ethers::types::{Address, U256};
use sha3::{Digest, Keccak256};

// Gnosis Safe CREATE typed data (standard for createProxyWithNonce)
// Based on GnosisSafeProxyFactory.sol signature
const SAFE_CREATE_TYPE_STR: &str = "SafeCreate(address owner,address paymentToken,uint256 payment,address paymentReceiver,uint256 nonce)";
const DOMAIN_TYPE_STR: &str = "EIP712Domain(uint256 chainId,address verifyingContract)";

fn keccak_bytes(data: &[u8]) -> [u8; 32] {
    let mut h = Keccak256::new();
    h.update(data);
    let out = h.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&out);
    arr
}

fn safe_create_type_hash() -> [u8; 32] {
    keccak_bytes(SAFE_CREATE_TYPE_STR.as_bytes())
}
fn domain_type_hash() -> [u8; 32] {
    keccak_bytes(DOMAIN_TYPE_STR.as_bytes())
}

fn eip712_domain_separator(chain_id: U256, verifying_contract: Address) -> [u8; 32] {
    let encoded = encode(&[
        Token::FixedBytes(domain_type_hash().to_vec()),
        Token::Uint(chain_id),
        Token::Address(verifying_contract),
    ]);
    keccak_bytes(&encoded)
}

fn safe_create_struct_hash(
    owner: Address,
    payment_token: Address,
    payment: U256,
    payment_receiver: Address,
    nonce: U256,
) -> [u8; 32] {
    let encoded = encode(&[
        Token::FixedBytes(safe_create_type_hash().to_vec()),
        Token::Address(owner),
        Token::Address(payment_token),
        Token::Uint(payment),
        Token::Address(payment_receiver),
        Token::Uint(nonce),
    ]);
    keccak_bytes(&encoded)
}

pub trait TypedDataSigner: Send + Sync {
    fn sign_typed_create_proxy(
        &self,
        safe_factory: &str,
        chain_id: u64,
        payment_token: &str,
        payment: &str,
        payment_receiver: &str,
    ) -> Result<String>;
}

pub trait AbstractSignerForCreate: Send + Sync {
    fn get_address(&self) -> Result<String>;
    fn sign_eip712_digest(&self, digest_hex: &str) -> Result<String>;
}

pub async fn build_safe_create_transaction_request(
    signer: &dyn AbstractSignerForCreate,
    safe_factory: &str,
    args: SafeCreateTransactionArgs,
) -> Result<TransactionRequest> {
    let owner: Address = args
        .from
        .parse()
        .map_err(|_| crate::errors::RelayClientError::SignerUnavailable)?;
    let factory: Address = safe_factory
        .parse()
        .map_err(|_| crate::errors::RelayClientError::SignerUnavailable)?;
    let payment_token: Address = args
        .payment_token
        .parse()
        .map_err(|_| crate::errors::RelayClientError::SignerUnavailable)?;
    let payment_receiver: Address = args
        .payment_receiver
        .parse()
        .map_err(|_| crate::errors::RelayClientError::SignerUnavailable)?;
    let payment = U256::from_dec_str(&args.payment).unwrap_or_default();

    // For SAFE CREATE, nonce is typically 0 or derived from keccak(owner)
    // Using 0 for simplicity (matches common patterns)
    let nonce = U256::zero();

    let struct_hash =
        safe_create_struct_hash(owner, payment_token, payment, payment_receiver, nonce);
    let domain_separator = eip712_domain_separator(U256::from(args.chain_id), factory);

    let mut prefix = vec![0x19, 0x01];
    prefix.extend_from_slice(&domain_separator);
    prefix.extend_from_slice(&struct_hash);
    let digest = keccak_bytes(&prefix);

    let sig = signer.sign_eip712_digest(&format!("0x{}", hex::encode(digest)))?;
    let packed_sig = split_and_pack_sig(&sig);

    let sig_params = SignatureParams {
        payment_token: Some(args.payment_token.clone()),
        payment: Some(args.payment.clone()),
        payment_receiver: Some(args.payment_receiver.clone()),
        ..Default::default()
    };

    let safe_address = derive_safe(&args.from, safe_factory);

    let req = TransactionRequest {
        from: args.from.clone(),
        to: safe_factory.to_string(),
        proxy_wallet: Some(safe_address),
        data: "0x".to_string(),
        nonce: None,
        signature: packed_sig,
        signature_params: sig_params,
        r#type: TransactionType::SafeCreate,
        metadata: None,
    };
    Ok(req)
}
