use crate::errors::ClobError;
use crate::exchange_order_builder::ExchangeOrderBuilder;
use crate::signing::Eip712Signer;
use crate::types::{OrderData, Side, SignatureType, SignedOrder};
use crate::utilities::{decimal_places, round_down, round_normal, round_up};
use rust_decimal::prelude::Decimal;
use rust_decimal::prelude::ToPrimitive;

#[derive(Debug, Clone)]
pub struct RoundConfig {
    pub price: u32,
    pub size: u32,
    pub amount: u32,
}

use std::collections::HashMap;

pub fn rounding_config() -> HashMap<&'static str, RoundConfig> {
    let mut m = HashMap::new();
    m.insert(
        "0.1",
        RoundConfig {
            price: 1,
            size: 2,
            amount: 3,
        },
    );
    m.insert(
        "0.01",
        RoundConfig {
            price: 2,
            size: 2,
            amount: 4,
        },
    );
    m.insert(
        "0.001",
        RoundConfig {
            price: 3,
            size: 2,
            amount: 5,
        },
    );
    m.insert(
        "0.0001",
        RoundConfig {
            price: 4,
            size: 2,
            amount: 6,
        },
    );
    m
}

pub struct RawAmounts {
    pub side: Side,
    pub raw_maker_amt: f64,
    pub raw_taker_amt: f64,
}

pub fn get_order_raw_amounts(
    side: Side,
    size: f64,
    price: f64,
    round_config: &RoundConfig,
) -> RawAmounts {
    let raw_price = round_normal(price, round_config.price);

    if let Side::BUY = side {
        let raw_taker_amt = round_down(size, round_config.size);
        let mut raw_maker_amt = raw_taker_amt * raw_price;
        if decimal_places(raw_maker_amt) > round_config.amount {
            raw_maker_amt = round_up(raw_maker_amt, round_config.amount + 4);
            if decimal_places(raw_maker_amt) > round_config.amount {
                raw_maker_amt = round_down(raw_maker_amt, round_config.amount);
            }
        }
        RawAmounts {
            side: Side::BUY,
            raw_maker_amt,
            raw_taker_amt,
        }
    } else {
        let raw_maker_amt = round_down(size, round_config.size);
        let mut raw_taker_amt = raw_maker_amt * raw_price;
        if decimal_places(raw_taker_amt) > round_config.amount {
            raw_taker_amt = round_up(raw_taker_amt, round_config.amount + 4);
            if decimal_places(raw_taker_amt) > round_config.amount {
                raw_taker_amt = round_down(raw_taker_amt, round_config.amount);
            }
        }
        RawAmounts {
            side: Side::SELL,
            raw_maker_amt,
            raw_taker_amt,
        }
    }
}

fn parse_units(value: &str, decimals: u32) -> Result<String, ClobError> {
    // Use Decimal for exact decimal arithmetic
    let d = value
        .parse::<Decimal>()
        .map_err(|e| ClobError::Other(format!("parse decimal error: {}", e)))?;
    let factor = Decimal::new(10i64.pow(decimals), 0);
    let scaled = d * factor;
    // Truncate toward zero
    let scaled_i = scaled.trunc();
    match scaled_i.to_i128() {
        Some(i) => Ok(i.to_string()),
        None => Err(ClobError::Other("scaled value out of range".to_string())),
    }
}

pub async fn build_order_creation_args(
    signer: &str,
    maker: &str,
    signature_type: SignatureType,
    user_order: &crate::types::UserOrder,
    round_config: &RoundConfig,
) -> Result<OrderData, ClobError> {
    let ra = get_order_raw_amounts(
        user_order.side.clone(),
        user_order.size,
        user_order.price,
        round_config,
    );

    let maker_amount = parse_units(&ra.raw_maker_amt.to_string(), 6)?; // COLLATERAL_TOKEN_DECIMALS = 6
    let taker_amount = parse_units(&ra.raw_taker_amt.to_string(), 6)?;

    let taker = if let Some(t) = &user_order.taker {
        if !t.is_empty() {
            t.clone()
        } else {
            "0x0000000000000000000000000000000000000000".to_string()
        }
    } else {
        "0x0000000000000000000000000000000000000000".to_string()
    };

    // fee_rate_bps 必填，直接转为字符串
    let fee_rate_bps = user_order.fee_rate_bps.to_string();
    let nonce = user_order
        .nonce
        .map(|v| v.to_string())
        .unwrap_or_else(|| "0".to_string());

    Ok(OrderData {
        maker: maker.to_string(),
        taker,
        token_id: user_order.token_id.clone(),
        maker_amount,
        taker_amount,
        side: ra.side,
        fee_rate_bps,
        nonce,
        signer: signer.to_string(),
        expiration: user_order
            .expiration
            .map(|v| v.to_string())
            .unwrap_or_else(|| "0".to_string()),
        signature_type,
    })
}

