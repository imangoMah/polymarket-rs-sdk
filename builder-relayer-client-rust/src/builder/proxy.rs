use crate::builder::derive::derive_proxy;
use crate::encode::proxy::encode_proxy_transaction_data;
use crate::errors::{RelayClientError, Result};
use crate::signer::AbstractSigner;
use crate::types::{
    ProxyTransaction, ProxyTransactionArgs, SignatureParams, TransactionRequest, TransactionType,
};
use alloy::primitives::{keccak256, Address, B256, U256};
use std::str::FromStr;

const DEFAULT_GAS_LIMIT: &str = "10000000";
const RELAY_HUB_PREFIX: &[u8] = b"rlx:";

#[derive(Clone, Debug)]
pub struct ProxyContractConfig {
    pub relay_hub: String,
    pub proxy_factory: String,
}

pub fn is_proxy_contract_config_valid(config: &ProxyContractConfig) -> bool {
    !config.proxy_factory.is_empty() && !config.relay_hub.is_empty()
}

fn as_be_bytes_32(value: &str) -> U256 {
    U256::from_str_radix(value, 10).unwrap_or(U256::ZERO)
}

fn create_struct_hash(
    from: &str,
    to: &str,
    data_hex: &str,
    tx_fee: &str,
    gas_price: &str,
    gas_limit: &str,
    nonce: &str,
    relay_hub: &str,
    relay: &str,
) -> Result<B256> {
    let from_addr = Address::from_str(from).map_err(|_| RelayClientError::InvalidAddress)?;
    let to_addr = Address::from_str(to).map_err(|_| RelayClientError::InvalidAddress)?;
    let relay_hub_addr =
        Address::from_str(relay_hub).map_err(|_| RelayClientError::InvalidAddress)?;
    let relay_addr = Address::from_str(relay).map_err(|_| RelayClientError::InvalidAddress)?;

    let mut data_bytes = data_hex.trim_start_matches("0x").as_bytes().to_vec();
    if let Ok(decoded) = hex::decode(data_hex.trim_start_matches("0x")) {
        data_bytes = decoded;
    }

    let tx_fee_bytes = as_be_bytes_32(tx_fee).to_be_bytes::<32>();
    let gas_price_bytes = as_be_bytes_32(gas_price).to_be_bytes::<32>();
    let gas_limit_bytes = as_be_bytes_32(gas_limit).to_be_bytes::<32>();
    let nonce_bytes = as_be_bytes_32(nonce).to_be_bytes::<32>();

    let mut payload: Vec<u8> = Vec::new();
    payload.extend_from_slice(RELAY_HUB_PREFIX);
    payload.extend_from_slice(from_addr.as_slice());
    payload.extend_from_slice(to_addr.as_slice());
    payload.extend_from_slice(&data_bytes);
    payload.extend_from_slice(&tx_fee_bytes);
    payload.extend_from_slice(&gas_price_bytes);
    payload.extend_from_slice(&gas_limit_bytes);
    payload.extend_from_slice(&nonce_bytes);
    payload.extend_from_slice(relay_hub_addr.as_slice());
    payload.extend_from_slice(relay_addr.as_slice());

    Ok(keccak256(payload))
}

async fn create_proxy_signature(signer: &dyn AbstractSigner, struct_hash: B256) -> Result<String> {
    // Mirror ethers.js signMessage on a 32-byte struct hash
    let mut msg = Vec::with_capacity(2 + 32 + 32);
    msg.extend_from_slice(b"\x19Ethereum Signed Message:\n32");
    msg.extend_from_slice(struct_hash.as_slice());
    let digest = keccak256(&msg);

    let sig = signer.sign_hash(digest).await?;
    Ok(format!("0x{}", hex::encode(sig.as_bytes())))
}

pub async fn build_proxy_transaction_request(
    signer: &dyn AbstractSigner,
    args: &ProxyTransactionArgs,
    proxy_contract_config: &ProxyContractConfig,
    metadata: Option<String>,
    txns: &[ProxyTransaction],
) -> Result<TransactionRequest> {
    if !is_proxy_contract_config_valid(proxy_contract_config) {
        return Err(RelayClientError::InvalidNetwork);
    }

    let proxy_factory = &proxy_contract_config.proxy_factory;
    let from_addr = Address::from_str(&args.from).map_err(|_| RelayClientError::InvalidAddress)?;
    let factory_addr =
        Address::from_str(proxy_factory).map_err(|_| RelayClientError::InvalidAddress)?;
    let proxy_wallet = derive_proxy(factory_addr, from_addr)?.to_string();

    let relayer_fee = "0".to_string();
    let gas_limit = args
        .gas_limit
        .clone()
        .filter(|g| !g.is_empty() && g != "0")
        .unwrap_or_else(|| DEFAULT_GAS_LIMIT.to_string());

    let struct_hash = create_struct_hash(
        &args.from,
        proxy_factory,
        &args.data,
        &relayer_fee,
        &args.gas_price,
        &gas_limit,
        &args.nonce,
        &proxy_contract_config.relay_hub,
        &args.relay,
    )?;

    let signature = create_proxy_signature(signer, struct_hash).await?;

    let sig_params = SignatureParams {
        gas_price: Some(args.gas_price.clone()),
        relayer_fee: Some(relayer_fee.clone()),
        gas_limit: Some(gas_limit.clone()),
        relay_hub: Some(proxy_contract_config.relay_hub.clone()),
        relay: Some(args.relay.clone()),
        ..SignatureParams::default()
    };

    let packed_metadata = metadata.unwrap_or_default();
    let data = encode_proxy_transaction_data(txns);

    Ok(TransactionRequest {
        r#type: TransactionType::Proxy,
        from: args.from.clone(),
        to: proxy_factory.clone(),
        proxy_wallet: Some(proxy_wallet),
        data,
        nonce: Some(args.nonce.clone()),
        signature,
        signature_params: sig_params,
        metadata: Some(packed_metadata),
    })
}
