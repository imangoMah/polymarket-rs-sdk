use crate::builder::derive::derive_safe;
use crate::encode::safe::create_safe_multisend_transaction;
use crate::errors::Result;
use crate::types::{
    SafeTransaction, SafeTransactionArgs, SignatureParams, TransactionRequest, TransactionType,
};
use crate::utils::split_and_pack_sig;
use ethers::abi::{encode, Token};
use ethers::types::{Address, U256};
use sha3::{Digest, Keccak256};

#[derive(Clone, Debug)]
pub struct SafeContractConfig {
    pub safe_factory: String,
    pub safe_multisend: String,
}

// Type strings (align with TS implementation & Gnosis Safe spec)
const SAFE_TX_TYPE_STR: &str = "SafeTx(address to,uint256 value,bytes data,uint8 operation,uint256 safeTxGas,uint256 baseGas,uint256 gasPrice,address gasToken,address refundReceiver,uint256 nonce)";
const DOMAIN_TYPE_STR: &str = "EIP712Domain(uint256 chainId,address verifyingContract)";

fn keccak_bytes(data: &[u8]) -> [u8; 32] {
    let mut h = Keccak256::new();
    h.update(data);
    let out = h.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&out);
    arr
}

fn safe_tx_type_hash() -> [u8; 32] {
    keccak_bytes(SAFE_TX_TYPE_STR.as_bytes())
}
fn domain_type_hash() -> [u8; 32] {
    keccak_bytes(DOMAIN_TYPE_STR.as_bytes())
}

fn keccak(data: &[u8]) -> [u8; 32] {
    keccak_bytes(data)
}

pub fn eip712_domain_separator(chain_id: U256, verifying_contract: Address) -> [u8; 32] {
    // abi.encode(typeHash, chainId, verifyingContract)
    let encoded = encode(&[
        Token::FixedBytes(domain_type_hash().to_vec()),
        Token::Uint(chain_id),
        Token::Address(verifying_contract),
    ]);
    keccak(&encoded)
}

pub fn safe_tx_struct_hash(
    to: Address,
    value: U256,
    data: &[u8],
    operation: u8,
    safe_tx_gas: U256,
    base_gas: U256,
    gas_price: U256,
    gas_token: Address,
    refund_receiver: Address,
    nonce: U256,
) -> [u8; 32] {
    let data_hash = keccak(data);
    let encoded = encode(&[
        Token::FixedBytes(safe_tx_type_hash().to_vec()),
        Token::Address(to),
        Token::Uint(value),
        Token::FixedBytes(data_hash.to_vec()),
        Token::Uint(U256::from(operation)),
        Token::Uint(safe_tx_gas),
        Token::Uint(base_gas),
        Token::Uint(gas_price),
        Token::Address(gas_token),
        Token::Address(refund_receiver),
        Token::Uint(nonce),
    ]);
    keccak(&encoded)
}

fn aggregate_transaction(txns: &[SafeTransaction], safe_multisend: &str) -> SafeTransaction {
    if txns.len() == 1 {
        txns[0].clone()
    } else {
        create_safe_multisend_transaction(txns, safe_multisend)
    }
}

