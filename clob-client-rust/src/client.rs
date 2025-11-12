use crate::constants::{END_CURSOR, INITIAL_CURSOR};
use crate::endpoints::*;
use crate::errors::ClobError;
// use crate::exchange_consts::ZERO_ADDRESS; // no longer needed; we resolve real exchange addresses dynamically
use crate::http_helpers::{RequestOptions, get, post};
use crate::order_builder::{BuilderConfig as ObBuilderConfig, OrderBuilder};
use crate::signer_adapter::EthersSigner;
use crate::types::OrderBookSummary;
use crate::types::{ApiKeyCreds, ApiKeyRaw};
use crate::types::{
    Notification, OpenOrder, Order, OrderResponse, OrderType, Reward, SignedOrder, Trade,
    UserMarketOrder, UserOrder,
};
// Removed unused alias import (Signer) after refactor; keep file clean
use serde::Deserialize;
use serde_json::Value;

#[allow(dead_code)]
#[derive(Deserialize)]
struct OkResp {
    ok: bool,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct SuccessResp {
    success: bool,
}
// Helper enums to accept either a bare array or an object with a `data` field
#[derive(Deserialize)]
#[serde(untagged)]
enum MaybeVec<T> {
    Vec(Vec<T>),
    Data { data: Vec<T> },
}

impl<T> MaybeVec<T> {
    fn into_vec(self) -> Vec<T> {
        match self {
            MaybeVec::Vec(v) => v,
            MaybeVec::Data { data } => data,
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum MaybeItem<T> {
    Item(T),
    Data { data: T },
}

impl<T> MaybeItem<T> {
    fn into_item(self) -> T {
        match self {
            MaybeItem::Item(i) => i,
            MaybeItem::Data { data } => data,
        }
    }
}
use std::collections::HashMap;
use std::sync::Arc;

pub struct ClobClient {
    pub host: String,
    pub chain_id: i64,
    pub signer: Option<Arc<EthersSigner>>,
    pub creds: Option<ApiKeyCreds>,
    pub use_server_time: bool,
    pub tick_sizes: HashMap<String, String>,
    pub neg_risk: HashMap<String, bool>,
    pub fee_rates: HashMap<String, f64>,
    // Optional Builder signer for builder-authenticated flows
    pub builder_signer: Option<builder_signing_sdk_rs::BuilderSigner>,
    // Optional default builder config for order creation
    pub builder_config: Option<ObBuilderConfig>,
}

impl ClobClient {
    // --- TypeScript parity alias section --------------------------------------------------
    // These thin wrappers mirror the naming style of the original TypeScript client so that
    // porting example code is mostly a mechanical rename (camelCase). They delegate to the
    // existing snake_case Rust methods. Prefer using the snake_case versions in new Rust code.

    /// TS parity alias for `get_markets` (camelCase). Prefer `get_markets` in Rust.
    #[allow(non_snake_case)]
    pub async fn getMarkets(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Market>, ClobError> {
        self.get_markets(params).await
    }

    /// TS parity alias for `getMarket` -> `get_market`.
    #[allow(non_snake_case)]
    pub async fn getMarket(
        &self,
        market_id: &str,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<crate::types::MarketSummary, ClobError> {
        self.get_market(market_id, params).await
    }

    #[allow(non_snake_case)]
    pub async fn getOrderBook(&mut self, token_id: &str) -> Result<OrderBookSummary, ClobError> {
        self.get_order_book(token_id).await
    }

    #[allow(non_snake_case)]
    pub async fn getTickSize(&mut self, token_id: &str) -> Result<String, ClobError> {
        self.get_tick_size(token_id).await
    }

    #[allow(non_snake_case)]
    pub async fn getNegRisk(&mut self, token_id: &str) -> Result<bool, ClobError> {
        self.get_neg_risk(token_id).await
    }

    #[allow(non_snake_case)]
    pub async fn getFeeRate(&mut self, token_id: &str) -> Result<f64, ClobError> {
        self.get_fee_rate(token_id).await
    }

    #[allow(non_snake_case)]
    pub async fn getOpenOrders(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<SignedOrder>, ClobError> {
        self.get_open_orders(params).await
    }

    #[allow(non_snake_case)]
    pub async fn getOrder(&self, order_id: &str) -> Result<OpenOrder, ClobError> {
        self.get_order(order_id).await
    }

    #[allow(non_snake_case)]
    pub async fn postSignedOrder(
        &self,
        signed_order: &SignedOrder,
    ) -> Result<OrderResponse, ClobError> {
        self.post_signed_order(signed_order, OrderType::GTC, false)
            .await
    }

    #[allow(non_snake_case)]
    pub async fn postOrders(
        &self,
        orders: Vec<SignedOrder>,
        defer_exec: bool,
    ) -> Result<Vec<Order>, ClobError> {
        self.post_orders(orders, defer_exec).await
    }

    #[allow(non_snake_case)]
    pub async fn createOrder(
        &mut self,
        user_order: UserOrder,
        options_tick: Option<&str>,
    ) -> Result<SignedOrder, ClobError> {
        self.create_order(user_order, options_tick).await
    }

    #[allow(non_snake_case)]
    pub async fn createMarketOrder(
        &mut self,
        user_market_order: UserMarketOrder,
        options_tick: Option<&str>,
    ) -> Result<SignedOrder, ClobError> {
        self.create_market_order(user_market_order, options_tick)
            .await
    }

    #[allow(non_snake_case)]
    pub async fn createAndPostOrder(
        &mut self,
        user_order: UserOrder,
        options_tick: Option<&str>,
    ) -> Result<OrderResponse, ClobError> {
        self.create_and_post_order(user_order, options_tick, None)
            .await
    }

    #[allow(non_snake_case)]
    pub async fn createAndPostMarketOrder(
        &mut self,
        user_market_order: UserMarketOrder,
        options_tick: Option<&str>,
    ) -> Result<OrderResponse, ClobError> {
        self.create_and_post_market_order(user_market_order, options_tick, None)
            .await
    }

    #[allow(non_snake_case)]
    pub async fn cancelOrder(&self, order_id: &str) -> Result<Order, ClobError> {
        self.cancel_order(order_id).await
    }

    #[allow(non_snake_case)]
    pub async fn cancelOrders(&self, order_ids: Vec<String>) -> Result<Vec<Order>, ClobError> {
        self.cancel_orders(order_ids).await
    }

    #[allow(non_snake_case)]
    pub async fn cancelAll(&self) -> Result<Vec<Order>, ClobError> {
        self.cancel_all().await
    }

    #[allow(non_snake_case)]
    pub async fn isOrderScoring(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<crate::types::OrderScoring, ClobError> {
        self.is_order_scoring(params).await
    }

    #[allow(non_snake_case)]
    pub async fn areOrdersScoring(
        &self,
        order_ids: Option<Vec<String>>,
    ) -> Result<crate::types::OrdersScoring, ClobError> {
        self.are_orders_scoring(order_ids).await
    }

    #[allow(non_snake_case)]
    pub async fn getTrades(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
        only_first_page: bool,
        next_cursor: Option<String>,
    ) -> Result<Vec<Value>, ClobError> {
        self.get_trades(params, only_first_page, next_cursor).await
    }

    #[allow(non_snake_case)]
    pub async fn getTradesTyped(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
        only_first_page: bool,
        next_cursor: Option<String>,
    ) -> Result<Vec<Trade>, ClobError> {
        self.get_trades_typed(params, only_first_page, next_cursor)
            .await
    }

    #[allow(non_snake_case)]
    pub async fn getNotifications(&self) -> Result<Vec<Notification>, ClobError> {
        self.get_notifications().await
    }

    #[allow(non_snake_case)]
    pub async fn dropNotifications(&self, ids: Option<&Vec<String>>) -> Result<(), ClobError> {
        self.drop_notifications(ids).await
    }

    #[allow(non_snake_case)]
    pub async fn getBalanceAllowance(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<crate::types::BalanceAllowanceResponse, ClobError> {
        self.get_balance_allowance(params).await
    }

    #[allow(non_snake_case)]
    pub async fn updateBalanceAllowance(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<(), ClobError> {
        self.update_balance_allowance(params).await
    }

    #[allow(non_snake_case)]
    pub async fn getTime(&self) -> Result<u64, ClobError> {
        self.get_server_time().await
    }

    // API keys (L2)
    #[allow(non_snake_case)]
    pub async fn getApiKeys(&self) -> Result<Vec<crate::types::ApiKeyCreds>, ClobError> {
        self.get_api_keys().await
    }
    #[allow(non_snake_case)]
    pub async fn deleteApiKey(&self) -> Result<(), ClobError> {
        self.delete_api_key().await
    }
    #[allow(non_snake_case)]
    pub async fn getClosedOnlyMode(&self) -> Result<crate::types::BanStatus, ClobError> {
        self.get_closed_only_mode().await
    }

    // L1-derived API keys
    #[allow(non_snake_case)]
    pub async fn createApiKey(&self, nonce: Option<u64>) -> Result<ApiKeyCreds, ClobError> {
        self.create_api_key(nonce).await
    }
    #[allow(non_snake_case)]
    pub async fn deriveApiKey(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<ApiKeyCreds, ClobError> {
        self.derive_api_key(params).await
    }

    // Builder API keys
    #[allow(non_snake_case)]
    pub async fn createBuilderApiKey(&self) -> Result<ApiKeyCreds, ClobError> {
        self.create_builder_api_key().await
    }
    #[allow(non_snake_case)]
    pub async fn getBuilderApiKeys(&self) -> Result<Vec<ApiKeyCreds>, ClobError> {
        self.get_builder_api_keys().await
    }
    #[allow(non_snake_case)]
    pub async fn revokeBuilderApiKey(&self, id: &str) -> Result<(), ClobError> {
        self.revoke_builder_api_key(id).await
    }

    // Markets/prices aliases
    #[allow(non_snake_case)]
    pub async fn getSimplifiedMarkets(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Market>, ClobError> {
        self.get_simplified_markets(params).await
    }
    #[allow(non_snake_case)]
    pub async fn getSamplingMarkets(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Market>, ClobError> {
        self.get_sampling_markets(params).await
    }
    #[allow(non_snake_case)]
    pub async fn getSamplingSimplifiedMarkets(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Market>, ClobError> {
        self.get_sampling_simplified_markets(params).await
    }
    #[allow(non_snake_case)]
    pub async fn getOrderBooks(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::OrderBookSummary>, ClobError> {
        self.get_order_books(params).await
    }
    #[allow(non_snake_case)]
    pub async fn getMidpoint(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::MarketPrice>, ClobError> {
        self.get_midpoint(params).await
    }
    #[allow(non_snake_case)]
    pub async fn getMidpoints(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::MarketPrice>, ClobError> {
        self.get_midpoints(params).await
    }
    #[allow(non_snake_case)]
    pub async fn getPrices(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::MarketPrice>, ClobError> {
        self.get_prices(params).await
    }
    #[allow(non_snake_case)]
    pub async fn getSpreads(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::MarketPrice>, ClobError> {
        self.get_spreads(params).await
    }
    #[allow(non_snake_case)]
    pub async fn getLastTradesPrices(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::MarketPrice>, ClobError> {
        self.get_last_trades_prices(params).await
    }
    #[allow(non_snake_case)]
    pub async fn getPricesHistory(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::MarketPrice>, ClobError> {
        self.get_prices_history(params).await
    }
    #[allow(non_snake_case)]
    pub async fn getMarketTradesEvents(
        &self,
        market_id: &str,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Trade>, ClobError> {
        self.get_market_trades_events(market_id, params).await
    }

    // Builder trades
    #[allow(non_snake_case)]
    pub async fn getBuilderTrades(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Trade>, ClobError> {
        self.get_builder_trades(params).await
    }

    // Rewards
    #[allow(non_snake_case)]
    pub async fn getEarningsForUserForDay(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Reward>, ClobError> {
        self.get_earnings_for_user_for_day(params).await
    }
    #[allow(non_snake_case)]
    pub async fn getEarningsForUserForDayTyped(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Reward>, ClobError> {
        self.get_rewards_user_for_day_typed(params).await
    }
    #[allow(non_snake_case)]
    pub async fn getTotalEarningsForUserForDay(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Reward>, ClobError> {
        self.get_total_earnings_for_user_for_day(params).await
    }
    #[allow(non_snake_case)]
    pub async fn getLiquidityRewardPercentages(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<std::collections::HashMap<String, f64>, ClobError> {
        self.get_liquidity_reward_percentages(params).await
    }
    #[allow(non_snake_case)]
    pub async fn getRewardsMarketsCurrent(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Reward>, ClobError> {
        self.get_rewards_markets_current(params).await
    }
    #[allow(non_snake_case)]
    pub async fn getRewardsEarningsPercentages(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Reward>, ClobError> {
        self.get_rewards_earnings_percentages(params).await
    }
    // --------------------------------------------------------------------------------------
    pub fn new(
        host: &str,
        chain_id: i64,
        signer: Option<Arc<EthersSigner>>,
        creds: Option<ApiKeyCreds>,
        use_server_time: bool,
    ) -> Self {
        Self {
            host: if let Some(stripped) = host.strip_suffix('/') {
                stripped.to_string()
            } else {
                host.to_string()
            },
            chain_id,
            signer,
            creds,
            use_server_time,
            tick_sizes: HashMap::new(),
            neg_risk: HashMap::new(),
            fee_rates: HashMap::new(),
            builder_signer: None,
            builder_config: None,
        }
    }

    /// Configure Builder API signer (builder auth). Secret must be base64 encoded.
    pub fn with_builder_signer(
        mut self,
        key: String,
        secret_b64: String,
        passphrase: String,
    ) -> Self {
        let creds = builder_signing_sdk_rs::BuilderApiKeyCreds {
            key,
            secret: secret_b64,
            passphrase,
        };
        self.builder_signer = Some(builder_signing_sdk_rs::BuilderSigner::new(creds));
        self
    }

    /// Set a default builder config for order creation (tick size, neg risk, signature type, funder).
    pub fn with_builder_config(mut self, cfg: ObBuilderConfig) -> Self {
        self.builder_config = Some(cfg);
        self
    }

    pub async fn get_order_book(&mut self, token_id: &str) -> Result<OrderBookSummary, ClobError> {
        let endpoint = format!("{}{}", self.host, GET_ORDER_BOOK);
        let mut params = std::collections::HashMap::new();
        params.insert("token_id".to_string(), token_id.to_string());
        let opts = RequestOptions {
            headers: None,
            data: None,
            params: Some(params),
        };
        let val = get(&endpoint, Some(opts)).await?;
        let obs: OrderBookSummary =
            serde_json::from_value(val).map_err(|e| ClobError::Other(e.to_string()))?;
        Ok(obs)
    }

    pub async fn get_tick_size(&mut self, token_id: &str) -> Result<String, ClobError> {
        if let Some(v) = self.tick_sizes.get(token_id) {
            return Ok(v.clone());
        }
        let endpoint = format!("{}{}", self.host, GET_TICK_SIZE);
        let mut params = std::collections::HashMap::new();
        params.insert("token_id".to_string(), token_id.to_string());
        let opts = RequestOptions {
            headers: None,
            data: None,
            params: Some(params),
        };
        let val = get(&endpoint, Some(opts)).await?;
        let tick = val
            .get("minimum_tick_size")
            .and_then(|v| v.as_str())
            .ok_or(ClobError::Other("invalid tick response".to_string()))?;
        self.tick_sizes
            .insert(token_id.to_string(), tick.to_string());
        Ok(tick.to_string())
    }

    pub async fn get_neg_risk(&mut self, token_id: &str) -> Result<bool, ClobError> {
        if let Some(v) = self.neg_risk.get(token_id) {
            return Ok(*v);
        }
        let endpoint = format!("{}{}", self.host, GET_NEG_RISK);
        let mut params = std::collections::HashMap::new();
        params.insert("token_id".to_string(), token_id.to_string());
        let opts = RequestOptions {
            headers: None,
            data: None,
            params: Some(params),
        };
        let val = get(&endpoint, Some(opts)).await?;
        let rr = val
            .get("neg_risk")
            .and_then(|v| v.as_bool())
            .ok_or(ClobError::Other("invalid neg risk response".to_string()))?;
        self.neg_risk.insert(token_id.to_string(), rr);
        Ok(rr)
    }

    pub async fn get_fee_rate(&mut self, token_id: &str) -> Result<f64, ClobError> {
        if let Some(v) = self.fee_rates.get(token_id) {
            return Ok(*v);
        }
        let endpoint = format!("{}{}", self.host, GET_FEE_RATE);
        let mut params = std::collections::HashMap::new();
        params.insert("token_id".to_string(), token_id.to_string());
        let opts = RequestOptions {
            headers: None,
            data: None,
            params: Some(params),
        };
        let val = get(&endpoint, Some(opts)).await?;
        let fee = val
            .get("base_fee")
            .and_then(|v| v.as_f64())
            .ok_or(ClobError::Other("invalid fee response".to_string()))?;
        self.fee_rates.insert(token_id.to_string(), fee);
        Ok(fee)
    }

    /// Convenience method that accepts a typed `SignedOrder` and posts it to the
    /// API. Delegates to `post_signed_order` which performs serialization,
    /// header creation and response parsing. Uses default orderType=GTC.
    pub async fn post_order(&self, signed_order: &SignedOrder) -> Result<OrderResponse, ClobError> {
        // Delegate to the typed helper with default GTC order type
        self.post_signed_order(signed_order, OrderType::GTC, false)
            .await
    }

    pub async fn get_api_keys(&self) -> Result<Vec<crate::types::ApiKeyCreds>, ClobError> {
        if self.creds.is_none() {
            return Err(ClobError::Other("L2 creds required".to_string()));
        }
        let signer_arc = self
            .signer
            .as_ref()
            .ok_or(ClobError::Other("L1 signer required".to_string()))?;
        let signer_ref: &EthersSigner = signer_arc.as_ref();
        let endpoint = format!("{}{}", self.host, GET_API_KEYS);
        let ts = if self.use_server_time {
            Some(self.get_server_time().await?)
        } else {
            None
        };
        let headers = crate::headers::create_l2_headers(
            signer_ref,
            self.creds.as_ref().unwrap(),
            "GET",
            GET_API_KEYS,
            None,
            ts,
        )
        .await?;
        let resp: crate::types::ApiKeysResponse = crate::http_helpers::get_typed(
            &endpoint,
            Some(RequestOptions::<Value> {
                headers: Some(headers),
                data: None,
                params: None,
            }),
        )
        .await?;
        Ok(resp.api_keys)
    }

    pub async fn get_closed_only_mode(&self) -> Result<crate::types::BanStatus, ClobError> {
        if self.creds.is_none() {
            return Err(ClobError::Other("L2 creds required".to_string()));
        }
        let signer_arc = self
            .signer
            .as_ref()
            .ok_or(ClobError::Other("L1 signer required".to_string()))?;
        let signer_ref: &EthersSigner = signer_arc.as_ref();
        let endpoint = format!("{}{}", self.host, CLOSED_ONLY);
        let ts = if self.use_server_time {
            Some(self.get_server_time().await?)
        } else {
            None
        };
        let headers = crate::headers::create_l2_headers(
            signer_ref,
            self.creds.as_ref().unwrap(),
            "GET",
            CLOSED_ONLY,
            None,
            ts,
        )
        .await?;
        let resp: crate::types::BanStatus = crate::http_helpers::get_typed(
            &endpoint,
            Some(RequestOptions::<Value> {
                headers: Some(headers),
                data: None,
                params: None,
            }),
        )
        .await?;
        Ok(resp)
    }

    pub async fn delete_api_key(&self) -> Result<(), ClobError> {
        if self.creds.is_none() {
            return Err(ClobError::Other("L2 creds required".to_string()));
        }
        let signer_arc = self
            .signer
            .as_ref()
            .ok_or(ClobError::Other("L1 signer required".to_string()))?;
        let signer_ref: &EthersSigner = signer_arc.as_ref();
        let endpoint = format!("{}{}", self.host, DELETE_API_KEY);
        let ts = if self.use_server_time {
            Some(self.get_server_time().await?)
        } else {
            None
        };
        let headers = crate::headers::create_l2_headers(
            signer_ref,
            self.creds.as_ref().unwrap(),
            "DELETE",
            DELETE_API_KEY,
            None,
            ts,
        )
        .await?;
        let _val: () = crate::http_helpers::del_typed::<(), Value>(
            &endpoint,
            Some(RequestOptions::<Value> {
                headers: Some(headers),
                data: None,
                params: None,
            }),
        )
        .await?;
        Ok(())
    }

    pub async fn get_trades(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
        only_first_page: bool,
        next_cursor: Option<String>,
    ) -> Result<Vec<Value>, ClobError> {
        if self.creds.is_none() {
            return Err(ClobError::Other("L2 creds required".to_string()));
        }
        let signer_arc = self
            .signer
            .as_ref()
            .ok_or(ClobError::Other("L1 signer required".to_string()))?;
        let signer_ref: &EthersSigner = signer_arc.as_ref();
        let ts = if self.use_server_time {
            Some(self.get_server_time().await?)
        } else {
            None
        };
        let headers = crate::headers::create_l2_headers(
            signer_ref,
            self.creds.as_ref().unwrap(),
            "GET",
            GET_TRADES,
            None,
            ts,
        )
        .await?;
        let mut results: Vec<Value> = vec![];
        let mut cursor = next_cursor.unwrap_or_else(|| INITIAL_CURSOR.to_string());
        while cursor != END_CURSOR {
            if only_first_page && cursor != INITIAL_CURSOR {
                break;
            }
            let mut p = params.clone().unwrap_or_default();
            p.insert("next_cursor".to_string(), cursor.clone());
            let val = get(
                &format!("{}{}", self.host, GET_TRADES),
                Some(RequestOptions {
                    headers: Some(headers.clone()),
                    data: None,
                    params: Some(p),
                }),
            )
            .await?;
            let data = val
                .get("data")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            for item in data {
                results.push(item);
            }
            cursor = val
                .get("next_cursor")
                .and_then(|v| v.as_str())
                .unwrap_or(END_CURSOR)
                .to_string();
        }
        Ok(results)
    }

    /// Typed variant of get_trades that deserializes each trade into `Trade`.
    pub async fn get_trades_typed(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
        only_first_page: bool,
        next_cursor: Option<String>,
    ) -> Result<Vec<Trade>, ClobError> {
        let vals = self
            .get_trades(params, only_first_page, next_cursor)
            .await?;
        let mut trades: Vec<Trade> = Vec::new();
        for v in vals {
            let t: Trade =
                serde_json::from_value(v).map_err(|e| ClobError::Other(e.to_string()))?;
            trades.push(t);
        }
        Ok(trades)
    }

    pub async fn get_notifications(&self) -> Result<Vec<Notification>, ClobError> {
        if self.creds.is_none() {
            return Err(ClobError::Other("L2 creds required".to_string()));
        }
        let signer_arc = self
            .signer
            .as_ref()
            .ok_or(ClobError::Other("L1 signer required".to_string()))?;
        let signer_ref: &EthersSigner = signer_arc.as_ref();
        let endpoint = format!("{}{}", self.host, GET_NOTIFICATIONS);
        let mut params = std::collections::HashMap::new();
        params.insert("signature_type".to_string(), "EOA".to_string());
        let ts = if self.use_server_time {
            Some(self.get_server_time().await?)
        } else {
            None
        };
        let headers = crate::headers::create_l2_headers(
            signer_ref,
            self.creds.as_ref().unwrap(),
            "GET",
            GET_NOTIFICATIONS,
            None,
            ts,
        )
        .await?;
        let resp: MaybeVec<Notification> = crate::http_helpers::get_typed(
            &endpoint,
            Some(RequestOptions::<Value> {
                headers: Some(headers),
                data: None,
                params: Some(params),
            }),
        )
        .await?;
        Ok(resp.into_vec())
    }

    /// Typed variant of get_notifications that deserializes notifications into `Notification`.
    pub async fn get_notifications_typed(&self) -> Result<Vec<Notification>, ClobError> {
        // Kept for compatibility: delegate to get_notifications
        self.get_notifications().await
    }

    pub async fn drop_notifications(&self, ids: Option<&Vec<String>>) -> Result<(), ClobError> {
        if self.creds.is_none() {
            return Err(ClobError::Other("L2 creds required".to_string()));
        }
        let signer_arc = self
            .signer
            .as_ref()
            .ok_or(ClobError::Other("L1 signer required".to_string()))?;
        let signer_ref: &EthersSigner = signer_arc.as_ref();
        let endpoint = format!("{}{}", self.host, DROP_NOTIFICATIONS);
        let params = crate::http_helpers::parse_drop_notification_params(ids);
        let ts = if self.use_server_time {
            Some(self.get_server_time().await?)
        } else {
            None
        };
        let headers = crate::headers::create_l2_headers(
            signer_ref,
            self.creds.as_ref().unwrap(),
            "DELETE",
            DROP_NOTIFICATIONS,
            None,
            ts,
        )
        .await?;
        let _raw: SuccessResp = crate::http_helpers::del_typed(
            &endpoint,
            Some(RequestOptions::<Value> {
                headers: Some(headers),
                data: None,
                params: Some(params),
            }),
        )
        .await?;
        Ok(())
    }

    pub async fn get_balance_allowance(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<crate::types::BalanceAllowanceResponse, ClobError> {
        if self.creds.is_none() {
            return Err(ClobError::Other("L2 creds required".to_string()));
        }
        let signer_arc = self
            .signer
            .as_ref()
            .ok_or(ClobError::Other("L1 signer required".to_string()))?;
        let signer_ref: &EthersSigner = signer_arc.as_ref();
        let ts = if self.use_server_time {
            Some(self.get_server_time().await?)
        } else {
            None
        };
        let headers = crate::headers::create_l2_headers(
            signer_ref,
            self.creds.as_ref().unwrap(),
            "GET",
            GET_BALANCE_ALLOWANCE,
            None,
            ts,
        )
        .await?;
        let resp: crate::types::BalanceAllowanceResponse = crate::http_helpers::get_typed(
            &format!("{}{}", self.host, GET_BALANCE_ALLOWANCE),
            Some(RequestOptions::<Value> {
                headers: Some(headers),
                data: None,
                params,
            }),
        )
        .await?;
        Ok(resp)
    }

    pub async fn update_balance_allowance(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<(), ClobError> {
        if self.creds.is_none() {
            return Err(ClobError::Other("L2 creds required".to_string()));
        }
        let signer_arc = self
            .signer
            .as_ref()
            .ok_or(ClobError::Other("L1 signer required".to_string()))?;
        let signer_ref: &EthersSigner = signer_arc.as_ref();
        let ts = if self.use_server_time {
            Some(self.get_server_time().await?)
        } else {
            None
        };
        let headers = crate::headers::create_l2_headers(
            signer_ref,
            self.creds.as_ref().unwrap(),
            "GET",
            UPDATE_BALANCE_ALLOWANCE,
            None,
            ts,
        )
        .await?;
        // API returns no useful body for update; call and ignore body
        let _resp: OkResp = crate::http_helpers::get_typed(
            &format!("{}{}", self.host, UPDATE_BALANCE_ALLOWANCE),
            Some(RequestOptions::<Value> {
                headers: Some(headers),
                data: None,
                params,
            }),
        )
        .await?;
        Ok(())
    }

    /// Typed helper to fetch user rewards for a day (deserializes into `Reward`).
    /// This endpoint may be public or require params depending on server; we accept optional query params.
    pub async fn get_rewards_user_for_day_typed(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<Reward>, ClobError> {
        let val = crate::http_helpers::get(
            &format!("{}{}", self.host, GET_EARNINGS_FOR_USER_FOR_DAY),
            Some(RequestOptions {
                headers: None,
                data: None,
                params,
            }),
        )
        .await?;

        let arr = if let Some(a) = val.get("data").and_then(|v| v.as_array()) {
            a.clone()
        } else if val.is_array() {
            val.as_array().cloned().unwrap_or_default()
        } else {
            Vec::new()
        };
        let mut out: Vec<Reward> = Vec::new();
        for v in arr {
            let r: Reward =
                serde_json::from_value(v).map_err(|e| ClobError::Other(e.to_string()))?;
            out.push(r);
        }
        Ok(out)
    }

    pub async fn create_order(
        &self,
        user_order: UserOrder,
        options_tick: Option<&str>,
    ) -> Result<SignedOrder, ClobError> {
        // L1 auth required
        self.can_l1_auth()?;
        let signer_arc = self
            .signer
            .as_ref()
            .ok_or(ClobError::Other("L1 signer required".to_string()))?;
        let signer_ref: &EthersSigner = signer_arc.as_ref();
        let ob = if let Some(cfg) = &self.builder_config {
            OrderBuilder::with_config(signer_ref, self.chain_id, cfg)
        } else {
            OrderBuilder::new(signer_ref, self.chain_id, None, None)
        };
        // 动态解析合约地址：根据 chainId 与 neg_risk(token) 选择标准或 negRisk 交易所
        let exchange_addr = self.resolve_exchange_address(&user_order.token_id);
        let signed = ob
            .build_order(&exchange_addr, &user_order, options_tick.unwrap_or("0.01"))
            .await?;
        Ok(signed)
    }

    pub async fn create_market_order(
        &self,
        user_market_order: UserMarketOrder,
        options_tick: Option<&str>,
    ) -> Result<SignedOrder, ClobError> {
        self.can_l1_auth()?;
        let signer_arc = self
            .signer
            .as_ref()
            .ok_or(ClobError::Other("L1 signer required".to_string()))?;
        let signer_ref: &EthersSigner = signer_arc.as_ref();
        let ob = if let Some(cfg) = &self.builder_config {
            OrderBuilder::with_config(signer_ref, self.chain_id, cfg)
        } else {
            OrderBuilder::new(signer_ref, self.chain_id, None, None)
        };
        let exchange_addr = self.resolve_exchange_address(&user_market_order.token_id);
        let signed = ob
            .build_market_order(
                &exchange_addr,
                &user_market_order,
                options_tick.unwrap_or("0.01"),
            )
            .await?;
        Ok(signed)
    }

    /// 依据链 ID 与 builder_config.neg_risk 选择 verifyingContract 地址。
    /// 注意：当前实现不主动查询 token neg_risk，只使用调用方传入的配置；如需自动检测可在上层先调用 get_neg_risk 并写入 builder_config。
    fn resolve_exchange_address(&self, _token_id: &str) -> String {
        let neg = self
            .builder_config
            .as_ref()
            .and_then(|c| c.neg_risk)
            .unwrap_or(false);
        match (self.chain_id, neg) {
            (137, false) => "0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E".to_string(),
            (137, true) => "0xC5d563A36AE78145C45a50134d48A1215220f80a".to_string(),
            (80002, false) => "0xdFE02Eb6733538f8Ea35D585af8DE5958AD99E40".to_string(),
            (80002, true) => "0xC5d563A36AE78145C45a50134d48A1215220f80a".to_string(),
            (_other, _) => "0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E".to_string(),
        }
    }

    // removed unused builder-auth helpers; builder headers are injected inline where needed

    pub async fn post_orders(
        &self,
        args: Vec<SignedOrder>,
        _defer_exec: bool,
    ) -> Result<Vec<Order>, ClobError> {
        // strong typed version returning parsed orders
        if self.creds.is_none() {
            return Err(ClobError::Other("L2 creds required".to_string()));
        }
        let signer_arc = self
            .signer
            .as_ref()
            .ok_or(ClobError::Other("L1 signer required".to_string()))?;
        let signer_ref: &EthersSigner = signer_arc.as_ref();
        let body_str = serde_json::to_string(&args).map_err(|e| ClobError::Other(e.to_string()))?;
        // 为了与 TypeScript SDK 保持完全一致的时间戳使用策略，这里在开启 use_server_time 时获取服务器时间用于 L2 HMAC
        let ts = if self.use_server_time {
            Some(self.get_server_time().await?)
        } else {
            None
        };
        let mut headers = crate::headers::create_l2_headers(
            signer_ref,
            self.creds.as_ref().unwrap(),
            "POST",
            POST_ORDERS,
            Some(&body_str),
            ts,
        )
        .await?;
        if let Some(b) = &self.builder_signer {
            let b_payload = b
                .create_builder_header_payload("POST", POST_ORDERS, Some(&body_str), None)
                .map_err(|e| ClobError::Other(format!("builder header error: {}", e)))?;
            headers = crate::headers::inject_builder_headers(headers, &b_payload);
        }
        let endpoint = format!("{}{}", self.host, POST_ORDERS);
        let raw: MaybeVec<Order> = crate::http_helpers::post_typed(
            &endpoint,
            Some(RequestOptions {
                headers: Some(headers),
                data: Some(args),
                params: None,
            }),
        )
        .await?;
        Ok(raw.into_vec())
    }

    /// Helper: post a single SignedOrder (typed) to the API. Wraps SignedOrder in NewOrder
    /// with orderType and performs L2-authenticated POST to POST_ORDER endpoint.
    pub async fn post_signed_order(
        &self,
        signed_order: &SignedOrder,
        order_type: OrderType,
        defer_exec: bool,
    ) -> Result<OrderResponse, ClobError> {
        // build headers and post, then parse into Order
        if self.creds.is_none() {
            return Err(ClobError::Other("L2 creds required".to_string()));
        }
        let signer_arc = self
            .signer
            .as_ref()
            .ok_or(ClobError::Other("L1 signer required".to_string()))?;
        let signer_ref: &EthersSigner = signer_arc.as_ref();

        // IMPORTANT: Use API key as owner, NOT wallet address
        // This matches TypeScript SDK behavior: orderToJson(order, this.creds?.key || "", ...)
        let owner = self.creds.as_ref().unwrap().key.clone();

        // Convert SignedOrder to NewOrder format
        let new_order =
            crate::utilities::order_to_json(signed_order, &owner, order_type, defer_exec);

        let body_str =
            serde_json::to_string(&new_order).map_err(|e| ClobError::Other(e.to_string()))?;
        // 使用服务器时间保证与服务器 HMAC 计算节奏一致
        let ts = if self.use_server_time {
            Some(self.get_server_time().await?)
        } else {
            None
        };
        let mut headers = crate::headers::create_l2_headers(
            signer_ref,
            self.creds.as_ref().unwrap(),
            "POST",
            POST_ORDER,
            Some(&body_str),
            ts,
        )
        .await?;
        // Inject builder headers if builder auth configured
        if let Some(b) = &self.builder_signer {
            let b_payload = b
                .create_builder_header_payload("POST", POST_ORDER, Some(&body_str), None)
                .map_err(|e| ClobError::Other(format!("builder header error: {}", e)))?;
            headers = crate::headers::inject_builder_headers(headers, &b_payload);
        }
        let endpoint = format!("{}{}", self.host, POST_ORDER);
        let opts = RequestOptions {
            headers: Some(headers),
            data: Some(new_order),
            params: None,
        };
        let res: MaybeItem<OrderResponse> =
            crate::http_helpers::post_typed(&endpoint, Some(opts)).await?;
        Ok(res.into_item())
    }

    /// Typed variant of posting a signed order: posts the signed order and attempts to
    /// deserialize the response into an `OrderResponse` (or into the `data` field if present).
    pub async fn post_signed_order_typed(
        &self,
        signed_order: &SignedOrder,
        order_type: OrderType,
        defer_exec: bool,
    ) -> Result<OrderResponse, ClobError> {
        self.post_signed_order(signed_order, order_type, defer_exec)
            .await
    }

    /// Convenience: create (build & sign) then immediately post a limit order.
    /// orderType defaults to GTC (Good Till Cancelled).
    pub async fn create_and_post_order(
        &mut self,
        user_order: UserOrder,
        options_tick: Option<&str>,
        order_type: Option<OrderType>,
    ) -> Result<OrderResponse, ClobError> {
        // 避免在创建阶段发起额外 HTTP: 优先使用调用方提供的 tick 或 builder_config 中的 tick，否则使用默认值
        let tick = if let Some(t) = options_tick {
            t.to_string()
        } else if let Some(cfg_tick) = self
            .builder_config
            .as_ref()
            .and_then(|c| c.tick_size.as_ref())
        {
            cfg_tick.clone()
        } else {
            "0.01".to_string()
        };
        let signed = self.create_order(user_order, Some(&tick)).await?;
        let order_type = order_type.unwrap_or(OrderType::GTC);
        self.post_signed_order(&signed, order_type, false).await
    }

    /// Convenience: create (build & sign) then immediately post a market order.
    /// orderType defaults to FOK (Fill Or Kill).
    pub async fn create_and_post_market_order(
        &mut self,
        user_market_order: UserMarketOrder,
        options_tick: Option<&str>,
        order_type: Option<OrderType>,
    ) -> Result<OrderResponse, ClobError> {
        // 避免在创建阶段发起额外 HTTP: 同上
        let tick = if let Some(t) = options_tick {
            t.to_string()
        } else if let Some(cfg_tick) = self
            .builder_config
            .as_ref()
            .and_then(|c| c.tick_size.as_ref())
        {
            cfg_tick.clone()
        } else {
            "0.01".to_string()
        };
        let signed = self
            .create_market_order(user_market_order, Some(&tick))
            .await?;
        let order_type = order_type.unwrap_or(OrderType::FOK);
        self.post_signed_order(&signed, order_type, false).await
    }

    // 已移除内部 tick size 解析逻辑（resolve_tick/get_tick_size_uncached）以避免隐式网络请求；保留显式 get_tick_size API。
    /// Helper: accept typed SignedOrder list and post them to POST_ORDERS endpoint.
    pub async fn post_orders_typed(
        &self,
        orders: Vec<SignedOrder>,
        _defer_exec: bool,
    ) -> Result<Value, ClobError> {
        if self.creds.is_none() {
            return Err(ClobError::Other("L2 creds required".to_string()));
        }
        let signer_arc = self
            .signer
            .as_ref()
            .ok_or(ClobError::Other("L1 signer required".to_string()))?;
        let signer_ref: &EthersSigner = signer_arc.as_ref();
        let body_str =
            serde_json::to_string(&orders).map_err(|e| ClobError::Other(e.to_string()))?;
        let ts = if self.use_server_time {
            Some(self.get_server_time().await?)
        } else {
            None
        };
        let headers = crate::headers::create_l2_headers(
            signer_ref,
            self.creds.as_ref().unwrap(),
            "POST",
            POST_ORDERS,
            Some(&body_str),
            ts,
        )
        .await?;
        let headers = if let Some(b) = &self.builder_signer {
            let b_payload = b
                .create_builder_header_payload("POST", POST_ORDERS, Some(&body_str), None)
                .map_err(|e| ClobError::Other(format!("builder header error: {}", e)))?;
            crate::headers::inject_builder_headers(headers, &b_payload)
        } else {
            headers
        };
        let endpoint = format!("{}{}", self.host, POST_ORDERS);
        let res = crate::http_helpers::post_typed::<Value, Vec<SignedOrder>>(
            &endpoint,
            Some(RequestOptions {
                headers: Some(headers),
                data: Some(orders),
                params: None,
            }),
        )
        .await?;
        Ok(res)
    }

    /// Post typed orders and return parsed `Vec<Order>` from the response.
    /// This wraps the existing `post_orders_typed` which returns a raw JSON Value
    /// and attempts to parse either a top-level array or `data` field into `Vec<Order>`.
    pub async fn post_orders_typed_parsed(
        &self,
        orders: Vec<SignedOrder>,
        _defer_exec: bool,
    ) -> Result<Vec<Order>, ClobError> {
        // Build body and headers similarly to post_orders_typed, but use the typed http helper
        if self.creds.is_none() {
            return Err(ClobError::Other("L2 creds required".to_string()));
        }
        let signer_arc = self
            .signer
            .as_ref()
            .ok_or(ClobError::Other("L1 signer required".to_string()))?;
        let signer_ref: &EthersSigner = signer_arc.as_ref();
        let body_str =
            serde_json::to_string(&orders).map_err(|e| ClobError::Other(e.to_string()))?;
        let ts = if self.use_server_time {
            Some(self.get_server_time().await?)
        } else {
            None
        };
        let headers = crate::headers::create_l2_headers(
            signer_ref,
            self.creds.as_ref().unwrap(),
            "POST",
            POST_ORDERS,
            Some(&body_str),
            ts,
        )
        .await?;
        let headers = if let Some(b) = &self.builder_signer {
            let b_payload = b
                .create_builder_header_payload("POST", POST_ORDERS, Some(&body_str), None)
                .map_err(|e| ClobError::Other(format!("builder header error: {}", e)))?;
            crate::headers::inject_builder_headers(headers, &b_payload)
        } else {
            headers
        };
        let endpoint = format!("{}{}", self.host, POST_ORDERS);
        let raw: MaybeVec<Order> = crate::http_helpers::post_typed(
            &endpoint,
            Some(RequestOptions {
                headers: Some(headers),
                data: Some(orders),
                params: None,
            }),
        )
        .await?;
        Ok(raw.into_vec())
    }

    pub async fn cancel_all(&self) -> Result<Vec<Order>, ClobError> {
        if self.creds.is_none() {
            return Err(ClobError::Other("L2 creds required".to_string()));
        }
        let signer_arc = self
            .signer
            .as_ref()
            .ok_or(ClobError::Other("L1 signer required".to_string()))?;
        let signer_ref: &EthersSigner = signer_arc.as_ref();
        let ts = if self.use_server_time {
            Some(self.get_server_time().await?)
        } else {
            None
        };
        let mut headers = crate::headers::create_l2_headers(
            signer_ref,
            self.creds.as_ref().unwrap(),
            "DELETE",
            CANCEL_ALL,
            None,
            ts,
        )
        .await?;
        if let Some(b) = &self.builder_signer {
            let b_payload = b
                .create_builder_header_payload("DELETE", CANCEL_ALL, None, None)
                .map_err(|e| ClobError::Other(format!("builder header error: {}", e)))?;
            headers = crate::headers::inject_builder_headers(headers, &b_payload);
        }
        let endpoint = format!("{}{}", self.host, CANCEL_ALL);
        let raw: MaybeVec<Order> = crate::http_helpers::del_typed(
            &endpoint,
            Some(RequestOptions::<Value> {
                headers: Some(headers),
                data: None,
                params: None,
            }),
        )
        .await?;
        Ok(raw.into_vec())
    }

    pub async fn cancel_market_orders(
        &self,
        order_ids: Vec<String>,
    ) -> Result<Vec<Order>, ClobError> {
        if self.creds.is_none() {
            return Err(ClobError::Other("L2 creds required".to_string()));
        }
        let signer_arc = self
            .signer
            .as_ref()
            .ok_or(ClobError::Other("L1 signer required".to_string()))?;
        let signer_ref: &EthersSigner = signer_arc.as_ref();
        let body_str =
            serde_json::to_string(&order_ids).map_err(|e| ClobError::Other(e.to_string()))?;
        let ts = if self.use_server_time {
            Some(self.get_server_time().await?)
        } else {
            None
        };
        let mut headers = crate::headers::create_l2_headers(
            signer_ref,
            self.creds.as_ref().unwrap(),
            "DELETE",
            CANCEL_MARKET_ORDERS,
            Some(&body_str),
            ts,
        )
        .await?;
        if let Some(b) = &self.builder_signer {
            let b_payload = b
                .create_builder_header_payload(
                    "DELETE",
                    CANCEL_MARKET_ORDERS,
                    Some(&body_str),
                    None,
                )
                .map_err(|e| ClobError::Other(format!("builder header error: {}", e)))?;
            headers = crate::headers::inject_builder_headers(headers, &b_payload);
        }
        let endpoint = format!("{}{}", self.host, CANCEL_MARKET_ORDERS);
        let raw: MaybeVec<Order> = crate::http_helpers::del_typed(
            &endpoint,
            Some(RequestOptions {
                headers: Some(headers),
                data: Some(order_ids),
                params: None,
            }),
        )
        .await?;
        Ok(raw.into_vec())
    }

    pub async fn is_order_scoring(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<crate::types::OrderScoring, ClobError> {
        if self.creds.is_none() {
            return Err(ClobError::Other("L2 creds required".to_string()));
        }
        let signer_arc = self
            .signer
            .as_ref()
            .ok_or(ClobError::Other("L1 signer required".to_string()))?;
        let signer_ref: &EthersSigner = signer_arc.as_ref();
        let mut headers = crate::headers::create_l2_headers(
            signer_ref,
            self.creds.as_ref().unwrap(),
            "GET",
            IS_ORDER_SCORING,
            None,
            if self.use_server_time {
                Some(self.get_server_time().await?)
            } else {
                None
            },
        )
        .await?;
        if let Some(b) = &self.builder_signer {
            let b_payload = b
                .create_builder_header_payload("GET", IS_ORDER_SCORING, None, None)
                .map_err(|e| ClobError::Other(format!("builder header error: {}", e)))?;
            headers = crate::headers::inject_builder_headers(headers, &b_payload);
        }
        let resp: crate::types::OrderScoring = crate::http_helpers::get_typed(
            &format!("{}{}", self.host, IS_ORDER_SCORING),
            Some(RequestOptions::<Value> {
                headers: Some(headers),
                data: None,
                params,
            }),
        )
        .await?;
        Ok(resp)
    }

    pub async fn are_orders_scoring(
        &self,
        order_ids: Option<Vec<String>>,
    ) -> Result<crate::types::OrdersScoring, ClobError> {
        if self.creds.is_none() {
            return Err(ClobError::Other("L2 creds required".to_string()));
        }
        let signer_arc = self
            .signer
            .as_ref()
            .ok_or(ClobError::Other("L1 signer required".to_string()))?;
        let signer_ref: &EthersSigner = signer_arc.as_ref();
        let body_str =
            serde_json::to_string(&order_ids).map_err(|e| ClobError::Other(e.to_string()))?;
        let ts = if self.use_server_time {
            Some(self.get_server_time().await?)
        } else {
            None
        };
        let mut headers = crate::headers::create_l2_headers(
            signer_ref,
            self.creds.as_ref().unwrap(),
            "POST",
            ARE_ORDERS_SCORING,
            Some(&body_str),
            ts,
        )
        .await?;
        if let Some(b) = &self.builder_signer {
            let b_payload = b
                .create_builder_header_payload("POST", ARE_ORDERS_SCORING, Some(&body_str), None)
                .map_err(|e| ClobError::Other(format!("builder header error: {}", e)))?;
            headers = crate::headers::inject_builder_headers(headers, &b_payload);
        }
        // Serialize optional ids to JSON Value (null or array)
        let body_json =
            serde_json::to_value(&order_ids).map_err(|e| ClobError::Other(e.to_string()))?;
        let resp: crate::types::OrdersScoring = crate::http_helpers::post_typed(
            &format!("{}{}", self.host, ARE_ORDERS_SCORING),
            Some(RequestOptions::<Value> {
                headers: Some(headers),
                data: Some(body_json),
                params: None,
            }),
        )
        .await?;
        Ok(resp)
    }

    pub async fn cancel_order(&self, order_id: &str) -> Result<Order, ClobError> {
        if self.creds.is_none() {
            return Err(ClobError::Other("L2 creds required".to_string()));
        }
        let creds = self.creds.as_ref().unwrap();
        let signer_arc = self
            .signer
            .as_ref()
            .ok_or(ClobError::Other("L1 signer required".to_string()))?;
        let signer_ref: &EthersSigner = signer_arc.as_ref();
        let mut params = std::collections::HashMap::new();
        params.insert("order_id".to_string(), order_id.to_string());
        let mut headers = crate::headers::create_l2_headers(
            signer_ref,
            creds,
            "DELETE",
            CANCEL_ORDER,
            None,
            if self.use_server_time {
                Some(self.get_server_time().await?)
            } else {
                None
            },
        )
        .await?;
        if let Some(b) = &self.builder_signer {
            let b_payload = b
                .create_builder_header_payload("DELETE", CANCEL_ORDER, None, None)
                .map_err(|e| ClobError::Other(format!("builder header error: {}", e)))?;
            headers = crate::headers::inject_builder_headers(headers, &b_payload);
        }
        let endpoint = format!("{}{}", self.host, CANCEL_ORDER);
        let opts: RequestOptions<Value> = RequestOptions::<Value> {
            headers: Some(headers),
            data: None,
            params: Some(params),
        };
        let raw: MaybeItem<Order> = crate::http_helpers::del_typed(&endpoint, Some(opts)).await?;
        let o = raw.into_item();
        Ok(o)
    }

    pub async fn cancel_orders(&self, order_ids: Vec<String>) -> Result<Vec<Order>, ClobError> {
        if self.creds.is_none() {
            return Err(ClobError::Other("L2 creds required".to_string()));
        }
        let creds = self.creds.as_ref().unwrap();
        let signer_arc = self
            .signer
            .as_ref()
            .ok_or(ClobError::Other("L1 signer required".to_string()))?;
        let signer_ref: &EthersSigner = signer_arc.as_ref();
        let body_str =
            serde_json::to_string(&order_ids).map_err(|e| ClobError::Other(e.to_string()))?;
        let ts = if self.use_server_time {
            Some(self.get_server_time().await?)
        } else {
            None
        };
        let mut headers = crate::headers::create_l2_headers(
            signer_ref,
            creds,
            "POST",
            CANCEL_ORDERS,
            Some(&body_str),
            ts,
        )
        .await?;
        if let Some(b) = &self.builder_signer {
            let b_payload = b
                .create_builder_header_payload("POST", CANCEL_ORDERS, Some(&body_str), None)
                .map_err(|e| ClobError::Other(format!("builder header error: {}", e)))?;
            headers = crate::headers::inject_builder_headers(headers, &b_payload);
        }
        let endpoint = format!("{}{}", self.host, CANCEL_ORDERS);
        let opts = RequestOptions {
            headers: Some(headers),
            data: Some(order_ids),
            params: None,
        };
        let raw: MaybeVec<Order> = crate::http_helpers::post_typed(&endpoint, Some(opts)).await?;
        Ok(raw.into_vec())
    }

    pub async fn get_order(&self, order_id: &str) -> Result<OpenOrder, ClobError> {
        // 与 TypeScript SDK 保持一致：执行带 L2 鉴权的调用
        self.get_order_typed(order_id).await
    }

    /// Typed variant: try to deserialize an order response into `OpenOrder`.
    pub async fn get_order_typed(&self, order_id: &str) -> Result<OpenOrder, ClobError> {
        // TS SDK 行为：必须 L2 鉴权（canL2Auth + createL2Headers），useServerTime 时使用服务器时间戳参与 HMAC
        if self.creds.is_none() {
            return Err(ClobError::Other("L2 creds required".to_string()));
        }
        let signer_arc = self
            .signer
            .as_ref()
            .ok_or(ClobError::Other("L1 signer required".to_string()))?;
        let signer_ref: &EthersSigner = signer_arc.as_ref();
        // requestPath 需要包含具体 /orders/{id}，与 TS 保持完全一致
        let request_path = format!("{}{}", GET_ORDER, order_id);
        let endpoint = format!("{}{}", self.host, request_path);
        let ts = if self.use_server_time {
            Some(self.get_server_time().await?)
        } else {
            None
        };
        let headers = crate::headers::create_l2_headers(
            signer_ref,
            self.creds.as_ref().unwrap(),
            "GET",
            &request_path,
            None,
            ts,
        )
        .await?;
        let opts = RequestOptions {
            headers: Some(headers),
            data: None,
            params: None,
        };
        let val = get(&endpoint, Some(opts)).await?;
        // API may return object or { data: object }
        if val.is_object() && val.get("id").is_some() {
            let o: OpenOrder =
                serde_json::from_value(val).map_err(|e| ClobError::Other(e.to_string()))?;
            Ok(o)
        } else if let Some(d) = val.get("data") {
            let o: OpenOrder =
                serde_json::from_value(d.clone()).map_err(|e| ClobError::Other(e.to_string()))?;
            Ok(o)
        } else {
            Err(ClobError::Other(
                "unexpected order response shape".to_string(),
            ))
        }
    }

    pub async fn get_open_orders(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<SignedOrder>, ClobError> {
        // Delegate to typed variant
        self.get_open_orders_typed(params).await
    }

    /// Typed variant: try to deserialize open orders `data` into Vec<SignedOrder>`.
    pub async fn get_open_orders_typed(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<SignedOrder>, ClobError> {
        let endpoint = format!("{}{}", self.host, GET_OPEN_ORDERS);
        let opts = RequestOptions {
            headers: None,
            data: None,
            params,
        };
        let val = get(&endpoint, Some(opts)).await?;
        // API might return { data: [...] } or an array directly
        let arr = if val.is_array() {
            val
        } else if let Some(d) = val.get("data") {
            d.clone()
        } else {
            return Err(ClobError::Other(
                "unexpected open orders response shape".to_string(),
            ));
        };
        let orders: Vec<SignedOrder> =
            serde_json::from_value(arr).map_err(|e| ClobError::Other(e.to_string()))?;
        Ok(orders)
    }

    pub async fn get_markets(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Market>, ClobError> {
        let endpoint = format!("{}{}", self.host, GET_MARKETS);
        let opts = RequestOptions {
            headers: None,
            data: None,
            params,
        };
        let val = get(&endpoint, Some(opts)).await?;
        // API may return an array or an object containing `data: [...]`
        let arr = if val.is_array() {
            val
        } else if let Some(d) = val.get("data") {
            d.clone()
        } else {
            return Err(ClobError::Other(
                "unexpected markets response shape".to_string(),
            ));
        };
        let markets: Vec<crate::types::Market> =
            serde_json::from_value(arr).map_err(|e| ClobError::Other(e.to_string()))?;
        Ok(markets)
    }

    pub async fn get_market(
        &self,
        market_id: &str,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<crate::types::MarketSummary, ClobError> {
        let endpoint = format!("{}{}{}", self.host, GET_MARKET, market_id);
        let opts = RequestOptions {
            headers: None,
            data: None,
            params,
        };
        let val = get(&endpoint, Some(opts)).await?;
        // Try direct deserialization into MarketSummary or unwrap `data` field
        if val.get("market").is_some() {
            let m: crate::types::MarketSummary =
                serde_json::from_value(val).map_err(|e| ClobError::Other(e.to_string()))?;
            Ok(m)
        } else if let Some(d) = val.get("data") {
            let m: crate::types::MarketSummary =
                serde_json::from_value(d.clone()).map_err(|e| ClobError::Other(e.to_string()))?;
            Ok(m)
        } else {
            Err(ClobError::Other(
                "unexpected market response shape".to_string(),
            ))
        }
    }

    pub async fn get_simplified_markets(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Market>, ClobError> {
        let endpoint = format!("{}{}", self.host, GET_SIMPLIFIED_MARKETS);
        let opts = RequestOptions {
            headers: None,
            data: None,
            params,
        };
        let val = get(&endpoint, Some(opts)).await?;
        let arr = if val.is_array() {
            val
        } else if let Some(d) = val.get("data") {
            d.clone()
        } else {
            return Err(ClobError::Other(
                "unexpected simplified markets response shape".to_string(),
            ));
        };
        let markets: Vec<crate::types::Market> =
            serde_json::from_value(arr).map_err(|e| ClobError::Other(e.to_string()))?;
        Ok(markets)
    }

    pub async fn get_sampling_markets(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Market>, ClobError> {
        let endpoint = format!("{}{}", self.host, GET_SAMPLING_MARKETS);
        let opts = RequestOptions {
            headers: None,
            data: None,
            params,
        };
        let val = get(&endpoint, Some(opts)).await?;
        let arr = if val.is_array() {
            val
        } else if let Some(d) = val.get("data") {
            d.clone()
        } else {
            return Err(ClobError::Other(
                "unexpected sampling markets response shape".to_string(),
            ));
        };
        let markets: Vec<crate::types::Market> =
            serde_json::from_value(arr).map_err(|e| ClobError::Other(e.to_string()))?;
        Ok(markets)
    }

    pub async fn get_server_time(&self) -> Result<u64, ClobError> {
        let endpoint = format!("{}{}", self.host, TIME);
        let val = get(&endpoint, None).await?;
        // Expect number or object with `time`
        if val.is_number() {
            Ok(val
                .as_u64()
                .ok_or(ClobError::Other("invalid time value".to_string()))?)
        } else if val.get("time").is_some() {
            Ok(val
                .get("time")
                .and_then(|v| v.as_u64())
                .ok_or(ClobError::Other("invalid time value".to_string()))?)
        } else {
            Err(ClobError::Other(
                "unexpected server time response".to_string(),
            ))
        }
    }

    fn can_l1_auth(&self) -> Result<(), ClobError> {
        if self.signer.is_none() {
            return Err(ClobError::Other("L1 auth required".to_string()));
        }
        Ok(())
    }

    pub async fn create_api_key(&self, nonce: Option<u64>) -> Result<ApiKeyCreds, ClobError> {
        self.can_l1_auth()?;
        let signer_arc = self.signer.as_ref().unwrap();
        let signer_ref: &EthersSigner = signer_arc.as_ref();
        let ts = if self.use_server_time {
            Some(self.get_server_time().await?)
        } else {
            None
        };
        let headers =
            crate::headers::create_l1_headers(signer_ref, self.chain_id as i32, nonce, ts).await?;
        let endpoint = format!("{}{}", self.host, CREATE_API_KEY);
        let opts = RequestOptions {
            headers: Some(headers),
            data: None,
            params: None,
        };
        let val = post(&endpoint, Some(opts)).await?;
        // Deserialize into ApiKeyRaw then to ApiKeyCreds
        let api_raw: ApiKeyRaw =
            serde_json::from_value(val).map_err(|e| ClobError::Other(e.to_string()))?;
        let api_key = ApiKeyCreds {
            key: api_raw.api_key,
            secret: api_raw.secret,
            passphrase: api_raw.passphrase,
        };
        Ok(api_key)
    }

    // Additional endpoints ported from TypeScript client
    pub async fn derive_api_key(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<ApiKeyCreds, ClobError> {
        self.can_l1_auth()?;
        let signer_arc = self.signer.as_ref().unwrap();
        let signer_ref: &EthersSigner = signer_arc.as_ref();
        let ts = if self.use_server_time {
            Some(self.get_server_time().await?)
        } else {
            None
        };
        let headers =
            crate::headers::create_l1_headers(signer_ref, self.chain_id as i32, None, ts).await?;
        let endpoint = format!("{}{}", self.host, DERIVE_API_KEY);
        let opts = RequestOptions {
            headers: Some(headers),
            data: None,
            params,
        };
        let val = get(&endpoint, Some(opts)).await?;
        // Deserialize ApiKeyRaw then map to ApiKeyCreds
        let api_raw: ApiKeyRaw =
            serde_json::from_value(val).map_err(|e| ClobError::Other(e.to_string()))?;
        let api_key = ApiKeyCreds {
            key: api_raw.api_key,
            secret: api_raw.secret,
            passphrase: api_raw.passphrase,
        };
        Ok(api_key)
    }

    pub async fn create_builder_api_key(&self) -> Result<crate::types::ApiKeyCreds, ClobError> {
        self.can_l1_auth()?;
        let signer_arc = self.signer.as_ref().unwrap();
        let signer_ref: &EthersSigner = signer_arc.as_ref();
        let headers =
            crate::headers::create_l1_headers(signer_ref, self.chain_id as i32, None, None).await?;
        let endpoint = format!("{}{}", self.host, CREATE_BUILDER_API_KEY);
        let resp: crate::types::ApiKeyCreds = crate::http_helpers::post_typed(
            &endpoint,
            Some(RequestOptions::<Value> {
                headers: Some(headers),
                data: None,
                params: None,
            }),
        )
        .await?;
        Ok(resp)
    }

    pub async fn get_builder_api_keys(&self) -> Result<Vec<crate::types::ApiKeyCreds>, ClobError> {
        self.can_l1_auth()?;
        let signer_arc = self.signer.as_ref().unwrap();
        let signer_ref: &EthersSigner = signer_arc.as_ref();
        let headers =
            crate::headers::create_l1_headers(signer_ref, self.chain_id as i32, None, None).await?;
        let endpoint = format!("{}{}", self.host, GET_BUILDER_API_KEYS);
        let resp: crate::types::ApiKeysResponse = crate::http_helpers::get_typed(
            &endpoint,
            Some(RequestOptions::<Value> {
                headers: Some(headers),
                data: None,
                params: None,
            }),
        )
        .await?;
        Ok(resp.api_keys)
    }

    pub async fn revoke_builder_api_key(&self, id: &str) -> Result<(), ClobError> {
        self.can_l1_auth()?;
        let signer_arc = self.signer.as_ref().unwrap();
        let signer_ref: &EthersSigner = signer_arc.as_ref();
        let headers =
            crate::headers::create_l1_headers(signer_ref, self.chain_id as i32, None, None).await?;
        let endpoint = format!("{}{}", self.host, REVOKE_BUILDER_API_KEY);
        let mut params = std::collections::HashMap::new();
        params.insert("id".to_string(), id.to_string());
        let _val: () = crate::http_helpers::del_typed::<(), Value>(
            &endpoint,
            Some(RequestOptions::<Value> {
                headers: Some(headers),
                data: None,
                params: Some(params),
            }),
        )
        .await?;
        Ok(())
    }

    pub async fn get_sampling_simplified_markets(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Market>, ClobError> {
        let endpoint = format!("{}{}", self.host, GET_SAMPLING_SIMPLIFIED_MARKETS);
        let opts = RequestOptions {
            headers: None,
            data: None,
            params,
        };
        let val = get(&endpoint, Some(opts)).await?;
        let arr = if val.is_object() && val.get("data").is_some() {
            val.get("data").cloned().unwrap_or_default()
        } else if val.is_array() {
            val
        } else {
            return Err(ClobError::Other(
                "unexpected sampling simplified markets response shape".to_string(),
            ));
        };
        let markets: Vec<crate::types::Market> =
            serde_json::from_value(arr).map_err(|e| ClobError::Other(e.to_string()))?;
        Ok(markets)
    }

    pub async fn get_order_books(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::OrderBookSummary>, ClobError> {
        let endpoint = format!("{}{}", self.host, GET_ORDER_BOOKS);
        let opts = RequestOptions {
            headers: None,
            data: None,
            params,
        };
        let val = get(&endpoint, Some(opts)).await?;
        let arr = if val.is_array() {
            val
        } else if let Some(d) = val.get("data") {
            d.clone()
        } else {
            return Err(ClobError::Other(
                "unexpected order books response shape".to_string(),
            ));
        };
        let books: Vec<crate::types::OrderBookSummary> =
            serde_json::from_value(arr).map_err(|e| ClobError::Other(e.to_string()))?;
        Ok(books)
    }

    pub async fn get_midpoint(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::MarketPrice>, ClobError> {
        let endpoint = format!("{}{}", self.host, GET_MIDPOINT);
        let opts = RequestOptions {
            headers: None,
            data: None,
            params,
        };
        let val = get(&endpoint, Some(opts)).await?;
        let arr = if val.is_array() {
            val
        } else if let Some(d) = val.get("data") {
            d.clone()
        } else {
            return Err(ClobError::Other(
                "unexpected midpoint response shape".to_string(),
            ));
        };
        let prices: Vec<crate::types::MarketPrice> =
            serde_json::from_value(arr).map_err(|e| ClobError::Other(e.to_string()))?;
        Ok(prices)
    }

    pub async fn get_midpoints(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::MarketPrice>, ClobError> {
        let endpoint = format!("{}{}", self.host, GET_MIDPOINTS);
        let opts = RequestOptions {
            headers: None,
            data: None,
            params,
        };
        let val = get(&endpoint, Some(opts)).await?;
        let arr = if val.is_array() {
            val
        } else if let Some(d) = val.get("data") {
            d.clone()
        } else {
            return Err(ClobError::Other(
                "unexpected midpoints response shape".to_string(),
            ));
        };
        let prices: Vec<crate::types::MarketPrice> =
            serde_json::from_value(arr).map_err(|e| ClobError::Other(e.to_string()))?;
        Ok(prices)
    }

    pub async fn get_prices(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::MarketPrice>, ClobError> {
        let endpoint = format!("{}{}", self.host, GET_PRICES);
        let opts = RequestOptions {
            headers: None,
            data: None,
            params,
        };
        let val = get(&endpoint, Some(opts)).await?;
        let arr = if val.is_array() {
            val
        } else if let Some(d) = val.get("data") {
            d.clone()
        } else {
            return Err(ClobError::Other(
                "unexpected prices response shape".to_string(),
            ));
        };
        let prices: Vec<crate::types::MarketPrice> =
            serde_json::from_value(arr).map_err(|e| ClobError::Other(e.to_string()))?;
        Ok(prices)
    }

    pub async fn get_spreads(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::MarketPrice>, ClobError> {
        let endpoint = format!("{}{}", self.host, GET_SPREADS);
        let opts = RequestOptions {
            headers: None,
            data: None,
            params,
        };
        let val = get(&endpoint, Some(opts)).await?;
        let arr = if val.is_array() {
            val
        } else if let Some(d) = val.get("data") {
            d.clone()
        } else {
            return Err(ClobError::Other(
                "unexpected spreads response shape".to_string(),
            ));
        };
        let prices: Vec<crate::types::MarketPrice> =
            serde_json::from_value(arr).map_err(|e| ClobError::Other(e.to_string()))?;
        Ok(prices)
    }

    pub async fn get_last_trades_prices(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::MarketPrice>, ClobError> {
        let endpoint = format!("{}{}", self.host, GET_LAST_TRADES_PRICES);
        let opts = RequestOptions {
            headers: None,
            data: None,
            params,
        };
        let val = get(&endpoint, Some(opts)).await?;
        let arr = if val.is_array() {
            val
        } else if let Some(d) = val.get("data") {
            d.clone()
        } else {
            return Err(ClobError::Other(
                "unexpected last trades prices response shape".to_string(),
            ));
        };
        let prices: Vec<crate::types::MarketPrice> =
            serde_json::from_value(arr).map_err(|e| ClobError::Other(e.to_string()))?;
        Ok(prices)
    }

    pub async fn get_prices_history(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::MarketPrice>, ClobError> {
        let endpoint = format!("{}{}", self.host, GET_PRICES_HISTORY);
        let opts = RequestOptions {
            headers: None,
            data: None,
            params,
        };
        let val = get(&endpoint, Some(opts)).await?;
        // Accept array or { data: [...] }
        let arr = if val.is_array() {
            val
        } else if let Some(d) = val.get("data") {
            d.clone()
        } else {
            return Err(ClobError::Other(
                "unexpected prices history response shape".to_string(),
            ));
        };
        let prices: Vec<crate::types::MarketPrice> =
            serde_json::from_value(arr).map_err(|e| ClobError::Other(e.to_string()))?;
        Ok(prices)
    }

    pub async fn get_market_trades_events(
        &self,
        market_id: &str,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Trade>, ClobError> {
        let endpoint = format!("{}{}{}", self.host, GET_MARKET_TRADES_EVENTS, market_id);
        let opts = RequestOptions {
            headers: None,
            data: None,
            params,
        };
        let val = get(&endpoint, Some(opts)).await?;
        let arr = if val.is_array() {
            val
        } else if let Some(d) = val.get("data") {
            d.clone()
        } else {
            return Err(ClobError::Other(
                "unexpected market trades events response shape".to_string(),
            ));
        };
        let trades: Vec<crate::types::Trade> =
            serde_json::from_value(arr).map_err(|e| ClobError::Other(e.to_string()))?;
        Ok(trades)
    }

    // Rewards endpoints
    pub async fn get_earnings_for_user_for_day(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Reward>, ClobError> {
        let endpoint = format!("{}{}", self.host, GET_EARNINGS_FOR_USER_FOR_DAY);
        let opts = RequestOptions {
            headers: None,
            data: None,
            params,
        };
        let val = get(&endpoint, Some(opts)).await?;
        let arr = if val.is_array() {
            val
        } else if let Some(d) = val.get("data") {
            d.clone()
        } else {
            return Err(ClobError::Other(
                "unexpected earnings response shape".to_string(),
            ));
        };
        let rewards: Vec<crate::types::Reward> =
            serde_json::from_value(arr).map_err(|e| ClobError::Other(e.to_string()))?;
        Ok(rewards)
    }

    pub async fn get_total_earnings_for_user_for_day(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Reward>, ClobError> {
        let endpoint = format!("{}{}", self.host, GET_TOTAL_EARNINGS_FOR_USER_FOR_DAY);
        let opts = RequestOptions {
            headers: None,
            data: None,
            params,
        };
        let val = get(&endpoint, Some(opts)).await?;
        let arr = if val.is_array() {
            val
        } else if let Some(d) = val.get("data") {
            d.clone()
        } else {
            return Err(ClobError::Other(
                "unexpected total earnings response shape".to_string(),
            ));
        };
        let rewards: Vec<crate::types::Reward> =
            serde_json::from_value(arr).map_err(|e| ClobError::Other(e.to_string()))?;
        Ok(rewards)
    }

    /// Typed wrapper for total earnings for user for day. Attempts to parse an array of Reward.
    pub async fn get_total_earnings_for_user_for_day_typed(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Reward>, ClobError> {
        let val = self.get_total_earnings_for_user_for_day(params).await?;
        Ok(val)
    }

    pub async fn get_liquidity_reward_percentages(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<std::collections::HashMap<String, f64>, ClobError> {
        let endpoint = format!("{}{}", self.host, GET_LIQUIDITY_REWARD_PERCENTAGES);
        let opts = RequestOptions {
            headers: None,
            data: None,
            params,
        };
        let val = get(&endpoint, Some(opts)).await?;
        // Accept object or { data: object }
        let obj = if val.is_object() {
            val
        } else if let Some(d) = val.get("data") {
            d.clone()
        } else {
            return Err(ClobError::Other(
                "unexpected liquidity percentages response shape".to_string(),
            ));
        };
        let map: std::collections::HashMap<String, f64> =
            serde_json::from_value(obj).map_err(|e| ClobError::Other(e.to_string()))?;
        Ok(map)
    }

    /// Typed wrapper for liquidity reward percentages. Attempts to parse into a map of market -> percentage.
    pub async fn get_liquidity_reward_percentages_typed(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<std::collections::HashMap<String, f64>, ClobError> {
        let val = self.get_liquidity_reward_percentages(params).await?;
        Ok(val)
    }

    pub async fn get_rewards_markets_current(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Reward>, ClobError> {
        let endpoint = format!("{}{}", self.host, GET_REWARDS_MARKETS_CURRENT);
        let opts = RequestOptions {
            headers: None,
            data: None,
            params,
        };
        let val = get(&endpoint, Some(opts)).await?;
        let arr = if val.is_array() {
            val
        } else if let Some(d) = val.get("data") {
            d.clone()
        } else {
            return Err(ClobError::Other(
                "unexpected rewards markets response shape".to_string(),
            ));
        };
        let rewards: Vec<crate::types::Reward> =
            serde_json::from_value(arr).map_err(|e| ClobError::Other(e.to_string()))?;
        Ok(rewards)
    }

    pub async fn get_rewards_markets(
        &self,
        market_id: &str,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Reward>, ClobError> {
        let endpoint = format!("{}{}{}", self.host, GET_REWARDS_MARKETS, market_id);
        let opts = RequestOptions {
            headers: None,
            data: None,
            params,
        };
        let val = get(&endpoint, Some(opts)).await?;
        let arr = if val.is_array() {
            val
        } else if let Some(d) = val.get("data") {
            d.clone()
        } else {
            return Err(ClobError::Other(
                "unexpected rewards markets response shape".to_string(),
            ));
        };
        let rewards: Vec<crate::types::Reward> =
            serde_json::from_value(arr).map_err(|e| ClobError::Other(e.to_string()))?;
        Ok(rewards)
    }

    /// Typed wrapper for get_rewards_markets (per-market rewards). Returns Vec<Reward>.
    pub async fn get_rewards_markets_typed(
        &self,
        market_id: &str,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Reward>, ClobError> {
        let val = self.get_rewards_markets(market_id, params).await?;
        Ok(val)
    }

    pub async fn get_rewards_earnings_percentages(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Reward>, ClobError> {
        let endpoint = format!("{}{}", self.host, GET_REWARDS_EARNINGS_PERCENTAGES);
        let opts = RequestOptions {
            headers: None,
            data: None,
            params,
        };
        let val = get(&endpoint, Some(opts)).await?;
        let arr = if val.is_array() {
            val
        } else if let Some(d) = val.get("data") {
            d.clone()
        } else {
            return Err(ClobError::Other(
                "unexpected rewards earnings percentages response shape".to_string(),
            ));
        };
        let rewards: Vec<crate::types::Reward> =
            serde_json::from_value(arr).map_err(|e| ClobError::Other(e.to_string()))?;
        Ok(rewards)
    }

    /// Typed wrapper for rewards earnings percentages. Returns Vec<Reward> or object parsed into map.
    pub async fn get_rewards_earnings_percentages_typed(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Reward>, ClobError> {
        let val = self.get_rewards_earnings_percentages(params).await?;
        Ok(val)
    }

    pub async fn get_builder_trades(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Trade>, ClobError> {
        let endpoint = format!("{}{}", self.host, GET_BUILDER_TRADES);
        let opts = RequestOptions {
            headers: None,
            data: None,
            params,
        };
        let val = get(&endpoint, Some(opts)).await?;
        let arr = if val.is_array() {
            val
        } else if let Some(d) = val.get("data") {
            d.clone()
        } else {
            return Err(ClobError::Other(
                "unexpected builder trades response shape".to_string(),
            ));
        };
        let trades: Vec<crate::types::Trade> =
            serde_json::from_value(arr).map_err(|e| ClobError::Other(e.to_string()))?;
        Ok(trades)
    }

    /// Typed variant for builder trades (kept for compatibility)
    pub async fn get_builder_trades_typed(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Trade>, ClobError> {
        self.get_builder_trades(params).await
    }

    /// Typed wrapper for get_earnings_for_user_for_day -> Vec<Reward>
    pub async fn get_earnings_for_user_for_day_typed(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Reward>, ClobError> {
        let val = self.get_earnings_for_user_for_day(params).await?;
        Ok(val)
    }

    /// Typed wrapper for current rewards markets
    pub async fn get_rewards_markets_current_typed(
        &self,
        params: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<crate::types::Reward>, ClobError> {
        let val = self.get_rewards_markets_current(params).await?;
        Ok(val)
    }
}
