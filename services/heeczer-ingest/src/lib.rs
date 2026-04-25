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

pub mod config;
pub mod error;
pub mod handlers;
pub mod state;

pub use config::Config;
pub use state::{AppState, Features};

use axum::http::{HeaderValue, Response};
use axum::routing::{get, post};
use axum::Router;
use axum_prometheus::metrics_exporter_prometheus::PrometheusHandle;
use axum_prometheus::PrometheusMetricLayer;
use std::sync::OnceLock;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;

/// Maximum request body size (1 MiB) — mirrors the CLI's `MAX_INPUT_BYTES`.
const MAX_REQUEST_BODY_BYTES: usize = 1024 * 1024;

/// Prometheus metric layer and handle, initialized exactly once per process.
/// `PrometheusMetricLayer::pair()` installs a global metrics recorder; calling
/// it more than once panics, so we guard the call with a `OnceLock`.
static PROMETHEUS: OnceLock<(PrometheusMetricLayer<'static>, PrometheusHandle)> = OnceLock::new();

fn prometheus_pair() -> (PrometheusMetricLayer<'static>, PrometheusHandle) {
    let (layer, handle) = PROMETHEUS.get_or_init(PrometheusMetricLayer::pair);
    (layer.clone(), handle.clone())
}

/// Middleware that stamps every response with `X-Heeczer-Spec-Version: 1.0`.
///
/// When v2 ships, this layer will additionally emit `Deprecation: true` and
/// `Sunset: <RFC 7231 date>` for responses that processed v1 events, giving
/// callers a standards-based signal that v1 support has an end-of-life date.
/// See ADR-0002 §v1 → v2 Evolution Policy.
async fn add_spec_version_header<B>(mut response: Response<B>) -> Response<B> {
    response
        .headers_mut()
        .insert("x-heeczer-spec-version", HeaderValue::from_static("1.0"));
    response
}

/// Build the application router. Kept as a free function so integration tests
/// can construct an in-memory database, build a Router, and exercise it via
/// `tower::ServiceExt::oneshot` with no network listener.
pub fn build_router(state: AppState) -> Router {
    let (prometheus_layer, metric_handle) = prometheus_pair();
    Router::new()
        .route("/healthz", get(handlers::healthz))
        .route("/v1/version", get(handlers::version))
        .route("/v1/events", post(handlers::ingest_event))
        .route("/v1/events:batch", post(handlers::ingest_batch))
        .route(
            "/v1/events/{event_id}",
            axum::routing::get(handlers::get_event),
        )
        .route(
            "/v1/events/{event_id}/scores",
            axum::routing::get(handlers::get_event_scores),
        )
        .route(
            "/v1/test/score-pipeline",
            post(handlers::test_score_pipeline),
        )
        .route(
            "/metrics",
            get(move || async move { metric_handle.render() }),
        )
        .layer(prometheus_layer)
        .layer(RequestBodyLimitLayer::new(MAX_REQUEST_BODY_BYTES))
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::map_response(add_spec_version_header))
        .with_state(state)
}
