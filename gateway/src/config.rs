use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

/// Backend services the gateway can route to, addressed as `/api/v1/<service>/...`.
pub const SERVICES: [&str; 5] = ["market", "trading", "transaction", "wallet", "defi"];

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// API key -> secret used to verify request signatures.
    pub keys: HashMap<String, String>,
    /// Service name -> upstream base URL.
    pub upstreams: HashMap<String, String>,
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
    /// Maximum allowed distance between the request timestamp and server time.
    #[serde(default = "default_recv_window_ms")]
    pub recv_window_ms: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitConfig {
    pub requests_per_second: f64,
    pub burst: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 10.0,
            burst: 20,
        }
    }
}

fn default_recv_window_ms() -> u64 {
    5_000
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("reading config {}", path.display()))?;
        let config: Config = serde_json::from_str(&raw)
            .with_context(|| format!("parsing config {}", path.display()))?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        if self.keys.is_empty() {
            bail!("config must define at least one API key");
        }
        for (service, url) in &self.upstreams {
            if !SERVICES.contains(&service.as_str()) {
                bail!("unknown service '{service}' in upstreams (expected one of {SERVICES:?})");
            }
            url::Url::parse(url).with_context(|| format!("invalid upstream URL for {service}"))?;
        }
        if self.rate_limit.requests_per_second <= 0.0 || self.rate_limit.burst == 0 {
            bail!("rateLimit.requestsPerSecond and rateLimit.burst must be positive");
        }
        Ok(())
    }
}
