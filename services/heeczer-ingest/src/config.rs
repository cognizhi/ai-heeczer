//! Layered configuration for the ingestion service.
//!
//! Resolution order (last wins): defaults → `heeczer.toml` → env vars.
//! All env vars are prefixed `HEECZER_`.

use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

use crate::state::{
    AuthConfig, IdempotencyConfig, PayloadLimitConfig, QuotaConfig, RateLimitConfig, WorkerConfig,
};

/// Top-level service configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// HTTP listen address. Default: `0.0.0.0:8080`.
    pub listen: String,
    /// SQLite database URL used by the HTTP service runtime.
    /// Default: `sqlite:heeczer.db?mode=rwc`.
    pub database_url: String,
    /// Maximum request body size in bytes. Default: 1 MiB.
    pub max_body_bytes: usize,
    /// API-key authentication settings.
    pub auth: AuthConfig,
    /// Per-API-key rate-limit settings.
    pub rate_limit: RateLimitConfig,
    /// Payload-size limits.
    pub payload_limits: PayloadLimitConfig,
    /// Batch idempotency settings.
    pub idempotency: IdempotencyConfig,
    /// Workspace quota defaults.
    pub quotas: QuotaConfig,
    /// Queue worker settings.
    pub worker: WorkerConfig,
    /// Feature flags.
    pub features: FeaturesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeaturesConfig {
    /// Enable `/v1/test/*` endpoints (ADR-0012).
    pub test_orchestration: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen: "0.0.0.0:8080".into(),
            database_url: "sqlite:heeczer.db?mode=rwc".into(),
            max_body_bytes: 1024 * 1024,
            auth: AuthConfig::default(),
            rate_limit: RateLimitConfig::default(),
            payload_limits: PayloadLimitConfig::default(),
            idempotency: IdempotencyConfig::default(),
            quotas: QuotaConfig::default(),
            worker: WorkerConfig::default(),
            features: FeaturesConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from the standard layered sources.
    ///
    /// Errors if required fields are missing or types mismatch.
    pub fn load() -> Result<Self, Box<figment::Error>> {
        Figment::from(Serialized::defaults(Config::default()))
            .merge(Toml::file("heeczer.toml"))
            .merge(Env::prefixed("HEECZER_").split("__"))
            .extract()
            .map_err(Box::new)
    }
}
