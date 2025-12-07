use crate::errors::ClobError;
use crate::types::ApiKeyCreds;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Type alias for header map
pub type Headers = HashMap<String, String>;

/// Build L1 headers using EIP-712 typed-data signing via the Eip712Signer trait.
pub async fn create_l1_headers<S: crate::signing::Eip712Signer + Send + Sync>(
    signer: &S,
    chain_id: i32,
    nonce: Option<u64>,
    timestamp: Option<u64>,
) -> Result<Headers, ClobError> {
    // 与官方 TypeScript SDK 对齐：使用“秒”级时间戳 Math.floor(Date.now()/1000)
    // 如果外部传入则直接使用；否则获取当前秒。
    let ts = timestamp.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });
    let n = nonce.unwrap_or(0);

    let address = signer.get_address().await?;

    // Use EIP-712 signing helper implemented in `signing.rs`.
    let sig = crate::signing::build_clob_eip712_signature(signer, chain_id as i64, ts, n)
        .await
        .map_err(|e| ClobError::Other(format!("failed to build eip712 signature: {}", e)))?;

    let mut headers = Headers::new();
    headers.insert("POLY_ADDRESS".to_string(), address);
    headers.insert("POLY_SIGNATURE".to_string(), sig);
    headers.insert("POLY_TIMESTAMP".to_string(), ts.to_string());
    headers.insert("POLY_NONCE".to_string(), n.to_string());
    Ok(headers)
}

/// Build L2 headers using HMAC signature (POLY API HMAC) along with API key/passphrase.
pub async fn create_l2_headers<S: crate::signing::Eip712Signer + Send + Sync>(
    signer: &S,
    creds: &ApiKeyCreds,
    method: &str,
    request_path: &str,
    body: Option<&str>,
    timestamp: Option<u64>,
) -> Result<Headers, ClobError> {
    // 对齐 TS：秒级时间戳，避免与后端验证窗口不一致。
    let ts = timestamp.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });
    let address = signer.get_address().await?;

    // creds.secret is expected to be base64 encoded; build HMAC signature.
    let sig = match crate::signing::build_poly_hmac_signature(
        &creds.secret,
        ts,
        method,
        request_path,
        body,
    ) {
        Ok(s) => s,
        Err(e) => {
            // If HMAC calculation fails (e.g., secret not base64), return an error to surface configuration problems.
            return Err(ClobError::Other(format!(
                "failed to build hmac signature: {}",
                e
            )));
        }
    };

    // 调试输出（受环境变量控制），避免在生产泄露敏感信息。
    // 仅在设置 CLOB_DEBUG_FULL 或 CLOB_DEBUG_RAW 时输出。
    if std::env::var("CLOB_DEBUG_FULL").is_ok() || std::env::var("CLOB_DEBUG_RAW").is_ok() {
        let preview_secret = if creds.secret.len() > 6 {
            format!("{}***", &creds.secret[..6])
        } else {
            "***".to_string()
        };
        let payload_preview = {
            let mut base = format!("{}{}{}", ts, method, request_path);
            if let Some(b) = body {
                base.push_str(b);
            }
            // 避免日志过长，只截取前后 200 字符
            if base.len() > 420 {
                format!("{} ... {}", &base[..200], &base[base.len() - 200..])
            } else {
                base
            }
        };
        eprintln!(
            "[L2 DEBUG] ts(sec)={} method={} path={} secret(b64-prefix)={} sig={} payload_part={} body_len={}",
            ts,
            method,
            request_path,
            preview_secret,
            sig,
            payload_preview,
            body.map(|b| b.len()).unwrap_or(0)
        );
    }

    let mut headers = Headers::new();
    headers.insert("POLY_ADDRESS".to_string(), address);
    headers.insert("POLY_SIGNATURE".to_string(), sig);
    headers.insert("POLY_TIMESTAMP".to_string(), ts.to_string());
    headers.insert("POLY_API_KEY".to_string(), creds.key.clone());
    headers.insert("POLY_PASSPHRASE".to_string(), creds.passphrase.clone());
    Ok(headers)
}

pub fn inject_builder_headers(
    mut l2: Headers,
    builder: &std::collections::HashMap<String, String>,
) -> Headers {
    for (k, v) in builder.iter() {
        l2.insert(k.clone(), v.clone());
    }
    l2
}
