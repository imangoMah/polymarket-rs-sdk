use crate::errors::ClobError;
use reqwest::Client;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;

pub const GET: &str = "GET";
pub const POST: &str = "POST";
pub const DELETE: &str = "DELETE";
pub const PUT: &str = "PUT";

pub type QueryParams = HashMap<String, String>;

pub struct RequestOptions<B = Value> {
    pub headers: Option<HashMap<String, String>>,
    pub data: Option<B>,
    pub params: Option<QueryParams>,
}

pub async fn post_typed<R, B>(
    endpoint: &str,
    options: Option<RequestOptions<B>>,
) -> Result<R, ClobError>
where
    R: DeserializeOwned,
    B: Serialize,
{
    let client = Client::new();
    let mut req = client.post(endpoint);
    let mut debug_headers: Option<std::collections::HashMap<String, String>> = None;
    let mut debug_body: Option<String> = None;
    let mut debug_params: Option<QueryParams> = None;
    if let Some(opts) = options {
        if let Some(h) = opts.headers {
            // Capture headers for debug (with masking) before moving
            if std::env::var("CLOB_DEBUG_FULL").is_ok() || std::env::var("CLOB_DEBUG_RAW").is_ok() {
                let mut masked = std::collections::HashMap::new();
                for (k, v) in h.iter() {
                    let key = k.to_string();
                    let val = if key.contains("PASSPHRASE") || key.contains("API_KEY") {
                        if v.len() > 6 {
                            format!("{}***", &v[..6])
                        } else {
                            "***".to_string()
                        }
                    } else if key.contains("SIGNATURE") {
                        if v.len() > 12 {
                            format!("{}...", &v[..12])
                        } else {
                            "***".to_string()
                        }
                    } else {
                        v.to_string()
                    };
                    masked.insert(key, val);
                }
                debug_headers = Some(masked);
            }
            for (k, v) in h.iter() {
                req = req.header(k, v);
            }
        }
        if let Some(body) = opts.data {
            // Serialize once for debug printing; req.json will produce the same representation
            if std::env::var("CLOB_DEBUG_FULL").is_ok() || std::env::var("CLOB_DEBUG_RAW").is_ok() {
                if let Ok(b) = serde_json::to_string(&body) {
                    debug_body = Some(b);
                }
            }
            req = req.json(&body);
        }
        if let Some(params) = opts.params {
            if std::env::var("CLOB_DEBUG_FULL").is_ok() || std::env::var("CLOB_DEBUG_RAW").is_ok() {
                debug_params = Some(params.clone());
            }
            req = req.query(&params);
        }
    }
    if std::env::var("CLOB_DEBUG_FULL").is_ok() || std::env::var("CLOB_DEBUG_RAW").is_ok() {
        eprintln!("[HTTP DEBUG] POST {}", endpoint);
        if let Some(h) = &debug_headers {
            eprintln!("  headers={:?}", h);
        }
        if let Some(p) = &debug_params {
            eprintln!("  params={:?}", p);
        }
        if let Some(b) = &debug_body {
            let preview = if b.len() > 800 {
                format!("{}... ({} bytes)", &b[..800], b.len())
            } else {
                b.clone()
            };
            eprintln!("  body={}", preview);
        }
    }
    let resp = req
        .send()
        .await
        .map_err(|e| ClobError::Other(format!("HTTP request failed: {}", e)))?;

    let status = resp.status();

    // Check status code first, before trying to parse
    if !status.is_success() {
        // Get response body as text for error details
        let body_text = resp
            .text()
            .await
            .unwrap_or_else(|_| "<unable to read response body>".to_string());

        eprintln!("❌ HTTP Error Response:");
        eprintln!("   Status: {}", status);
        eprintln!("   Endpoint: {}", endpoint);
        eprintln!("   Response Body: {}", body_text);

        return Err(ClobError::Other(format!(
            "HTTP {} error from {}: {}",
            status, endpoint, body_text
        )));
    }

    // Try to parse JSON, with detailed error message if it fails
    let body_text = resp
        .text()
        .await
        .map_err(|e| ClobError::Other(format!("Failed to read response body: {}", e)))?;

    match serde_json::from_str::<R>(&body_text) {
        Ok(val) => Ok(val),
        Err(e) => {
            eprintln!("❌ JSON Parse Error:");
            eprintln!("   Endpoint: {}", endpoint);
            eprintln!("   Error: {}", e);
            eprintln!("   Response Body: {}", body_text);

            Err(ClobError::Other(format!(
                "Failed to parse JSON response from {}: {}. Response body: {}",
                endpoint,
                e,
                if body_text.len() > 500 {
                    format!(
                        "{}... (truncated, {} bytes total)",
                        &body_text[..500],
                        body_text.len()
                    )
                } else {
                    body_text
                }
            )))
        }
    }
}

