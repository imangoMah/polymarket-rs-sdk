use crate::builder::create::AbstractSignerForCreate;
use crate::builder::safe::{AbstractSigner, SafeContractConfig, SignatureMode};
use crate::builder::{
    build_safe_create_transaction_request, build_safe_transaction_request, derive_safe,
};
use crate::endpoints::*;
use crate::errors::{RelayClientError, Result};
use crate::types::*;
use crate::utils::sleep_ms;
use builder_signing_sdk_rs::{BuilderApiKeyCreds, BuilderSigner};
use reqwest::Client as HttpClient;
// use serde_json::json;

pub struct RelayClient {
    pub relayer_url: String,
    pub chain_id: u64,
    pub contract_config: SafeContractConfig,
    http: HttpClient,
    signer: Option<Box<dyn AbstractSigner + Send + Sync>>,
    typed_signer: Option<Box<dyn AbstractSignerForCreate + Send + Sync>>,
    builder_signer: Option<BuilderSigner>,
}

impl RelayClient {
    pub fn new(relayer_url: impl Into<String>, chain_id: u64) -> Self {
        let url = relayer_url.into();
        let contract_config = match chain_id {
            137 | 80002 => SafeContractConfig {
                safe_factory: "0xaacFeEa03eb1561C4e67d661e40682Bd20E3541b".into(),
                safe_multisend: "0xA238CBeb142c10Ef7Ad8442C6D1f9E89e07e7761".into(),
            },
            _ => SafeContractConfig {
                safe_factory: String::new(),
                safe_multisend: String::new(),
            },
        };
        Self {
            relayer_url: url.trim_end_matches('/').to_string(),
            chain_id,
            contract_config,
            http: HttpClient::new(),
            signer: None,
            typed_signer: None,
            builder_signer: None,
        }
    }

    pub fn with_signer(
        mut self,
        signer: Box<dyn AbstractSigner + Send + Sync>,
        typed: Box<dyn AbstractSignerForCreate + Send + Sync>,
    ) -> Self {
        self.signer = Some(signer);
        self.typed_signer = Some(typed);
        self
    }

    pub fn with_builder_api_key(mut self, creds: BuilderApiKeyCreds) -> Self {
        self.builder_signer = Some(builder_signing_sdk_rs::BuilderSigner::new(creds));
        self
    }

    async fn send<T: for<'de> serde::Deserialize<'de>>(
        &self,
        path: &str,
        method: &str,
        body: Option<String>,
        params: Option<Vec<(String, String)>>,
        builder_headers: Option<&std::collections::HashMap<String, String>>,
    ) -> Result<T> {
        let url = format!("{}{}", self.relayer_url, path);
        let mut req = match method {
            "GET" => self.http.get(&url),
            "POST" => self.http.post(&url),
            _ => return Err(RelayClientError::Http("unsupported method".into())),
        };
        if let Some(p) = params {
            req = req.query(&p);
        }
        if let Some(b) = body {
            req = req.body(b);
        }
        if let Some(h) = builder_headers {
            for (k, v) in h {
                req = req.header(k, v);
            }
        }
        let resp = req
            .send()
            .await
            .map_err(|e| RelayClientError::Http(e.to_string()))?;
        let status = resp.status();
        if !status.is_success() {
            // Try to include response body for diagnostics
            let text = resp.text().await.unwrap_or_default();
            let snippet = if text.len() > 512 {
                &text[..512]
            } else {
                &text
            };
            return Err(RelayClientError::Http(format!(
                "status {} body: {}",
                status, snippet
            )));
        }
        resp.json::<T>()
            .await
            .map_err(|e| RelayClientError::Serde(e.to_string()))
    }

    pub async fn get_nonce(&self, signer_address: &str, signer_type: &str) -> Result<NoncePayload> {
        self.send(
            GET_NONCE,
            "GET",
            None,
            Some(vec![
                ("address".into(), signer_address.into()),
                ("type".into(), signer_type.into()),
            ]),
            None,
        )
        .await
    }

