use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RelayerTxType {
    #[serde(rename = "SAFE")]
    Safe,
    #[serde(rename = "PROXY")]
    Proxy,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum TransactionType {
    #[serde(rename = "SAFE")]
    SAFE,
    #[serde(rename = "SAFE-CREATE")]
    SafeCreate,
    #[serde(rename = "PROXY")]
    Proxy,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SignatureParams {
    #[serde(skip_serializing_if = "Option::is_none", rename = "gasPrice")]
    pub gas_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "relayerFee")]
    pub relayer_fee: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "gasLimit")]
    pub gas_limit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "relayHub")]
    pub relay_hub: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relay: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    // Match TS SDK naming (safeTxnGas)
    #[serde(skip_serializing_if = "Option::is_none", rename = "safeTxnGas")]
    pub safe_txn_gas: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "baseGas")]
    pub base_gas: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "gasToken")]
    pub gas_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "refundReceiver")]
    pub refund_receiver: Option<String>,

    // SAFE CREATE
    #[serde(skip_serializing_if = "Option::is_none", rename = "paymentToken")]
    pub payment_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "paymentReceiver")]
    pub payment_receiver: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NoncePayload {
    pub nonce: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub r#type: TransactionType,
    pub from: String,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "proxyWallet")]
    pub proxy_wallet: Option<String>,
    pub data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    pub signature: String,
    #[serde(rename = "signatureParams")]
    pub signature_params: SignatureParams,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum CallType {
    #[serde(rename = "0")]
    Invalid = 0,
    #[serde(rename = "1")]
    Call = 1,
    #[serde(rename = "2")]
    DelegateCall = 2,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProxyTransaction {
    #[serde(rename = "to")]
    pub to: String,
    #[serde(rename = "typeCode")]
    pub type_code: CallType,
    pub data: String,
    pub value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub to: String,
    pub data: String,
    pub value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProxyTransactionArgs {
    pub from: String,
    pub nonce: String,
    #[serde(rename = "gasPrice")]
    pub gas_price: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "gasLimit")]
    pub gas_limit: Option<String>,
    pub data: String,
    pub relay: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelayPayload {
    pub address: String,
    pub nonce: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum OperationType {
    Call = 0,
    DelegateCall = 1,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SafeTransaction {
    pub to: String,
    pub operation: OperationType,
    pub data: String,
    pub value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SafeTransactionArgs {
    pub from: String,
    pub nonce: String,
    pub chain_id: u64,
    pub transactions: Vec<SafeTransaction>,
    /// Optional: specify the actual Safe address instead of deriving it
    /// If provided, this address will be used directly; otherwise it will be derived from `from`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safe_address: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SafeCreateTransactionArgs {
    pub from: String,
    pub chain_id: u64,
    pub payment_token: String,
    pub payment: String,
    pub payment_receiver: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RelayerTransactionState {
    #[serde(rename = "STATE_NEW")]
    StateNew,
    #[serde(rename = "STATE_EXECUTED")]
    StateExecuted,
    #[serde(rename = "STATE_MINED")]
    StateMined,
    #[serde(rename = "STATE_INVALID")]
    StateInvalid,
    #[serde(rename = "STATE_CONFIRMED")]
    StateConfirmed,
    #[serde(rename = "STATE_FAILED")]
    StateFailed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelayerTransaction {
    pub transaction_id: String,
    pub transaction_hash: String,
    pub from: String,
    pub to: String,
    pub proxy_address: String,
    pub data: String,
    pub nonce: String,
    pub value: String,
    pub state: String,
    pub r#type: String,
    pub metadata: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelayerTransactionResponse {
    // Be flexible with field names across relayer versions
    #[serde(
        default,
        rename = "transactionID",
        alias = "transactionId",
        alias = "transaction_id",
        alias = "id"
    )]
    pub transaction_id: String,

    #[serde(default, alias = "status", alias = "state")]
    pub state: String,

    // Some responses use `hash` or `txHash`; keep both `hash` and `transaction_hash`
    #[serde(default, alias = "txHash", alias = "hash")]
    pub hash: String,

    #[serde(default, rename = "transactionHash", alias = "transaction_hash")]
    pub transaction_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetDeployedResponse {
    pub deployed: bool,
}
