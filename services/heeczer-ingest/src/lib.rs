//! ai-heeczer ingestion service library surface (ADR-0005, plan 0004).
//!
//! Exposes a Router constructor so the binary entrypoint and integration
//! tests share the exact same wiring.
//!
//! Endpoints:
//! - `GET  /healthz`                          — liveness probe.
//! - `GET  /v1/ready`                         — readiness probe (DB reachable).
//! - `GET  /v1/version`                       — engine + spec versions.
//! - `POST /v1/events`                        — validate + score + persist a single event.
//! - `POST /v1/events:batch`                  — batch ingest (up to 100 events).
//! - `GET  /v1/events/{event_id}`             — fetch stored event.
//! - `GET  /v1/events/{event_id}/scores`      — list score versions for an event.
//! - `POST /v1/events/{event_id}/rescore`     — explicit re-score; inserts new score row.
//! - `GET  /v1/jobs/{job_id}`                 — queue job status.
//! - `POST /v1/test/score-pipeline`           — RBAC- + feature-flagged back-to-back
//!   pipeline used by the dashboard test-orchestration view (ADR-0012). Never
//!   persists. Requires `HEECZER_FEATURE_TEST_ORCHESTRATION=1` at process
//!   start and the `x-heeczer-tester: 1` header on the request.

pub mod auth;
pub mod config;
pub mod error;
pub mod handlers;
pub mod openapi;
pub mod queue;
pub mod state;
pub mod worker;

pub use config::Config;
pub use state::{AppState, Features};

use axum::http::{HeaderValue, Request, Response, StatusCode};
use axum::middleware;
use axum::routing::{get, post};
use axum::Router;
use axum_prometheus::metrics_exporter_prometheus::PrometheusHandle;
use axum_prometheus::PrometheusMetricLayer;
use std::sync::OnceLock;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::KeyExtractor;
use tower_governor::{GovernorError, GovernorLayer};
use tower_http::trace::TraceLayer;

/// Prometheus metric layer and handle, initialized exactly once per process.
/// `PrometheusMetricLayer::pair()` installs a global metrics recorder; calling
/// it more than once panics, so we guard the call with a `OnceLock`.
static PROMETHEUS: OnceLock<(PrometheusMetricLayer<'static>, PrometheusHandle)> = OnceLock::new();

fn prometheus_pair() -> (PrometheusMetricLayer<'static>, PrometheusHandle) {
    let (layer, handle) = PROMETHEUS.get_or_init(PrometheusMetricLayer::pair);
    (layer.clone(), handle.clone())
}

#[derive(Clone, Debug)]
struct ApiKeyRateLimitKey;

impl KeyExtractor for ApiKeyRateLimitKey {
    type Key = String;

    fn extract<B>(&self, req: &Request<B>) -> Result<Self::Key, GovernorError> {
        req.headers()
            .get("x-heeczer-api-key")
            .and_then(|value| value.to_str().ok())
            .filter(|value| !value.trim().is_empty())
            .map(auth::hash_api_key)
            .ok_or(GovernorError::Other {
                code: StatusCode::UNAUTHORIZED,
                msg: Some("x-heeczer-api-key header required".to_string()),
                headers: None,
            })
    }
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
    let protected = Router::new()
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
            "/v1/events/{event_id}/rescore",
            post(handlers::rescore_event),
        )
        .route("/v1/jobs/{job_id}", axum::routing::get(handlers::get_job))
        .route(
            "/v1/test/score-pipeline",
            post(handlers::test_score_pipeline),
        );

    let protected = if state.auth.enabled {
        let governor = GovernorConfigBuilder::default()
            .per_second(state.rate_limit.refill_per_second.into())
            .burst_size(state.rate_limit.burst_size)
            .key_extractor(ApiKeyRateLimitKey)
            .use_headers()
            .finish()
            .expect("valid rate-limit config");
        protected.layer(GovernorLayer::new(governor))
    } else {
        protected
    }
    .layer(middleware::from_fn_with_state(
        state.clone(),
        auth::authenticate,
    ));

    Router::new()
        .route("/healthz", get(handlers::healthz))
        .route("/v1/ready", get(handlers::ready))
        .route("/v1/version", get(handlers::version))
        .route("/openapi.yaml", get(openapi::openapi_yaml))
        .merge(protected)
        .route(
            "/metrics",
            get(move || async move { metric_handle.render() }),
        )
        .layer(prometheus_layer)
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::map_response(add_spec_version_header))
        .with_state(state)
}