pub trait AbstractSigner: Send + Sync {
    fn get_address(&self) -> Result<String>;
    fn sign_message(&self, hash32_hex: &str) -> Result<String>; // legacy message signing
    fn sign_eip712_digest(&self, digest_hex: &str) -> Result<String>; // explicit typed data digest signing
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignatureMode {
    /// EIP-191 over structHash (ethers.js signMessage on 32-byte struct hash)
    Eip191StructHash,
    /// Directly sign the EIP-712 digest (0x1901||domainSeparator||structHash)
    Eip712Digest,
    /// EIP-191 over the EIP-712 digest (ethers.js signMessage on digest returned by hashTypedData)
    Eip191Digest,
}

pub async fn build_safe_transaction_request(
    signer: &dyn AbstractSigner,
    args: SafeTransactionArgs,
    safe_contract_config: SafeContractConfig,
    metadata: Option<String>,
    sig_mode: SignatureMode,
) -> Result<TransactionRequest> {
    let safe_factory = &safe_contract_config.safe_factory;
    let safe_multisend = &safe_contract_config.safe_multisend;
    let transaction = aggregate_transaction(&args.transactions, safe_multisend);
    let safe_txn_gas = U256::zero();
    let base_gas = U256::zero();
    let gas_price = U256::zero();
    let gas_token: Address = "0x0000000000000000000000000000000000000000"
        .parse()
        .unwrap();
    let refund_receiver: Address = "0x0000000000000000000000000000000000000000"
        .parse()
        .unwrap();

    let safe_address = args
        .safe_address
        .clone()
        .unwrap_or_else(|| derive_safe(&args.from, safe_factory));

    eprintln!("[DEBUG] args.safe_address input: {:?}", args.safe_address);
    eprintln!("[DEBUG] Final safe_address used: {}", safe_address);

    let to_addr: Address = transaction
        .to
        .parse()
        .expect("invalid address in transaction.to");
    let value = U256::from_dec_str(&transaction.value).unwrap_or_default();
    let data_bytes = hex::decode(transaction.data.trim_start_matches("0x")).unwrap_or_default();
    let nonce = U256::from_dec_str(&args.nonce).unwrap_or_default();
    let struct_hash = safe_tx_struct_hash(
        to_addr,
        value,
        &data_bytes,
        transaction.operation as u8,
        safe_txn_gas,
        base_gas,
        gas_price,
        gas_token,
        refund_receiver,
        nonce,
    );
    let domain_separator =
        eip712_domain_separator(U256::from(args.chain_id), safe_address.parse().unwrap());
    let mut prefix = vec![0x19, 0x01];
    prefix.extend_from_slice(&domain_separator);
    prefix.extend_from_slice(&struct_hash);
    let digest = keccak(&prefix);

    eprintln!("[DEBUG] Safe address: {}", safe_address);
    eprintln!(
        "[DEBUG] Domain separator: 0x{}",
        hex::encode(domain_separator)
    );
    eprintln!("[DEBUG] Struct hash: 0x{}", hex::encode(struct_hash));
    eprintln!(
        "[DEBUG] Digest (0x1901||domain||struct): 0x{}",
        hex::encode(digest)
    );

    // Signature selection based on mode
    let sig = match sig_mode {
        SignatureMode::Eip191StructHash => {
            eprintln!("[DEBUG] SignatureMode=Eip191StructHash (signMessage(structHash))");
            signer.sign_message(&format!("0x{}", hex::encode(struct_hash)))?
        }
        SignatureMode::Eip712Digest => {
            eprintln!("[DEBUG] SignatureMode=Eip712Digest (sign_eip712_digest(digest))");
            signer.sign_eip712_digest(&format!("0x{}", hex::encode(digest)))?
        }
        SignatureMode::Eip191Digest => {
            eprintln!("[DEBUG] SignatureMode=Eip191Digest (signMessage(digest))");
            signer.sign_message(&format!("0x{}", hex::encode(digest)))?
        }
    };
    eprintln!("[DEBUG] Raw signature before pack: {}", sig);
    let packed_sig = split_and_pack_sig(&sig);
    eprintln!("[DEBUG] Packed signature: {}", packed_sig);

    // Verify signature recovers to correct address using the digest as the
    // signed message (this mirrors the sign_eip712_digest path used above).
    let signer_addr = signer.get_address()?;
    eprintln!("[DEBUG] Expected signer address: {}", signer_addr);

    use ethers::types::Signature as EthSig;
    if let Ok(sig_parsed) = packed_sig.parse::<EthSig>() {
        let verify_hash = match sig_mode {
            SignatureMode::Eip191StructHash => {
                let mut msg = b"\x19Ethereum Signed Message:\n32".to_vec();
                msg.extend_from_slice(&struct_hash);
                keccak(&msg)
            }
            SignatureMode::Eip712Digest => digest,
            SignatureMode::Eip191Digest => {
                let mut msg = b"\x19Ethereum Signed Message:\n32".to_vec();
                msg.extend_from_slice(&digest);
                keccak(&msg)
            }
        };
        if let Ok(recovered) = sig_parsed.recover(verify_hash) {
            eprintln!("[DEBUG] Recovered address: 0x{:x}", recovered);
            if format!("0x{:x}", recovered).to_lowercase() != signer_addr.to_lowercase() {
                eprintln!("[WARNING] Signature does not recover to signer address!");
            } else {
                eprintln!("[DEBUG] Signature recovery VERIFIED ✓");
            }
        }
    }

    let sig_params = SignatureParams {
        gas_price: Some(gas_price.to_string()),
        operation: Some((transaction.operation as u8).to_string()),
        safe_txn_gas: Some(safe_txn_gas.to_string()),
        base_gas: Some(base_gas.to_string()),
        gas_token: Some(format!("0x{:x}", gas_token)),
        refund_receiver: Some(format!("0x{:x}", refund_receiver)),
        ..Default::default()
    };

    Ok(TransactionRequest {
        from: args.from.clone(),
        to: transaction.to.clone(),
        proxy_wallet: Some(safe_address),
        data: transaction.data.clone(),
        nonce: Some(args.nonce.clone()),
        signature: packed_sig,
        signature_params: sig_params,
        r#type: TransactionType::SAFE,
        metadata,
    })
}
