use crate::builder::create::AbstractSignerForCreate;
use crate::builder::proxy::{
    build_proxy_transaction_request, is_proxy_contract_config_valid, ProxyContractConfig,
};
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
use serde_json::json;
// use serde_json::json;

#[derive(Clone, Debug)]
pub struct ContractConfig {
    pub safe: SafeContractConfig,
    pub proxy: ProxyContractConfig,
}

pub struct RelayClient {
    pub relayer_url: String,
    pub chain_id: u64,
    pub contract_config: ContractConfig,
    pub relay_tx_type: RelayerTxType,
    http: HttpClient,
    gas_estimate_rpc: Option<String>,
    signer: Option<Box<dyn AbstractSigner + Send + Sync>>,
    typed_signer: Option<Box<dyn AbstractSignerForCreate + Send + Sync>>,
    builder_signer: Option<BuilderSigner>,
}

impl RelayClient {
    pub fn new(relayer_url: impl Into<String>, chain_id: u64) -> Self {
        Self::new_with_type(relayer_url, chain_id, RelayerTxType::Safe)
    }

    pub fn new_with_type(
        relayer_url: impl Into<String>,
        chain_id: u64,
        relay_tx_type: RelayerTxType,
    ) -> Self {
        let url = relayer_url.into();
        let contract_config = match chain_id {
            137 => ContractConfig {
                proxy: ProxyContractConfig {
                    proxy_factory: "0xaB45c5A4B0c941a2F231C04C3f49182e1A254052".into(),
                    relay_hub: "0xD216153c06E857cD7f72665E0aF1d7D82172F494".into(),
                },
                safe: SafeContractConfig {
                    safe_factory: "0xaacFeEa03eb1561C4e67d661e40682Bd20E3541b".into(),
                    safe_multisend: "0xA238CBeb142c10Ef7Ad8442C6D1f9E89e07e7761".into(),
                },
            },
            80002 => ContractConfig {
                proxy: ProxyContractConfig {
                    proxy_factory: String::new(),
                    relay_hub: String::new(),
                },
                safe: SafeContractConfig {
                    safe_factory: "0xaacFeEa03eb1561C4e67d661e40682Bd20E3541b".into(),
                    safe_multisend: "0xA238CBeb142c10Ef7Ad8442C6D1f9E89e07e7761".into(),
                },
            },
            _ => ContractConfig {
                proxy: ProxyContractConfig {
                    proxy_factory: String::new(),
                    relay_hub: String::new(),
                },
                safe: SafeContractConfig {
                    safe_factory: String::new(),
                    safe_multisend: String::new(),
                },
            },
        };

        Self {
            relayer_url: url.trim_end_matches('/').to_string(),
            chain_id,
            contract_config,
            relay_tx_type,
            http: HttpClient::new(),
            gas_estimate_rpc: None,
            signer: None,
            typed_signer: None,
            builder_signer: None,
        }
    }

    /// Override the relay transaction type (SAFE / PROXY) after construction.
    /// This is useful when you want to pick the type dynamically (e.g., via config/env).
    pub fn with_relay_tx_type(mut self, relay_tx_type: RelayerTxType) -> Self {
        self.relay_tx_type = relay_tx_type;
        self
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

    /// Override the RPC used for gas estimation (defaults to BLOCK_RPC_URL env or polygon-rpc.com).
    pub fn with_gas_estimate_rpc(mut self, rpc: impl Into<String>) -> Self {
        self.gas_estimate_rpc = Some(rpc.into());
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

    /// Best-effort gas estimation for proxy txns, mirroring TS SDK behavior.
    /// Uses BLOCK_RPC_URL env or defaults to https://polygon-rpc.com.
    async fn estimate_gas_limit(&self, from: &str, to: &str, data: &str) -> Result<String> {
        let rpc = if let Some(r) = self.gas_estimate_rpc.as_ref() {
            r.clone()
        } else {
            std::env::var("BLOCK_RPC_URL").unwrap_or_else(|_| "https://polygon-rpc.com".to_string())
        };
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_estimateGas",
            "params": [json!({
                "from": from,
                "to": to,
                "data": data,
            })],
        });

        let resp = self
            .http
            .post(&rpc)
            .json(&payload)
            .send()
            .await
            .map_err(|e| RelayClientError::Http(e.to_string()))?;

        let value: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| RelayClientError::Serde(e.to_string()))?;