pub async fn post(endpoint: &str, options: Option<RequestOptions>) -> Result<Value, ClobError> {
    post_typed::<Value, Value>(endpoint, options).await
}

pub async fn get_typed<R, B>(
    endpoint: &str,
    options: Option<RequestOptions<B>>,
) -> Result<R, ClobError>
where
    R: DeserializeOwned,
    B: Serialize,
{
    let client = Client::new();
    let mut req = client.get(endpoint);
    let mut debug_headers: Option<std::collections::HashMap<String, String>> = None;
    let mut debug_params: Option<QueryParams> = None;
    if let Some(opts) = options {
        if let Some(h) = opts.headers {
            if std::env::var("CLOB_DEBUG_FULL").is_ok() || std::env::var("CLOB_DEBUG_RAW").is_ok() {
                let mut masked = std::collections::HashMap::new();
                for (k, v) in h.iter() {
                    let key = k.to_string();
                    let val = if key.contains("PASSPHRASE") || key.contains("API_KEY") {
                        if v.len() > 6 {
                            format!("{}***", &v[..6])
                        } else {
                            "***".to_string()
                        }
                    } else if key.contains("SIGNATURE") {
                        if v.len() > 12 {
                            format!("{}...", &v[..12])
                        } else {
                            "***".to_string()
                        }
                    } else {
                        v.to_string()
                    };
                    masked.insert(key, val);
                }
                debug_headers = Some(masked);
            }
            for (k, v) in h.iter() {
                req = req.header(k, v);
            }
        }
        if let Some(params) = opts.params {
            if std::env::var("CLOB_DEBUG_FULL").is_ok() || std::env::var("CLOB_DEBUG_RAW").is_ok() {
                debug_params = Some(params.clone());
            }
            req = req.query(&params);
        }
    }
    if std::env::var("CLOB_DEBUG_FULL").is_ok() || std::env::var("CLOB_DEBUG_RAW").is_ok() {
        eprintln!("[HTTP DEBUG] GET {}", endpoint);
        if let Some(h) = &debug_headers {
            eprintln!("  headers={:?}", h);
        }
        if let Some(p) = &debug_params {
            eprintln!("  params={:?}", p);
        }
    }
    let resp = req
        .send()
        .await
        .map_err(|e| ClobError::Other(format!("HTTP request failed: {}", e)))?;

    let status = resp.status();

    // Check status code first, before trying to parse
    if !status.is_success() {
        // Get response body as text for error details
        let body_text = resp
            .text()
            .await
            .unwrap_or_else(|_| "<unable to read response body>".to_string());

        eprintln!("❌ HTTP Error Response:");
        eprintln!("   Status: {}", status);
        eprintln!("   Endpoint: {}", endpoint);
        eprintln!("   Response Body: {}", body_text);

        return Err(ClobError::Other(format!(
            "HTTP {} error from {}: {}",
            status, endpoint, body_text
        )));
    }

    // Try to parse JSON, with detailed error message if it fails
    let body_text = resp
        .text()
        .await
        .map_err(|e| ClobError::Other(format!("Failed to read response body: {}", e)))?;

    match serde_json::from_str::<R>(&body_text) {
        Ok(val) => Ok(val),
        Err(e) => {
            eprintln!("❌ JSON Parse Error:");
            eprintln!("   Endpoint: {}", endpoint);
            eprintln!("   Error: {}", e);
            eprintln!("   Response Body: {}", body_text);

            Err(ClobError::Other(format!(
                "Failed to parse JSON response from {}: {}. Response body: {}",
                endpoint,
                e,
                if body_text.len() > 500 {
                    format!(
                        "{}... (truncated, {} bytes total)",
                        &body_text[..500],
                        body_text.len()
                    )
                } else {
                    body_text
                }
            )))
        }
    }
}

pub async fn get(endpoint: &str, options: Option<RequestOptions>) -> Result<Value, ClobError> {
    get_typed::<Value, Value>(endpoint, options).await
}

