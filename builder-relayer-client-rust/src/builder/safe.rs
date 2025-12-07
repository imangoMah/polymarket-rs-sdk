use crate::builder::derive::derive_safe;
use crate::encode::safe::create_safe_multisend_transaction;
use crate::errors::Result;
pub use crate::signer::AbstractSigner;
use crate::types::{
    SafeTransaction, SafeTransactionArgs, SignatureParams, TransactionRequest, TransactionType,
};
use crate::utils::split_and_pack_sig;
use alloy::primitives::{keccak256, Address, Bytes, U256};
use alloy::sol;
use alloy::sol_types::SolStruct;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct SafeContractConfig {
    pub safe_factory: String,
    pub safe_multisend: String,
}

// Define EIP-712 structs using alloy::sol!
sol! {
    #[derive(Debug)]
    struct SafeTx {
        address to;
        uint256 value;
        bytes data;
        uint8 operation;
        uint256 safeTxGas;
        uint256 baseGas;
        uint256 gasPrice;
        address gasToken;
        address refundReceiver;
        uint256 nonce;
    }

    #[derive(Debug)]
    struct EIP712Domain {
        uint256 chainId;
        address verifyingContract;
    }
}

pub fn eip712_domain_separator(chain_id: U256, verifying_contract: Address) -> [u8; 32] {
    let domain = EIP712Domain {
        chainId: chain_id,
        verifyingContract: verifying_contract,
    };
    domain.eip712_hash_struct().into()
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
    let tx = SafeTx {
        to,
        value,
        data: Bytes::from(data.to_vec()),
        operation,
        safeTxGas: safe_tx_gas,
        baseGas: base_gas,
        gasPrice: gas_price,
        gasToken: gas_token,
        refundReceiver: refund_receiver,
        nonce,
    };
    tx.eip712_hash_struct().into()
}

fn aggregate_transaction(txns: &[SafeTransaction], safe_multisend: &str) -> SafeTransaction {
    if txns.len() == 1 {
        txns[0].clone()
    } else {
        create_safe_multisend_transaction(txns, safe_multisend)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignatureMode {
    /// EIP-191 over structHash (ethers.js signMessage on 32-byte struct hash)
    Eip191StructHash,
    /// Directly sign the EIP-712 digest (0x1901||domainSeparator||structHash)
    Eip712Digest,
    Eip191Digest,
}

pub async fn build_safe_transaction_request(
    signer: &dyn AbstractSigner,
    args: &SafeTransactionArgs,
    config: &SafeContractConfig,
    metadata: Option<String>,
    mode: SignatureMode,
) -> Result<TransactionRequest> {
    let safe_address = if let Some(addr) = &args.safe_address {
        Address::from_str(addr).map_err(|_| crate::errors::RelayClientError::InvalidAddress)?
    } else {
        let owner = Address::from_str(&args.from)
            .map_err(|_| crate::errors::RelayClientError::InvalidAddress)?;
        let factory = Address::from_str(&config.safe_factory)
            .map_err(|_| crate::errors::RelayClientError::InvalidAddress)?;
        derive_safe(factory, owner)?
    };

    let tx = aggregate_transaction(&args.transactions, &config.safe_multisend);

    let to =
        Address::from_str(&tx.to).map_err(|_| crate::errors::RelayClientError::InvalidAddress)?;
    let value = U256::from_str(&tx.value).unwrap_or(U256::ZERO);
    let data = hex::decode(tx.data.trim_start_matches("0x")).unwrap_or_default();
    let operation = tx.operation as u8;

    // TS client sends zeroed gas fields by default; mirror that exactly.
    let safe_tx_gas = U256::ZERO;
    let base_gas = U256::ZERO;
    let gas_price = U256::ZERO;
    let gas_token = Address::ZERO;
    let refund_receiver = Address::ZERO;

    let nonce = U256::from_str(&args.nonce).unwrap_or(U256::ZERO);

    let struct_hash = safe_tx_struct_hash(
        to,
        value,
        &data,
        operation,
        safe_tx_gas,
        base_gas,
        gas_price,
        gas_token,
        refund_receiver,
        nonce,
    );

    let chain_id = U256::from(args.chain_id);
    let domain_separator = eip712_domain_separator(chain_id, safe_address);

    // digest = keccak256("\x19\x01" || domainSeparator || structHash)
    let mut digest_input = Vec::with_capacity(2 + 32 + 32);
    digest_input.extend_from_slice(&[0x19, 0x01]);
    digest_input.extend_from_slice(&domain_separator);
    digest_input.extend_from_slice(&struct_hash);
    let digest = keccak256(&digest_input);

    let signature = match mode {
        SignatureMode::Eip191StructHash => {
            let mut msg = Vec::with_capacity(60);
            msg.extend_from_slice(b"\x19Ethereum Signed Message:\n32");
            msg.extend_from_slice(&struct_hash);
            let hash = keccak256(&msg);

            signer.sign_hash(hash).await?
        }
        SignatureMode::Eip712Digest => signer.sign_eip712_digest(digest).await?,
        SignatureMode::Eip191Digest => {
            // EIP-191 over the EIP-712 digest (TS signMessage(hashTypedData(...)))
            let mut msg = Vec::with_capacity(60);
            msg.extend_from_slice(b"\x19Ethereum Signed Message:\n32");
            msg.extend_from_slice(digest.as_slice());
            let hash = keccak256(&msg);
            signer.sign_hash(hash).await?
        }
    };

    // Convert signature to hex string for split_and_pack_sig
    let sig_hex = format!("0x{}", hex::encode(signature.as_bytes()));
    let packed_sig = split_and_pack_sig(&sig_hex);

    let signature_params = SignatureParams {
        gas_price: Some(gas_price.to_string()),
        relayer_fee: None,
        gas_limit: None,
        relay_hub: None,
        relay: None,
        operation: Some(operation.to_string()),
        safe_txn_gas: Some(safe_tx_gas.to_string()),
        base_gas: Some(base_gas.to_string()),
        gas_token: Some(format!("{:#x}", gas_token)),
        refund_receiver: Some(format!("{:#x}", refund_receiver)),
        payment_token: None,
        payment: None,
        payment_receiver: None,
    };

    Ok(TransactionRequest {
        r#type: TransactionType::SAFE,
        from: args.from.clone(),
        to: tx.to.clone(),
        proxy_wallet: Some(format!("{:#x}", safe_address)),
        data: tx.data.clone(),
        nonce: Some(args.nonce.clone()),
        signature: packed_sig,
        signature_params,
        metadata,
    })
}