        if let Some(err) = value.get("error") {
            return Err(RelayClientError::Http(format!("estimateGas error: {err}")));
        }
        let res = value
            .get("result")
            .and_then(|v| v.as_str())
            .ok_or_else(|| RelayClientError::Http("missing result in estimateGas".into()))?;

        // result is hex string
        let gas_limit = u128::from_str_radix(res.trim_start_matches("0x"), 16)
            .unwrap_or(0)
            .to_string();
        Ok(gas_limit)
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

    pub async fn get_relay_payload(
        &self,
        signer_address: &str,
        signer_type: &str,
    ) -> Result<RelayPayload> {
        self.send(
            GET_RELAY_PAYLOAD,
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

    pub async fn get_deployed(&self, safe_address: &str) -> Result<bool> {
        let resp: GetDeployedResponse = self
            .send(
                GET_DEPLOYED,
                "GET",
                None,
                Some(vec![("address".into(), safe_address.into())]),
                None,
            )
            .await?;
        Ok(resp.deployed)
    }

    pub async fn deploy(&self) -> Result<RelayerTransactionResponse> {
        self.ensure_signer()?;
        let signer = self.signer.as_ref().unwrap();
        let factory = self
            .contract_config
            .safe
            .safe_factory
            .parse()
            .map_err(|_| RelayClientError::InvalidAddress)?;
        let safe_addr = derive_safe(factory, signer.address())?;
        let safe = safe_addr.to_string();

        if self.get_deployed(&safe).await? {
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
        let from = signer.address().to_string();
        let args = SafeCreateTransactionArgs {
            from: from.clone(),
            chain_id: self.chain_id,
            payment_token: "0x0000000000000000000000000000000000000000".into(),
            payment: "0".into(),
            payment_receiver: "0x0000000000000000000000000000000000000000".into(),
        };
        let req = build_safe_create_transaction_request(
            typed.as_ref(),
            &self.contract_config.safe.safe_factory,
            args,
        )
        .await?;
        let payload =
            serde_json::to_string(&req).map_err(|e| RelayClientError::Serde(e.to_string()))?;
        let resp: RelayerTransactionResponse =
            self.authed_post(SUBMIT_TRANSACTION, &payload).await?;
        Ok(resp)
    }

    /// 按 relayTxType 分发执行（对齐 TS execute 行为）
    pub async fn execute_transactions(
        &self,
        txns: Vec<Transaction>,
        metadata: Option<String>,
    ) -> Result<RelayerTransactionResponse> {
        match self.relay_tx_type {
            RelayerTxType::Safe => {
                let safe_txns: Vec<SafeTransaction> = txns
                    .into_iter()
                    .map(|t| SafeTransaction {
                        to: t.to,
                        operation: OperationType::Call,
                        data: t.data,
                        // TS execute() 对 SAFE 分支将 value 固定为 "0"
                        value: "0".to_string(),
                    })
                    .collect();
                self.execute_with_safe(safe_txns, metadata, None).await
            }
            RelayerTxType::Proxy => {
                let proxy_txns: Vec<ProxyTransaction> = txns
                    .into_iter()
                    .map(|t| ProxyTransaction {
                        to: t.to,
                        type_code: CallType::Call,
                        data: t.data,
                        value: t.value,
                    })
                    .collect();
                self.execute_proxy_transactions(proxy_txns, metadata).await
            }
        }
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
        let from = signer.address().to_string();

        // Debug: compare derived safe and provided safe (if any)
        let factory = self
            .contract_config
            .safe
            .safe_factory
            .parse()
            .map_err(|_| RelayClientError::InvalidAddress)?;
        let derived_safe_addr = derive_safe(factory, signer.address())?;
        let derived_safe = derived_safe_addr.to_string();
        let safe_to_check = if let Some(ref provided_safe) = safe_address {
            eprintln!(
                "[RelayClient][execute] derived_safe={} provided_safe={} equal? {}",
                derived_safe,
                provided_safe,
                (derived_safe.to_lowercase() == provided_safe.to_lowercase())
            );
            provided_safe.clone()
        } else {
            eprintln!(
                "[RelayClient][execute] derived_safe={} (no provided safe)",
                derived_safe
            );
            derived_safe.clone()
        };

        // Check if Safe is deployed
        if !self.get_deployed(&safe_to_check).await? {
            return Err(RelayClientError::SafeNotDeployed);
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
        // let mode_env = std::env::var("RELAYER_SIG_MODE").unwrap_or_else(|_| "auto".into());
        // let initial_mode = match mode_env.to_lowercase().as_str() {
        //     "structhash" => SignatureMode::Eip191StructHash,
        //     "digest" => SignatureMode::Eip712Digest,
        //     "eip191_digest" => SignatureMode::Eip191Digest,
        //     _ => SignatureMode::Eip191Digest, // 默认与 TS signMessage(hashTypedData digest) 行为一致
        // };

        // TS implementation: signMessage(hashTypedData(...)) => EIP-191 over the EIP-712 digest
        let initial_mode = SignatureMode::Eip191Digest;

        let mut _last_err: Option<crate::errors::RelayClientError> = None;
        let mut attempt = 0;
        // let max_attempts = if mode_env.to_lowercase() == "auto" {
        //     3
        // } else {
        //     1
        // };
        let max_attempts = 1; // Disable retry loop

        loop {
            attempt += 1;
            let mode = initial_mode;
            // let mode = if attempt == 1 {
            //     initial_mode
            // } else {
            //     match (initial_mode, attempt) {
            //         (SignatureMode::Eip191Digest, 2) => SignatureMode::Eip712Digest,
            //         (SignatureMode::Eip191Digest, 3) => SignatureMode::Eip191StructHash,
            //         (SignatureMode::Eip712Digest, 2) => SignatureMode::Eip191Digest,
            //         (SignatureMode::Eip712Digest, 3) => SignatureMode::Eip191StructHash,
            //         (SignatureMode::Eip191StructHash, 2) => SignatureMode::Eip191Digest,
            //         (SignatureMode::Eip191StructHash, 3) => SignatureMode::Eip712Digest,
            //         _ => initial_mode,
            //     }
            // };

            let req = build_safe_transaction_request(
                signer.as_ref(),
                &args.clone(),
                &self.contract_config.safe,
                metadata.clone(),
                mode,
            )
            .await?;
            let body =
                serde_json::to_string(&req).map_err(|e| RelayClientError::Serde(e.to_string()))?;
            eprintln!("[RelayClient][execute] outbound body: {}", body);
            let res = self.authed_post(SUBMIT_TRANSACTION, &body).await;
            match res {
                Ok(resp) => {
                    eprintln!(
                        "[RelayClient][execute] Transaction submitted successfully using signature mode: {:?}",
                        mode
                    );
                    return Ok(resp);
                }
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

    pub async fn execute_proxy_transactions(
        &self,
        txns: Vec<ProxyTransaction>,
        metadata: Option<String>,
    ) -> Result<RelayerTransactionResponse> {
        self.ensure_signer()?;
        if !is_proxy_contract_config_valid(&self.contract_config.proxy) {
            return Err(RelayClientError::InvalidNetwork);
        }

        let signer = self.signer.as_ref().unwrap();
        let from = signer.address().to_string();

        let relay_payload = self.get_relay_payload(&from, "PROXY").await?;
        let encoded_data = crate::encode::proxy::encode_proxy_transaction_data(&txns);

        // Try to mirror TS SDK behavior: estimate gas if not provided by caller.
        let gas_limit_est = self
            .estimate_gas_limit(
                &from,
                &self.contract_config.proxy.proxy_factory,
                &encoded_data,
            )
            .await
            .ok();

        let args = ProxyTransactionArgs {
            from: from.clone(),
            nonce: relay_payload.nonce.clone(),
            gas_price: "0".to_string(),
            gas_limit: gas_limit_est,
            data: encoded_data,
            relay: relay_payload.address.clone(),
        };

        let req = build_proxy_transaction_request(
            signer.as_ref(),
            &args,
            &self.contract_config.proxy,
            metadata.clone(),
            &txns,
        )
        .await?;

        let body =
            serde_json::to_string(&req).map_err(|e| RelayClientError::Serde(e.to_string()))?;
        self.authed_post(SUBMIT_TRANSACTION, &body).await
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
