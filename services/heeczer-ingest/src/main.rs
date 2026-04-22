//! ai-heeczer ingestion service binary entrypoint.
//!
//! See ADR-0005 (language choice) and plan 0004 for the larger contract.

use std::net::SocketAddr;

use anyhow::Context;
use heeczer_ingest::{build_router, AppState, Features};

/// Redact the password component of a database DSN so it is safe to log.
/// Returns the DSN unchanged if it cannot be parsed as a URL.
fn redact_dsn(dsn: &str) -> String {
    // Fast path: no "//" means it's not a URL (e.g. "sqlite::memory:").
    if !dsn.contains("://") {
        return dsn.to_owned();
    }
    // Replace ://user:password@ with ://user:***@
    if let Some(at_pos) = dsn.rfind('@') {
        let scheme_end = dsn.find("://").map(|i| i + 3).unwrap_or(0);
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
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let database_url =
        std::env::var("HEECZER_DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_string());
    let pool = heeczer_storage::sqlite::open(&database_url)
        .await
        .with_context(|| {
            // Redact password component so credentials don't appear in logs.
            let redacted = redact_dsn(&database_url);
            format!("opening database at {redacted}")
        })?;
    heeczer_storage::sqlite::migrate(&pool).await?;

    let features = Features {
        // Test-orchestration endpoints (`/v1/test/*`) are gated by a feature
        // flag per ADR-0012. Off by default in production deployments.
        test_orchestration: std::env::var("HEECZER_FEATURE_TEST_ORCHESTRATION")
            .ok()
            .is_some_and(|v| matches!(v.as_str(), "1" | "true" | "on")),
    };
    let state = AppState { pool, features };

    let addr: SocketAddr = std::env::var("HEECZER_INGEST_BIND")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse()
        .context("parsing HEECZER_INGEST_BIND")?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("heeczer-ingest listening on {addr}");
    axum::serve(listener, build_router(state)).await?;
    Ok(())
}