    pub async fn get_transaction(&self, transaction_id: &str) -> Result<Vec<RelayerTransaction>> {
        self.send(
            GET_TRANSACTION,
            "GET",
            None,
            Some(vec![("id".into(), transaction_id.into())]),
            None,
        )
        .await
    }

    pub async fn get_transactions(&self) -> Result<Vec<RelayerTransaction>> {
        self.authed_get(GET_TRANSACTIONS).await
    }

    async fn authed_get<T: for<'de> serde::Deserialize<'de>>(&self, path: &str) -> Result<T> {
        if let Some(bs) = &self.builder_signer {
            let headers = bs
                .create_builder_header_payload("GET", path, None, None)
                .map_err(RelayClientError::Http)?;
            return self.send(path, "GET", None, None, Some(&headers)).await;
        }
        self.send(path, "GET", None, None, None).await
    }

    async fn authed_post<T: for<'de> serde::Deserialize<'de>>(
        &self,
        path: &str,
        body: &str,
    ) -> Result<T> {
        if let Some(bs) = &self.builder_signer {
            let headers = bs
                .create_builder_header_payload("POST", path, Some(body), None)
                .map_err(RelayClientError::Http)?;
            return self
                .send(path, "POST", Some(body.to_string()), None, Some(&headers))
                .await;
        }
        self.send(path, "POST", Some(body.to_string()), None, None)
            .await
    }

    fn ensure_signer(&self) -> Result<()> {
        if self.signer.is_none() {
            Err(RelayClientError::SignerUnavailable)
        } else {
            Ok(())
        }
    }

    pub async fn deploy(&self) -> Result<RelayerTransactionResponse> {
        self.ensure_signer()?;
        let signer = self.signer.as_ref().unwrap();
        let addr = signer.get_address()?;
        let safe = derive_safe(&addr, &self.contract_config.safe_factory);
        let deployed: GetDeployedResponse = self
            .send(
                GET_DEPLOYED,
                "GET",
                None,
                Some(vec![("address".into(), safe.clone())]),
                None,
            )
            .await?;
        if deployed.deployed {
            return Err(RelayClientError::SafeDeployed);
        }
        self._deploy().await
    }

    async fn _deploy(&self) -> Result<RelayerTransactionResponse> {
        self.ensure_signer()?;
        let signer = self.signer.as_ref().unwrap();
        let typed = self
            .typed_signer
            .as_ref()
            .ok_or(RelayClientError::SignerUnavailable)?;
        let from = signer.get_address()?;
        let args = SafeCreateTransactionArgs {
            from: from.clone(),
            chain_id: self.chain_id,
            payment_token: "0x0000000000000000000000000000000000000000".into(),
            payment: "0".into(),
            payment_receiver: "0x0000000000000000000000000000000000000000".into(),
        };
        let req = build_safe_create_transaction_request(
            typed.as_ref(),
            &self.contract_config.safe_factory,
            args,
        )
        .await?;
        let payload =
            serde_json::to_string(&req).map_err(|e| RelayClientError::Serde(e.to_string()))?;
        let resp: RelayerTransactionResponse =
            self.authed_post(SUBMIT_TRANSACTION, &payload).await?;
        Ok(resp)
    }

    pub async fn execute(
        &self,
        txns: Vec<SafeTransaction>,
        metadata: Option<String>,
    ) -> Result<RelayerTransactionResponse> {
        self.execute_with_safe(txns, metadata, None).await
    }

