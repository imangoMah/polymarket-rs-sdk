use crate::builder::derive::derive_safe;
use crate::errors::Result;
use crate::types::{
    SafeCreateTransactionArgs, SignatureParams, TransactionRequest, TransactionType,
};
use crate::utils::split_and_pack_sig;
use alloy::primitives::Address;
use async_trait::async_trait;
use std::str::FromStr;

// Define EIP-712 structs using alloy::sol!
pub const SAFE_FACTORY_NAME: &str = "Polymarket Contract Proxy Factory";

pub mod internal {
    alloy::sol! {
        #[derive(Debug)]
        struct CreateProxy {
            address paymentToken;
            uint256 payment;
            address paymentReceiver;
        }

        #[derive(Debug)]
        struct EIP712Domain {
            string name;
            uint256 chainId;
            address verifyingContract;
        }
    }
}
pub use internal::{CreateProxy, EIP712Domain};

#[async_trait]
pub trait AbstractSignerForCreate {
    fn address(&self) -> Address;
    async fn sign_typed_create_proxy(
        &self,
        safe_factory: &str,
        chain_id: u64,
        payment_token: &str,
        payment: &str,
        payment_receiver: &str,
    ) -> Result<String>;
}

pub async fn build_safe_create_transaction_request(
    signer: &dyn AbstractSignerForCreate,
    safe_factory: &str,
    args: SafeCreateTransactionArgs,
) -> Result<TransactionRequest> {
    let factory = Address::from_str(safe_factory)
        .map_err(|_| crate::errors::RelayClientError::InvalidAddress)?;
    let owner_addr = Address::from_str(&args.from)
        .map_err(|_| crate::errors::RelayClientError::InvalidAddress)?;

    // derive safe address
    let safe_address = derive_safe(factory, owner_addr)?;

    // Sign
    let sig_hex = signer
        .sign_typed_create_proxy(
            safe_factory,
            args.chain_id,
            &args.payment_token,
            &args.payment,
            &args.payment_receiver,
        )
        .await?;

    let packed_sig = split_and_pack_sig(&sig_hex);

    Ok(TransactionRequest {
        r#type: TransactionType::SafeCreate,
        from: args.from.clone(),
        to: safe_factory.to_string(),
        proxy_wallet: Some(safe_address.to_string()),
        data: "0x".to_string(),
        nonce: Some(args.payment.clone()),
        signature: packed_sig,
        signature_params: SignatureParams {
            payment_token: Some(args.payment_token.clone()),
            payment: Some(args.payment.clone()),
            payment_receiver: Some(args.payment_receiver.clone()),
            ..Default::default()
        },
        metadata: None,
    })
}