// Placeholder for build_order which uses exchange order builder and signing
pub async fn build_order(
    signer: &impl crate::signing::Eip712Signer,
    exchange_address: &str,
    chain_id: i32,
    order_data: OrderData,
) -> Result<SignedOrder, ClobError> {
    let builder = ExchangeOrderBuilder::new(exchange_address, chain_id as i64, signer);
    builder.build_signed_order(order_data).await
}

pub fn get_market_order_raw_amounts(
    side: Side,
    amount: f64,
    price: f64,
    round_config: &RoundConfig,
) -> RawAmounts {
    let raw_price = round_down(price, round_config.price);

    if let Side::BUY = side {
        let raw_maker_amt = round_down(amount, round_config.size);
        let mut raw_taker_amt = raw_maker_amt / raw_price;
        if decimal_places(raw_taker_amt) > round_config.amount {
            raw_taker_amt = round_up(raw_taker_amt, round_config.amount + 4);
            if decimal_places(raw_taker_amt) > round_config.amount {
                raw_taker_amt = round_down(raw_taker_amt, round_config.amount);
            }
        }
        RawAmounts {
            side: Side::BUY,
            raw_maker_amt,
            raw_taker_amt,
        }
    } else {
        let raw_maker_amt = round_down(amount, round_config.size);
        let mut raw_taker_amt = raw_maker_amt * raw_price;
        if decimal_places(raw_taker_amt) > round_config.amount {
            raw_taker_amt = round_up(raw_taker_amt, round_config.amount + 4);
            if decimal_places(raw_taker_amt) > round_config.amount {
                raw_taker_amt = round_down(raw_taker_amt, round_config.amount);
            }
        }
        RawAmounts {
            side: Side::SELL,
            raw_maker_amt,
            raw_taker_amt,
        }
    }
}

pub struct OrderBuilder<'a, S: Eip712Signer> {
    signer: &'a S,
    chain_id: i64,
    signature_type: SignatureType,
    funder_address: Option<String>,
    // get_signer omitted for simplicity; can be added later
}

#[derive(Debug, Clone)]
pub struct BuilderConfig {
    pub tick_size: Option<String>,
    pub neg_risk: Option<bool>,
    pub signature_type: SignatureType,
    pub funder_address: Option<String>,
}

impl Default for BuilderConfig {
    fn default() -> Self {
        Self {
            tick_size: None,
            neg_risk: None,
            signature_type: SignatureType::EOA,
            funder_address: None,
        }
    }
}

impl<'a, S: Eip712Signer> OrderBuilder<'a, S> {
    pub fn new(
        signer: &'a S,
        chain_id: i64,
        signature_type: Option<SignatureType>,
        funder_address: Option<String>,
    ) -> Self {
        Self {
            signer,
            chain_id,
            signature_type: signature_type.unwrap_or(SignatureType::EOA),
            funder_address,
        }
    }

    pub fn with_config(signer: &'a S, chain_id: i64, cfg: &BuilderConfig) -> Self {
        Self {
            signer,
            chain_id,
            signature_type: cfg.signature_type.clone(),
            funder_address: cfg.funder_address.clone(),
        }
    }

    pub async fn build_order(
        &self,
        exchange_address: &str,
        user_order: &crate::types::UserOrder,
        options_tick: &str,
    ) -> Result<crate::types::SignedOrder, ClobError> {
        let rc_map = rounding_config();
        let round_config = rc_map
            .get(options_tick)
            .ok_or(ClobError::Other("invalid tick size".to_string()))?;
        let eoa_addr = self.signer.get_address().await?;
        let maker = self.funder_address.clone().unwrap_or(eoa_addr.clone());

        let order_data = build_order_creation_args(
            &eoa_addr,
            &maker,
            self.signature_type.clone(),
            user_order,
            round_config,
        )
        .await?;

        let builder = ExchangeOrderBuilder::new(exchange_address, self.chain_id, self.signer);
        builder.build_signed_order(order_data).await
    }

    /// 构建限价单（强制指定 salt，用于跨语言/跨路径签名完全一致的验证）
    pub async fn build_order_with_salt(
        &self,
        exchange_address: &str,
        user_order: &crate::types::UserOrder,
        options_tick: &str,
        forced_salt: &str,
    ) -> Result<crate::types::SignedOrder, ClobError> {
        let rc_map = rounding_config();
        let round_config = rc_map
            .get(options_tick)
            .ok_or(ClobError::Other("invalid tick size".to_string()))?;
        let eoa_addr = self.signer.get_address().await?;
        let maker = self.funder_address.clone().unwrap_or(eoa_addr.clone());

        let order_data = build_order_creation_args(
            &eoa_addr,
            &maker,
            self.signature_type.clone(),
            user_order,
            round_config,
        )
        .await?;

        let builder = ExchangeOrderBuilder::new(exchange_address, self.chain_id, self.signer);
        builder
            .build_signed_order_with_salt(order_data, forced_salt)
            .await
    }

