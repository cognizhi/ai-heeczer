//! ai-heeczer ingestion service binary entrypoint.
//!
//! See ADR-0005 (language choice) and plan 0004 for the larger contract.

use std::net::SocketAddr;

use anyhow::Context;
use heeczer_ingest::{build_router, AppState, Config, Features};

/// Redact the password component of a database DSN so it is safe to log.
/// Returns the DSN unchanged if it cannot be parsed as a URL.
fn redact_dsn(dsn: &str) -> String {
    // Fast path: no "//" means it's not a URL (e.g. "sqlite::memory:").
    if !dsn.contains("://") {
        return dsn.to_owned();
    }
    // Replace ://user:password@ with ://user:***@
    if let Some(at_pos) = dsn.rfind('@') {
        let scheme_end = dsn.find("://").map_or(0, |i| i + 3);
        let authority = &dsn[scheme_end..at_pos];
        if let Some(colon) = authority.find(':') {
            let user = &authority[..colon];
            let prefix = &dsn[..scheme_end];
            let suffix = &dsn[at_pos..];
            return format!("{prefix}{user}:***{suffix}");
        }
    }
    dsn.to_owned()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cfg = Config::load().context("loading configuration")?;
    let pool = heeczer_storage::sqlite::open(&cfg.database_url)
        .await
        .with_context(|| {
            // Redact password component so credentials don't appear in logs.
            let redacted = redact_dsn(&cfg.database_url);
            format!("opening database at {redacted}")
        })?;
    heeczer_storage::sqlite::migrate(&pool).await?;

    let features = Features {
        test_orchestration: cfg.features.test_orchestration,
    };
    let state = AppState {
        pool,
        features,
        auth: cfg.auth,
        rate_limit: cfg.rate_limit,
        payload_limits: cfg.payload_limits,
        idempotency: cfg.idempotency,
        quotas: cfg.quotas,
        worker: cfg.worker,
        rate_limiter: heeczer_ingest::state::RateLimiter::default(),
    };

    let addr: SocketAddr = cfg.listen.parse().context("parsing listen address")?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("heeczer-ingest listening on {addr}");
    axum::serve(listener, build_router(state)).await?;
    Ok(())
}
