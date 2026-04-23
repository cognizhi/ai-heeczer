//! Shared application state for the ingestion service.

use sqlx_sqlite::SqlitePool;

/// Feature flags for runtime-gated surfaces. Defaults are production-safe
/// (everything optional is off).
#[derive(Debug, Clone, Copy, Default)]
pub struct Features {
    /// Enable the `/v1/test/*` endpoints used by the dashboard
    /// test-orchestration view (ADR-0012). Off by default.
    pub test_orchestration: bool,
}

/// Application state shared across handlers via axum's `State` extractor.
#[derive(Debug, Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub features: Features,
}
