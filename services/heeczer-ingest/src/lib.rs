//! ai-heeczer ingestion service library surface (ADR-0005, plan 0004).
//!
//! Exposes a Router constructor so the binary entrypoint and integration
//! tests share the exact same wiring.
//!
//! Endpoints (MVP):
//! - `GET  /healthz`                 — liveness probe.
//! - `GET  /v1/version`              — engine + spec versions.
//! - `POST /v1/events`               — validate + score + persist a single event.
//! - `POST /v1/test/score-pipeline`  — RBAC- + feature-flagged back-to-back
//!   pipeline used by the dashboard test-orchestration view (ADR-0012). Never
//!   persists. Requires `HEECZER_FEATURE_TEST_ORCHESTRATION=1` at process
//!   start and the `x-heeczer-tester: 1` header on the request.

pub mod error;
pub mod handlers;
pub mod state;

pub use state::{AppState, Features};

use axum::routing::{get, post};
use axum::Router;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;

/// Maximum request body size (1 MiB) — mirrors the CLI's `MAX_INPUT_BYTES`.
const MAX_REQUEST_BODY_BYTES: usize = 1 * 1024 * 1024;

/// Build the application router. Kept as a free function so integration tests
/// can construct an in-memory database, build a Router, and exercise it via
/// `tower::ServiceExt::oneshot` with no network listener.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(handlers::healthz))
        .route("/v1/version", get(handlers::version))
        .route("/v1/events", post(handlers::ingest_event))
        .route(
            "/v1/test/score-pipeline",
            post(handlers::test_score_pipeline),
        )
        .layer(RequestBodyLimitLayer::new(MAX_REQUEST_BODY_BYTES))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