    /// Execute transactions with an optional explicit Safe address
    ///
    /// If `safe_address` is provided, it will be used directly instead of deriving from signer address.
    /// This is useful when the Safe address is already known (e.g., from Polymarket account).
    pub async fn execute_with_safe(
        &self,
        txns: Vec<SafeTransaction>,
        metadata: Option<String>,
        safe_address: Option<String>,
    ) -> Result<RelayerTransactionResponse> {
        self.ensure_signer()?;
        let signer = self.signer.as_ref().unwrap();
        let from = signer.get_address()?;

        // Debug: compare derived safe and provided safe (if any)
        let derived_safe = derive_safe(&from, &self.contract_config.safe_factory);
        if let Some(ref provided_safe) = safe_address {
            eprintln!(
                "[RelayClient][execute] derived_safe={} provided_safe={} equal? {}",
                derived_safe,
                provided_safe,
                (derived_safe.to_lowercase() == provided_safe.to_lowercase())
            );
        } else {
            eprintln!(
                "[RelayClient][execute] derived_safe={} (no provided safe)",
                derived_safe
            );
        }

        // Use EOA `from` to fetch nonce, matching the TypeScript SDK behaviour.
        // The relayer expects the nonce query to be done against the signer EOA
        // even when a proxyWallet (Safe address) is provided in the request.
        let nonce_payload = self.get_nonce(&from, "SAFE").await?;

        let args = SafeTransactionArgs {
            from: from.clone(),
            nonce: nonce_payload.nonce.clone(),
            chain_id: self.chain_id,
            transactions: txns,
            safe_address,
        };
        // Resolve signature mode from env
        let mode_env = std::env::var("RELAYER_SIG_MODE").unwrap_or_else(|_| "auto".into());
        let initial_mode = match mode_env.to_lowercase().as_str() {
            "structhash" => SignatureMode::Eip191StructHash,
            "digest" => SignatureMode::Eip712Digest,
            "eip191_digest" => SignatureMode::Eip191Digest,
            _ => SignatureMode::Eip191Digest, // 默认与 TS signMessage(hashTypedData digest) 行为一致
        };

        let mut _last_err: Option<crate::errors::RelayClientError> = None;
        let mut attempt = 0;
        let max_attempts = if mode_env.to_lowercase() == "auto" {
            3
        } else {
            1
        };
        loop {
            attempt += 1;
            let mode = if attempt == 1 {
                initial_mode
            } else {
                match (initial_mode, attempt) {
                    (SignatureMode::Eip191Digest, 2) => SignatureMode::Eip712Digest,
                    (SignatureMode::Eip191Digest, 3) => SignatureMode::Eip191StructHash,
                    (SignatureMode::Eip712Digest, 2) => SignatureMode::Eip191Digest,
                    (SignatureMode::Eip712Digest, 3) => SignatureMode::Eip191StructHash,
                    (SignatureMode::Eip191StructHash, 2) => SignatureMode::Eip191Digest,
                    (SignatureMode::Eip191StructHash, 3) => SignatureMode::Eip712Digest,
                    _ => initial_mode,
                }
            };

            let req = build_safe_transaction_request(
                signer.as_ref(),
                args.clone(),
                self.contract_config.clone(),
                metadata.clone(),
                mode,
            )
            .await?;
            let body =
                serde_json::to_string(&req).map_err(|e| RelayClientError::Serde(e.to_string()))?;
            eprintln!("[RelayClient][execute] outbound body: {}", body);
            let res = self.authed_post(SUBMIT_TRANSACTION, &body).await;
            match res {
                Ok(resp) => return Ok(resp),
                Err(e) => {
                    let msg = format!("{}", e);
                    let is_invalid_sig =
                        msg.contains("invalid signature") || msg.contains("validation error");
                    if attempt < max_attempts && is_invalid_sig {
                        eprintln!("[RelayClient][execute] invalid signature, retrying with alternate signature mode...");
                        _last_err = Some(e);
                        continue;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    }

    pub async fn poll_until_state(
        &self,
        transaction_id: &str,
        states: &[RelayerTransactionState],
        fail_state: Option<RelayerTransactionState>,
        max_polls: usize,
        poll_freq_ms: u64,
    ) -> Result<Option<RelayerTransaction>> {
        let mut count = 0usize;
        while count < max_polls {
            let txns = self.get_transaction(transaction_id).await?;
            if let Some(first) = txns.get(0) {
                if states.iter().any(|s| first.state == format!("{:?}", s)) {
                    return Ok(Some(first.clone()));
                }
                if let Some(ref fail) = fail_state {
                    if first.state == format!("{:?}", fail) {
                        return Ok(None);
                    }
                }
            }
            count += 1;
            sleep_ms(poll_freq_ms).await;
        }
        Ok(None)
    }
}