    pub async fn build_market_order(
        &self,
        exchange_address: &str,
        user_market_order: &crate::types::UserMarketOrder,
        options_tick: &str,
    ) -> Result<crate::types::SignedOrder, ClobError> {
        let rc_map = rounding_config();
        let round_config = rc_map
            .get(options_tick)
            .ok_or(ClobError::Other("invalid tick size".to_string()))?;
        let eoa_addr = self.signer.get_address().await?;
        let maker = self.funder_address.clone().unwrap_or(eoa_addr.clone());

        let order_data = build_market_order_creation_args(
            &eoa_addr,
            &maker,
            self.signature_type.clone(),
            user_market_order,
            round_config,
        )
        .await?;

        let builder = ExchangeOrderBuilder::new(exchange_address, self.chain_id, self.signer);
        builder.build_signed_order(order_data).await
    }
}

pub async fn build_market_order_creation_args(
    signer: &str,
    maker: &str,
    signature_type: SignatureType,
    user_market_order: &crate::types::UserMarketOrder,
    round_config: &RoundConfig,
) -> Result<OrderData, ClobError> {
    // 市价单价格需外部计算并传入，移除默认值
    let price = user_market_order.price;
    let ra = get_market_order_raw_amounts(
        user_market_order.side.clone(),
        user_market_order.amount,
        price,
        round_config,
    );

    let maker_amount = parse_units(&ra.raw_maker_amt.to_string(), 6)?;
    let taker_amount = parse_units(&ra.raw_taker_amt.to_string(), 6)?;

    let taker = if let Some(t) = &user_market_order.taker {
        if !t.is_empty() {
            t.clone()
        } else {
            "0x0000000000000000000000000000000000000000".to_string()
        }
    } else {
        "0x0000000000000000000000000000000000000000".to_string()
    };

    // fee_rate_bps 必填
    let fee_rate_bps = user_market_order.fee_rate_bps.to_string();
    let nonce = user_market_order
        .nonce
        .map(|v| v.to_string())
        .unwrap_or_else(|| "0".to_string());

    Ok(OrderData {
        maker: maker.to_string(),
        taker,
        token_id: user_market_order.token_id.clone(),
        maker_amount,
        taker_amount,
        side: ra.side,
        fee_rate_bps,
        nonce,
        signer: signer.to_string(),
        expiration: "0".to_string(),
        signature_type,
    })
}

pub fn calculate_buy_market_price(
    positions: &[crate::types::OrderSummary],
    amount_to_match: f64,
    order_type: crate::types::OrderType,
) -> Result<f64, ClobError> {
    if positions.is_empty() {
        return Err(ClobError::Other("no match".to_string()));
    }
    let mut sum = 0.0;
    for i in (0..positions.len()).rev() {
        let p = &positions[i];
        let price = p.price.parse::<f64>().unwrap_or(0.0);
        let size = p.size.parse::<f64>().unwrap_or(0.0);
        sum += size * price;
        if sum >= amount_to_match {
            return Ok(price);
        }
    }
    if let crate::types::OrderType::FOK = order_type {
        return Err(ClobError::Other("no match".to_string()));
    }
    Ok(positions[0].price.parse::<f64>().unwrap_or(0.0))
}

pub fn calculate_sell_market_price(
    positions: &[crate::types::OrderSummary],
    amount_to_match: f64,
    order_type: crate::types::OrderType,
) -> Result<f64, ClobError> {
    if positions.is_empty() {
        return Err(ClobError::Other("no match".to_string()));
    }
    let mut sum = 0.0;
    for i in (0..positions.len()).rev() {
        let p = &positions[i];
        let price = p.price.parse::<f64>().unwrap_or(0.0);
        let size = p.size.parse::<f64>().unwrap_or(0.0);
        sum += size;
        if sum >= amount_to_match {
            return Ok(price);
        }
    }
    if let crate::types::OrderType::FOK = order_type {
        return Err(ClobError::Other("no match".to_string()));
    }
    Ok(positions[0].price.parse::<f64>().unwrap_or(0.0))
}

/// 基于外部订单簿快照计算市价单价格（纯函数，无 HTTP）
pub fn compute_market_price_from_book(
    book: &crate::types::OrderBookSummary,
    side: crate::types::Side,
    amount: f64,
    order_type: crate::types::OrderType,
) -> Result<f64, ClobError> {
    match side {
        crate::types::Side::BUY => {
            if book.asks.is_empty() {
                return Err(ClobError::Other("no match".to_string()));
            }
            calculate_buy_market_price(&book.asks, amount, order_type)
        }
        crate::types::Side::SELL => {
            if book.bids.is_empty() {
                return Err(ClobError::Other("no match".to_string()));
            }
            calculate_sell_market_price(&book.bids, amount, order_type)
        }
    }
}
