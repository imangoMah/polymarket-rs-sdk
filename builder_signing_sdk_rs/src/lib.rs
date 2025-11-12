use base64::Engine;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;

pub type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, Debug)]
pub struct BuilderApiKeyCreds {
    pub key: String,
    pub secret: String, // base64-encoded secret
    pub passphrase: String,
}

/// Build URL-safe base64 HMAC-SHA256 signature (keeps '=' padding)
pub fn build_hmac_signature(
    secret_b64: &str,
    timestamp: i64,
    method: &str,
    request_path: &str,
    body: Option<&str>,
) -> Result<String, String> {
    let mut message = format!("{}{}{}", timestamp, method, request_path);
    if let Some(b) = body {
        message.push_str(b);
    }

    let secret_bytes = base64::engine::general_purpose::STANDARD
        .decode(secret_b64)
        .map_err(|e| e.to_string())?;

    let mut mac = HmacSha256::new_from_slice(&secret_bytes).map_err(|e| e.to_string())?;
    mac.update(message.as_bytes());
    let sig_bytes = mac.finalize().into_bytes();
    let sig_b64 = base64::engine::general_purpose::STANDARD.encode(sig_bytes);
    // Make URL-safe but keep '='
    let sig_url_safe = sig_b64.replace('+', "-").replace('/', "_");
    Ok(sig_url_safe)
}

pub struct BuilderSigner {
    pub creds: BuilderApiKeyCreds,
}

impl BuilderSigner {
    pub fn new(creds: BuilderApiKeyCreds) -> Self {
        Self { creds }
    }

    /// Create headers map equivalent to TS BuilderSigner.createBuilderHeaderPayload
    pub fn create_builder_header_payload(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
        timestamp: Option<i64>,
    ) -> Result<HashMap<String, String>, String> {
        let ts = timestamp.unwrap_or_else(|| {
            // seconds
            (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()) as i64
        });
        let sig = build_hmac_signature(&self.creds.secret, ts, method, path, body)?;
        let mut map = HashMap::new();
        map.insert("POLY_BUILDER_API_KEY".to_string(), self.creds.key.clone());
        map.insert(
            "POLY_BUILDER_PASSPHRASE".to_string(),
            self.creds.passphrase.clone(),
        );
        map.insert("POLY_BUILDER_SIGNATURE".to_string(), sig);
        map.insert("POLY_BUILDER_TIMESTAMP".to_string(), ts.to_string());
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_signature_url_safe() {
        // secret: base64 of 'secret'
        let secret_b64 = base64::engine::general_purpose::STANDARD.encode(b"secret");
        let sig = build_hmac_signature(
            secret_b64.as_str(),
            1700000000,
            "POST",
            "/v1/x",
            Some("{\"a\":1}"),
        )
        .expect("sig");
        // Should not contain '+' or '/'
        assert!(!sig.contains('+'));
        assert!(!sig.contains('/'));
        // Keep '=' padding allowed (may or may not present depending on length)
    }
}