pub async fn del_typed<R, B>(
    endpoint: &str,
    options: Option<RequestOptions<B>>,
) -> Result<R, ClobError>
where
    R: DeserializeOwned,
    B: Serialize,
{
    let client = Client::new();
    let mut req = client.delete(endpoint);
    let mut debug_headers: Option<std::collections::HashMap<String, String>> = None;
    let mut debug_body: Option<String> = None;
    let mut debug_params: Option<QueryParams> = None;
    if let Some(opts) = options {
        if let Some(h) = opts.headers {
            if std::env::var("CLOB_DEBUG_FULL").is_ok() || std::env::var("CLOB_DEBUG_RAW").is_ok() {
                let mut masked = std::collections::HashMap::new();
                for (k, v) in h.iter() {
                    let key = k.to_string();
                    let val = if key.contains("PASSPHRASE") || key.contains("API_KEY") {
                        if v.len() > 6 {
                            format!("{}***", &v[..6])
                        } else {
                            "***".to_string()
                        }
                    } else if key.contains("SIGNATURE") {
                        if v.len() > 12 {
                            format!("{}...", &v[..12])
                        } else {
                            "***".to_string()
                        }
                    } else {
                        v.to_string()
                    };
                    masked.insert(key, val);
                }
                debug_headers = Some(masked);
            }
            for (k, v) in h.iter() {
                req = req.header(k, v);
            }
        }
        if let Some(body) = opts.data {
            if std::env::var("CLOB_DEBUG_FULL").is_ok() || std::env::var("CLOB_DEBUG_RAW").is_ok() {
                if let Ok(b) = serde_json::to_string(&body) {
                    debug_body = Some(b);
                }
            }
            req = req.json(&body);
        }
        if let Some(params) = opts.params {
            if std::env::var("CLOB_DEBUG_FULL").is_ok() || std::env::var("CLOB_DEBUG_RAW").is_ok() {
                debug_params = Some(params.clone());
            }
            req = req.query(&params);
        }
    }
    if std::env::var("CLOB_DEBUG_FULL").is_ok() || std::env::var("CLOB_DEBUG_RAW").is_ok() {
        eprintln!("[HTTP DEBUG] DELETE {}", endpoint);
        if let Some(h) = &debug_headers {
            eprintln!("  headers={:?}", h);
        }
        if let Some(p) = &debug_params {
            eprintln!("  params={:?}", p);
        }
        if let Some(b) = &debug_body {
            let preview = if b.len() > 800 {
                format!("{}... ({} bytes)", &b[..800], b.len())
            } else {
                b.clone()
            };
            eprintln!("  body={}", preview);
        }
    }
    let resp = req
        .send()
        .await
        .map_err(|e| ClobError::Other(format!("HTTP request failed: {}", e)))?;

    let status = resp.status();

    // Check status code first, before trying to parse
    if !status.is_success() {
        // Get response body as text for error details
        let body_text = resp
            .text()
            .await
            .unwrap_or_else(|_| "<unable to read response body>".to_string());

        eprintln!("❌ HTTP Error Response:");
        eprintln!("   Status: {}", status);
        eprintln!("   Endpoint: {}", endpoint);
        eprintln!("   Response Body: {}", body_text);

        return Err(ClobError::Other(format!(
            "HTTP {} error from {}: {}",
            status, endpoint, body_text
        )));
    }

    // Try to parse JSON, with detailed error message if it fails
    let body_text = resp
        .text()
        .await
        .map_err(|e| ClobError::Other(format!("Failed to read response body: {}", e)))?;

    match serde_json::from_str::<R>(&body_text) {
        Ok(val) => Ok(val),
        Err(e) => {
            eprintln!("❌ JSON Parse Error:");
            eprintln!("   Endpoint: {}", endpoint);
            eprintln!("   Error: {}", e);
            eprintln!("   Response Body: {}", body_text);

            Err(ClobError::Other(format!(
                "Failed to parse JSON response from {}: {}. Response body: {}",
                endpoint,
                e,
                if body_text.len() > 500 {
                    format!(
                        "{}... (truncated, {} bytes total)",
                        &body_text[..500],
                        body_text.len()
                    )
                } else {
                    body_text
                }
            )))
        }
    }
}

pub async fn del(endpoint: &str, options: Option<RequestOptions>) -> Result<Value, ClobError> {
    del_typed::<Value, Value>(endpoint, options).await
}

pub fn parse_orders_scoring_params(order_ids: Option<&Vec<String>>) -> QueryParams {
    let mut params = QueryParams::new();
    if let Some(ids) = order_ids {
        params.insert("order_ids".to_string(), ids.join(","));
    }
    params
}

pub fn parse_drop_notification_params(ids: Option<&Vec<String>>) -> QueryParams {
    let mut params = QueryParams::new();
    if let Some(arr) = ids {
        params.insert("ids".to_string(), arr.join(","));
    }
    params
}
