//! Client that signs requests the way the gateway expects.

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use reqwest::{Method, StatusCode};
use serde_json::Value;

use crate::auth::{self, HEADER_API_KEY, HEADER_SIGNATURE, HEADER_TIMESTAMP};

pub struct GatewayClient {
    base_url: String,
    api_key: String,
    api_secret: String,
    http: reqwest::Client,
}

pub struct ApiResponse {
    pub status: StatusCode,
    pub request_id: Option<String>,
    pub body: Value,
}

impl GatewayClient {
    pub fn new(base_url: &str, api_key: &str, api_secret: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            api_secret: api_secret.to_string(),
            http: reqwest::Client::new(),
        }
    }

    /// Calls `/api/v1/<service>/<path>`. `path` may include a query string.
    pub async fn call(
        &self,
        method: Method,
        service: &str,
        path: &str,
        body: Option<&Value>,
    ) -> Result<ApiResponse> {
        let path_and_query = format!("/api/v1/{service}/{}", path.trim_start_matches('/'));
        let body = match body {
            Some(v) => serde_json::to_vec(v)?,
            None => Vec::new(),
        };
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis()
            .to_string();
        let signature = auth::sign(
            &self.api_secret,
            &auth::signing_payload(&timestamp, method.as_str(), &path_and_query, &body),
        );

        let mut req = self
            .http
            .request(method, format!("{}{}", self.base_url, path_and_query))
            .header(HEADER_API_KEY, &self.api_key)
            .header(HEADER_TIMESTAMP, timestamp)
            .header(HEADER_SIGNATURE, signature);
        if !body.is_empty() {
            req = req
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(body);
        }

        let resp = req.send().await.context("sending request to gateway")?;
        let status = resp.status();
        let request_id = resp
            .headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let bytes = resp.bytes().await?;
        let body = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes)
                .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into()))
        };
        Ok(ApiResponse {
            status,
            request_id,
            body,
        })
    }

    pub async fn market_price(&self, symbol: &str) -> Result<ApiResponse> {
        self.call(
            Method::GET,
            "market",
            &format!("price?symbol={symbol}"),
            None,
        )
        .await
    }

    pub async fn trading_quote(
        &self,
        chain_id: &str,
        from_token: &str,
        to_token: &str,
        amount: &str,
    ) -> Result<ApiResponse> {
        let path = format!(
            "quote?chainId={chain_id}&fromToken={from_token}&toToken={to_token}&amount={amount}"
        );
        self.call(Method::GET, "trading", &path, None).await
    }

    pub async fn transaction_simulate(&self, tx: &Value) -> Result<ApiResponse> {
        self.call(Method::POST, "transaction", "simulate", Some(tx))
            .await
    }

    pub async fn wallet_balance(&self, address: &str) -> Result<ApiResponse> {
        self.call(
            Method::GET,
            "wallet",
            &format!("balance?address={address}"),
            None,
        )
        .await
    }

    pub async fn defi_positions(&self, address: &str) -> Result<ApiResponse> {
        self.call(
            Method::GET,
            "defi",
            &format!("positions?address={address}"),
            None,
        )
        .await
    }
}
