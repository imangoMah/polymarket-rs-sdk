use crate::types::{NewOrder, NewOrderData, OrderBookSummary, OrderType, SignedOrder};
use serde_json::to_string;
use sha1::{Digest, Sha1};

/// Convert a SignedOrder to NewOrder format for posting to the API
/// This matches the TypeScript orderToJson function
pub fn order_to_json(
    order: &SignedOrder,
    owner: &str,
    order_type: OrderType,
    defer_exec: bool,
) -> NewOrder {
    NewOrder {
        order: NewOrderData {
            // 与 TS SDK 一致：parseInt(order.salt, 10)，将字符串盐解析为数值
            // 已确保生成时在 JS 安全整数范围内（2^53-1），不会溢出
            salt: order.salt.parse::<i64>().unwrap_or(0),
            maker: order.maker.clone(),
            signer: order.signer.clone(),
            taker: order.taker.clone(),
            token_id: order.token_id.clone(),
            maker_amount: order.maker_amount.clone(),
            taker_amount: order.taker_amount.clone(),
            expiration: order.expiration.clone(),
            nonce: order.nonce.clone(),
            fee_rate_bps: order.fee_rate_bps.clone(),
            side: order.side.clone(),
            signature_type: order.signature_type,
            signature: order.signature.clone(),
        },
        owner: owner.to_string(),
        order_type,
        defer_exec,
    }
}

/// Round normally to given decimals
pub fn round_normal(num: f64, decimals: u32) -> f64 {
    if decimal_places(num) <= decimals {
        return num;
    }
    let factor = 10f64.powi(decimals as i32);
    ((num + f64::EPSILON) * factor).round() / factor
}

pub fn round_down(num: f64, decimals: u32) -> f64 {
    if decimal_places(num) <= decimals {
        return num;
    }
    let factor = 10f64.powi(decimals as i32);
    (num * factor).floor() / factor
}

pub fn round_up(num: f64, decimals: u32) -> f64 {
    if decimal_places(num) <= decimals {
        return num;
    }
    let factor = 10f64.powi(decimals as i32);
    (num * factor).ceil() / factor
}

pub fn decimal_places(num: f64) -> u32 {
    if num.fract() == 0.0 {
        return 0;
    }
    let s = format!("{}", num);
    match s.split('.').nth(1) {
        Some(frac) => frac.len() as u32,
        None => 0,
    }
}

/// Generates a SHA1 hash of the provided orderbook JSON. Returns the hex string.
pub fn generate_orderbook_summary_hash(orderbook: &OrderBookSummary) -> String {
    // Serialize with unchanged hash field for a stable representation
    let json = match to_string(orderbook) {
        Ok(j) => j,
        Err(_) => "".to_string(),
    };
    let mut hasher = Sha1::new();
    hasher.update(json.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn is_tick_size_smaller(a: &str, b: &str) -> bool {
    match (a.parse::<f64>(), b.parse::<f64>()) {
        (Ok(aa), Ok(bb)) => aa < bb,
        _ => false,
    }
}

pub fn price_valid(price: f64, tick_size: &str) -> bool {
    match tick_size.parse::<f64>() {
        Ok(ts) => price >= ts && price <= 1.0 - ts,
        Err(_) => false,
    }
}
