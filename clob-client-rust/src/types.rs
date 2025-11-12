use serde::de::Deserializer;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyCreds {
    pub key: String,
    pub secret: String,
    pub passphrase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyRaw {
    pub api_key: String,
    pub secret: String,
    pub passphrase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeysResponse {
    pub api_keys: Vec<ApiKeyCreds>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanStatus {
    pub closed_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceAllowanceResponse {
    pub balance: String,
    pub allowance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderScoring {
    pub scoring: bool,
}

pub type OrdersScoring = std::collections::HashMap<String, bool>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    BUY,
    SELL,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderType {
    GTC,
    FOK,
    GTD,
    FAK,
}

pub type TickSize = String; // e.g. "0.01"

pub type TickSizes = std::collections::HashMap<String, TickSize>;
pub type NegRisk = std::collections::HashMap<String, bool>;
pub type FeeRates = std::collections::HashMap<String, u32>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Market {
    pub id: String,
    pub name: Option<String>,
    pub asset_id: Option<String>,
    pub min_order_size: Option<String>,
    pub tick_size: Option<String>,
    pub neg_risk: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MarketSummary {
    pub market: String,
    pub asset_id: String,
    pub timestamp: String,
    pub bids: Vec<OrderSummary>,
    pub asks: Vec<OrderSummary>,
    pub min_order_size: String,
    pub tick_size: String,
    pub neg_risk: bool,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderSummary {
    pub price: String,
    pub size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderBookSummary {
    pub market: String,
    pub asset_id: String,
    pub timestamp: String,
    pub bids: Vec<OrderSummary>,
    pub asks: Vec<OrderSummary>,
    pub min_order_size: String,
    pub tick_size: String,
    pub neg_risk: bool,
    pub hash: String,
}

/// Signature type for orders
/// - EOA: Standard Externally Owned Account (default)
/// - PolyProxy: Polymarket Proxy Wallet
/// - PolyGnosisSafe: Gnosis Safe Multisig
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SignatureType {
    EOA = 0,
    PolyProxy = 1,
    PolyGnosisSafe = 2,
}

impl Default for SignatureType {
    fn default() -> Self {
        SignatureType::EOA
    }
}

impl From<SignatureType> for u8 {
    fn from(sig_type: SignatureType) -> Self {
        sig_type as u8
    }
}

impl From<u8> for SignatureType {
    fn from(value: u8) -> Self {
        match value {
            0 => SignatureType::EOA,
            1 => SignatureType::PolyProxy,
            2 => SignatureType::PolyGnosisSafe,
            _ => SignatureType::EOA, // fallback to default
        }
    }
}

// Custom serialization: always serialize as number
impl Serialize for SignatureType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

// Custom deserialization: accept both string and number
impl<'de> Deserialize<'de> for SignatureType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SignatureTypeVisitor;

        impl<'de> serde::de::Visitor<'de> for SignatureTypeVisitor {
            type Value = SignatureType;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter
                    .write_str("a string (EOA, POLY_PROXY, POLY_GNOSIS_SAFE) or a number (0, 1, 2)")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    0 => Ok(SignatureType::EOA),
                    1 => Ok(SignatureType::PolyProxy),
                    2 => Ok(SignatureType::PolyGnosisSafe),
                    _ => Ok(SignatureType::EOA), // fallback
                }
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_u64(value as u64)
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "EOA" => Ok(SignatureType::EOA),
                    "POLY_PROXY" => Ok(SignatureType::PolyProxy),
                    "POLY_GNOSIS_SAFE" => Ok(SignatureType::PolyGnosisSafe),
                    _ => Ok(SignatureType::EOA), // fallback
                }
            }
        }

        deserializer.deserialize_any(SignatureTypeVisitor)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderData {
    pub maker: String,
    pub taker: String,
    pub token_id: String,
    pub maker_amount: String,
    pub taker_amount: String,
    pub side: Side,
    pub fee_rate_bps: String,
    pub nonce: String,
    pub signer: String,
    pub expiration: String,
    pub signature_type: SignatureType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignedOrder {
    pub salt: String,
    pub maker: String,
    pub signer: String,
    pub taker: String,
    pub token_id: String,
    pub maker_amount: String,
    pub taker_amount: String,
    pub expiration: String,
    pub nonce: String,
    pub fee_rate_bps: String,
    pub side: Side,
    pub signature_type: SignatureType,
    pub signature: String,
}

/// NewOrder is the payload structure for posting orders to the API
/// It wraps SignedOrder with orderType, owner, and deferExec fields
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewOrder {
    pub order: NewOrderData,
    pub owner: String,
    pub order_type: OrderType,
    #[serde(default)]
    pub defer_exec: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewOrderData {
    // API 要求 salt 为 number；为避免 JS 精度丢失，生成时限制在 2^53-1 范围内
    // 序列化时使用 i64 以确保 JSON 中是 number 而非字符串
    pub salt: i64,
    pub maker: String,
    pub signer: String,
    pub taker: String,
    pub token_id: String,
    pub maker_amount: String,
    pub taker_amount: String,
    pub expiration: String,
    pub nonce: String,
    pub fee_rate_bps: String,
    pub side: Side,
    pub signature_type: SignatureType,
    pub signature: String,
}

// Simple Chain constants
pub const CHAIN_POLYGON: i32 = 137;
pub const CHAIN_AMOY: i32 = 80002;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOrder {
    pub token_id: String,
    pub price: f64,
    pub size: f64,
    pub side: Side,
    // 改为必填: fee_rate_bps 由外部传入，不再在创建流程中自动获取
    pub fee_rate_bps: f64,
    pub nonce: Option<u64>,
    pub expiration: Option<u64>,
    pub taker: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserMarketOrder {
    pub token_id: String,
    // 改为必填: 市价单价格需由外部计算并传入
    pub price: f64,
    pub amount: f64,
    pub side: Side,
    // 改为必填: 由外部传入
    pub fee_rate_bps: f64,
    pub nonce: Option<u64>,
    pub taker: Option<String>,
    // order_type 改为必填: 市价单必须明确 FOK/FAK (或未来支持的其他类型)
    pub order_type: OrderType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub id: Option<String>,
    pub salt: Option<String>,
    pub maker: Option<String>,
    pub signer: Option<String>,
    pub taker: Option<String>,
    pub token_id: Option<String>,
    pub maker_amount: Option<String>,
    pub taker_amount: Option<String>,
    pub price: Option<String>,
    pub size: Option<String>,
    pub expiration: Option<String>,
    pub nonce: Option<String>,
    pub fee_rate_bps: Option<String>,
    pub side: Option<Side>,
    pub signature_type: Option<SignatureType>,
    pub signature: Option<String>,
    pub status: Option<OrderStatus>,
    pub metadata: Option<serde_json::Value>,
}

/// Response from GET /order endpoint
/// According to API docs: https://docs.polymarket.com/developers/CLOB/orders/get-order
/// Matches TypeScript SDK's OpenOrder interface
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OpenOrder {
    pub id: String,
    pub status: String,
    pub owner: String,
    pub maker_address: String,
    pub market: String,
    pub asset_id: String,
    pub side: String,
    pub original_size: String,
    pub size_matched: String,
    pub price: String,
    pub associate_trades: Vec<String>,
    pub outcome: String,
    pub created_at: u64,
    pub expiration: String,
    #[serde(rename = "type", alias = "order_type")]
    pub order_type: String,
}

/// Response from POST /order endpoint
/// According to API docs: https://docs.polymarket.com/developers/CLOB/orders/create-order
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResponse {
    /// Boolean indicating if server-side error occurred (success = false -> server-side error)
    pub success: bool,

    /// Error message in case of unsuccessful placement
    #[serde(rename = "errorMsg", default)]
    pub error_msg: String,

    /// ID of the order
    #[serde(rename = "orderID", alias = "orderId", default)]
    pub order_id: String,

    /// Hash of settlement transaction if order was marketable and triggered a match
    /// API docs call this "orderHashes", but TypeScript SDK uses "transactionsHashes"
    #[serde(rename = "orderHashes", alias = "transactionsHashes", default)]
    pub order_hashes: Vec<String>,

    /// Order status: "matched", "live", "delayed", "unmatched"
    #[serde(default)]
    pub status: Option<String>,

    /// Taking amount (not in official API docs, but returned by some endpoints)
    #[serde(default)]
    pub taking_amount: Option<String>,

    /// Making amount (not in official API docs, but returned by some endpoints)
    #[serde(default)]
    pub making_amount: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrderStatus {
    OPEN,
    FILLED,
    CANCELLED,
    OTHER(String),
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            OrderStatus::OPEN => "OPEN",
            OrderStatus::FILLED => "FILLED",
            OrderStatus::CANCELLED => "CANCELLED",
            OrderStatus::OTHER(v) => v.as_str(),
        };
        write!(f, "{}", s)
    }
}

impl Serialize for OrderStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for OrderStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "OPEN" | "open" => Ok(OrderStatus::OPEN),
            "FILLED" | "filled" => Ok(OrderStatus::FILLED),
            "CANCELLED" | "CANCELED" | "cancelled" | "canceled" => Ok(OrderStatus::CANCELLED),
            other => Ok(OrderStatus::OTHER(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    pub id: Option<String>,
    pub market: Option<String>,
    pub token_id: Option<String>,
    pub price: Option<String>,
    pub size: Option<String>,
    pub side: Option<Side>,
    pub maker: Option<String>,
    pub taker: Option<String>,
    pub timestamp: Option<String>,
    pub order_id: Option<String>,
    pub fee_rate_bps: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub id: Option<String>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub data: Option<serde_json::Value>,
    pub created_at: Option<String>,
    pub read: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reward {
    pub market: Option<String>,
    pub amount: Option<String>,
    pub timestamp: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketPrice {
    /// timestamp (ms or unix seconds depending on API)
    pub t: i64,
    /// price
    pub p: f64,
}
