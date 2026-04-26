//! Shared application state for the ingestion service.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use sqlx_sqlite::SqlitePool;

/// Authenticated caller context inserted by the API-key middleware.
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub workspace_id: String,
    pub api_key_id: String,
    pub authenticated: bool,
}

impl AuthContext {
    pub fn anonymous() -> Self {
        Self {
            workspace_id: String::new(),
            api_key_id: "anonymous".into(),
            authenticated: false,
        }
    }
}

/// Feature flags for runtime-gated surfaces. Defaults are production-safe
/// (everything optional is off).
#[derive(Debug, Clone, Copy, Default)]
pub struct Features {
    /// Enable the `/v1/test/*` endpoints used by the dashboard
    /// test-orchestration view (ADR-0012). Off by default.
    pub test_orchestration: bool,
}

/// API-key authentication settings.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Require `x-heeczer-api-key` on protected API routes.
    pub enabled: bool,
}

impl AuthConfig {
    pub fn disabled() -> Self {
        Self { enabled: false }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Per-API-key token-bucket settings.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Tokens replenished per second.
    pub refill_per_second: u32,
    /// Maximum burst size.
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            refill_per_second: 17,
            burst_size: 200,
        }
    }
}

/// Payload-size limits. PRD §12.18 defaults are 64 KiB/event and 1 MiB/batch.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PayloadLimitConfig {
    pub event_bytes: usize,
    pub batch_bytes: usize,
}

impl Default for PayloadLimitConfig {
    fn default() -> Self {
        Self {
            event_bytes: 64 * 1024,
            batch_bytes: 1024 * 1024,
        }
    }
}

/// Batch idempotency settings.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct IdempotencyConfig {
    /// Retention window for replayable `Idempotency-Key` responses.
    pub retention_hours: i64,
}

impl Default for IdempotencyConfig {
    fn default() -> Self {
        Self {
            retention_hours: 24,
        }
    }
}

/// Workspace quota defaults. Per-workspace overrides live in
/// `heec_workspaces.settings_json.daily_event_quota`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct QuotaConfig {
    pub daily_events: i64,
}

impl Default for QuotaConfig {
    fn default() -> Self {
        Self {
            daily_events: 5_000_000,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WorkerConfig {
    pub enabled: bool,
    pub max_attempts: i64,
    pub idle_backoff_ms: u64,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_attempts: 5,
            idle_backoff_ms: 250,
        }
    }
}

#[derive(Debug)]
struct Bucket {
    tokens: f64,
    updated_at: Instant,
}

/// Small in-process token bucket keyed by API key id. The router also wires
/// `tower-governor` for the protected route tree; this stateful bucket gives us
/// deterministic quota headers and test coverage tied to authenticated keys.
#[derive(Debug, Clone, Default)]
pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, Bucket>>>,
}

#[derive(Debug, Clone, Copy)]
pub struct RateLimitDecision {
    pub limit: u32,
    pub remaining: u32,
    pub retry_after_seconds: u64,
}

impl RateLimiter {
    pub fn check(
        &self,
        key: &str,
        config: RateLimitConfig,
    ) -> Result<RateLimitDecision, RateLimitDecision> {
        let now = Instant::now();
        let mut buckets = self.buckets.lock().expect("rate limiter mutex poisoned");
        let bucket = buckets.entry(key.to_owned()).or_insert(Bucket {
            tokens: f64::from(config.burst_size),
            updated_at: now,
        });
        let elapsed = now.duration_since(bucket.updated_at).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * f64::from(config.refill_per_second))
            .min(f64::from(config.burst_size));
        bucket.updated_at = now;

        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            Ok(RateLimitDecision {
                limit: config.burst_size,
                remaining: bucket.tokens.floor() as u32,
                retry_after_seconds: 0,
            })
        } else {
            let retry_after_seconds = if config.refill_per_second == 0 {
                60
            } else {
                (1.0 / f64::from(config.refill_per_second)).ceil() as u64
            };
            Err(RateLimitDecision {
                limit: config.burst_size,
                remaining: 0,
                retry_after_seconds: retry_after_seconds.max(1),
            })
        }
    }
}

/// Application state shared across handlers via axum's `State` extractor.
#[derive(Debug, Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub features: Features,
    pub auth: AuthConfig,
    pub rate_limit: RateLimitConfig,
    pub payload_limits: PayloadLimitConfig,
    pub idempotency: IdempotencyConfig,
    pub quotas: QuotaConfig,
    pub worker: WorkerConfig,
    pub rate_limiter: RateLimiter,
}

impl AppState {
    pub fn new(pool: SqlitePool, features: Features) -> Self {
        Self {
            pool,
            features,
            auth: AuthConfig::disabled(),
            rate_limit: RateLimitConfig::default(),
            payload_limits: PayloadLimitConfig::default(),
            idempotency: IdempotencyConfig::default(),
            quotas: QuotaConfig::default(),
            worker: WorkerConfig::default(),
            rate_limiter: RateLimiter::default(),
        }
    }
}
