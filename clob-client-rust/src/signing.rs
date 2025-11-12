use crate::errors::ClobError;
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Trait representing a signer that can perform EIP-712 typed data signing.
/// In a full integration this will be implemented by an ethers-rs signer wrapper.
#[async_trait]
pub trait Eip712Signer: Send + Sync {
    /// Return the address for this signer (e.g. Ethereum address hex)
    async fn get_address(&self) -> Result<String, ClobError>;

    /// Sign a typed data payload and return the signature as hex or 0x-prefixed string.
    /// The concrete implementation decides canonical format.
    async fn sign_typed_data(
        &self,
        domain: &str,
        types: &str,
        value: &str,
    ) -> Result<String, ClobError>;
}

/// Build the canonical Polymarket CLOB EIP712 signature.
/// This function delegates to the provided `Eip712Signer` implementation.
pub async fn build_clob_eip712_signature<S: Eip712Signer>(
    signer: &S,
    chain_id: i64,
    timestamp: u64,
    nonce: u64,
) -> Result<String, ClobError> {
    let address = signer.get_address().await?;
    if std::env::var("CLOB_DEBUG_FULL").is_ok() || std::env::var("CLOB_DEBUG_RAW").is_ok() {
        eprintln!(
            "[EIP712 DEBUG] signer.get_address() => '{}' (len={})",
            address,
            address.len()
        );
    }
    let ts = format!("{}", timestamp);

    // Build minimal canonical domain/types/value as JSON strings. The signer
    // implementation is expected to interpret them appropriately.
    // Align domain fields exactly with TS client (name, version, chainId)
    let domain = format!(
        r#"{{"name":"ClobAuthDomain","version":"1","chainId":{}}}"#,
        chain_id
    );

    // Types: Match TypeScript SDK exactly - NO EIP712Domain type, only ClobAuth
    // Why we must NOT inject EIP712Domain here:
    // - TypeScript uses _signTypedData(domain, types, value) where `types` does NOT include EIP712Domain.
    // - 手动把 EIP712Domain 注入到 types 会改变最终的 struct hash，后端的验签也据此失败（会返回 Invalid L1 headers）。
    // - 因此这里保持与 TS 完全一致：types 只包含 ClobAuth，domain 单独传入。
    let types = r#"{
        "ClobAuth": [
            {"name": "address", "type": "address"},
            {"name": "timestamp", "type": "string"},
            {"name": "nonce", "type": "uint256"},
            {"name": "message", "type": "string"}
        ]
    }"#;

    // Value: include address, timestamp, nonce, and canonical message
    // The canonical message used in TS is MSG_TO_SIGN; for portability the
    // caller/Signer implementation may embed the same literal.
    // NOTE: TypeScript clob-client uses MSG_TO_SIGN = "This message attests that I control the given wallet"
    // If we use a different literal (e.g. "POLY"), the backend will reject the signature as invalid.
    // Align with canonical string for parity.
    const MSG_TO_SIGN: &str = "This message attests that I control the given wallet";
    let value = format!(
        r#"{{"address":"{}","timestamp":"{}","nonce":{},"message":"{}"}}"#,
        address, ts, nonce, MSG_TO_SIGN
    );

    let mut sig = signer.sign_typed_data(&domain, types, &value).await?;
    // 重要：确保签名以 0x 开头
    // 原因：ethers.js 的 _signTypedData 返回 0x 前缀的十六进制签名；后端也按该格式校验。
    // 若缺少 0x，服务端会拒绝（此前就因此被判定为 Invalid L1 headers）。
    if !sig.starts_with("0x") {
        sig = format!("0x{}", sig);
    }
    if std::env::var("CLOB_DEBUG_FULL").is_ok() || std::env::var("CLOB_DEBUG_RAW").is_ok() {
        eprintln!("[EIP712 DEBUG] final signature (0x-prefixed): {}", sig);
    }
    Ok(sig)
}

/// Build the canonical Polymarket CLOB HMAC signature
pub fn build_poly_hmac_signature(
    secret_b64: &str,
    timestamp: u64,
    method: &str,
    request_path: &str,
    body: Option<&str>,
) -> Result<String, ClobError> {
    let mut message = format!("{}{}{}", timestamp, method, request_path);
    if let Some(b) = body {
        message.push_str(b);
    }

    // secret is base64 or base64url encoded (server returns secrets containing '_' and '-')
    // We must accept URL-safe alphabet: '-' for '+', '_' for '/'. Padding '=' may be absent.
    let normalized = secret_b64.replace('-', "+").replace('_', "/");
    // Add padding if missing (length % 4 != 0)
    let padded = match normalized.len() % 4 {
        0 => normalized.clone(),
        2 => format!("{}==", normalized),
        3 => format!("{}=", normalized),
        _ => normalized.clone(),
    };
    let secret = general_purpose::STANDARD
        .decode(padded.as_bytes())
        .map_err(|e| ClobError::Other(format!("invalid base64(url) secret: {}", e)))?;

    let mut mac = HmacSha256::new_from_slice(&secret)
        .map_err(|e| ClobError::Other(format!("hmac init error: {}", e)))?;
    mac.update(message.as_bytes());
    let result = mac.finalize().into_bytes();

    // Base64 encode the raw HMAC result
    let sig = general_purpose::STANDARD.encode(result);
    // Produce URL-safe form for transmission (replace '+' -> '-', '/' -> '_')
    let sig_url = sig.replace('+', "-").replace('/', "_");
    Ok(sig_url)
}
