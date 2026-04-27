//! End-to-end tests for the ingestion service. Uses `tower::ServiceExt::oneshot`
//! to drive the Router without a network listener.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use heeczer_ingest::auth::hash_api_key;
use heeczer_ingest::state::{AuthConfig, RateLimitConfig};
use heeczer_ingest::{build_router, AppState, Features};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

const CANONICAL: &str = "../../core/schema/fixtures/events/valid/01-prd-canonical.json";

async fn router_with_features(features: Features) -> axum::Router {
    let pool = heeczer_storage::sqlite::open("sqlite::memory:")
        .await
        .unwrap();
    heeczer_storage::sqlite::migrate(&pool).await.unwrap();
    build_router(AppState::new(pool, features))
}

async fn body_json(resp: axum::response::Response) -> Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).expect("response body is JSON")
}

async fn body_bytes(resp: axum::response::Response) -> axum::body::Bytes {
    resp.into_body().collect().await.unwrap().to_bytes()
}

async fn auth_enabled_router() -> (axum::Router, sqlx_sqlite::SqlitePool) {
    let pool = heeczer_storage::sqlite::open("sqlite::memory:")
        .await
        .unwrap();
    heeczer_storage::sqlite::migrate(&pool).await.unwrap();
    sqlx_core::query::query(
        "INSERT INTO heec_workspaces (workspace_id, display_name) VALUES ('ws_auth', 'Auth')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx_core::query::query(
        "INSERT INTO heec_api_keys (api_key_id, workspace_id, hashed_key, label) \
         VALUES ('key-auth', 'ws_auth', ?1, 'test')",
    )
    .bind(hash_api_key("test-api-key"))
    .execute(&pool)
    .await
    .unwrap();
    let mut state = AppState::new(pool.clone(), Features::default());
    state.auth = AuthConfig { enabled: true };
    (build_router(state), pool)
}

#[tokio::test]
async fn healthz_returns_ok_envelope() {
    let app = router_with_features(Features::default()).await;
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body["ok"], true);
    assert_eq!(body["envelope_version"], "1");
}

#[tokio::test]
async fn version_reports_pinned_constants() {
    let app = router_with_features(Features::default()).await;
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/v1/version")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body["scoring_version"], "1.0.0");
    assert_eq!(body["spec_version"], "1.0");
}

#[tokio::test]
async fn ingest_event_validates_scores_and_persists() {
    let app = router_with_features(Features::default()).await;
    let event: Value = serde_json::from_str(&std::fs::read_to_string(CANONICAL).unwrap()).unwrap();
    let body = json!({
        "workspace_id": "ws_test",
        "event": event,
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "ingest event must succeed");
    let body = body_json(resp).await;
    assert_eq!(body["ok"], true);
    assert!(!body["event_id"].as_str().unwrap().is_empty());
    assert!(body["score"]["final_estimated_minutes"].is_string());
}

#[tokio::test]
async fn ingest_event_rejects_invalid_payload() {
    let app = router_with_features(Features::default()).await;
    let body = json!({
        "workspace_id": "ws_test",
        // spec_version present so routing passes; remaining fields are invalid
        // and must be caught by the schema validator.
        "event": { "spec_version": "1.0", "not_a_real_field": "value" },
    });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = body_json(resp).await;
    assert_eq!(body["ok"], false);
    assert_eq!(body["error"]["kind"], "schema");
}

#[tokio::test]
async fn ingest_event_rejects_prompt_or_output_content_fields() {
    let app = router_with_features(Features::default()).await;
    let mut event: Value =
        serde_json::from_str(&std::fs::read_to_string(CANONICAL).unwrap()).unwrap();
    event["meta"]["extensions"] = json!({
        "prompt_text": "never store prompt bodies",
        "nested": { "output_text": "never store model output bodies" }
    });
    let body = json!({
        "workspace_id": "ws_test",
        "event": event,
    });

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = body_json(resp).await;
    assert_eq!(body["error"]["kind"], "schema");
}

#[tokio::test]
async fn ingest_event_requires_workspace_id() {
    let app = router_with_features(Features::default()).await;
    let event: Value = serde_json::from_str(&std::fs::read_to_string(CANONICAL).unwrap()).unwrap();
    let body = json!({ "workspace_id": "", "event": event });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = body_json(resp).await;
    assert_eq!(body["error"]["kind"], "bad_request");
}

#[tokio::test]
async fn test_score_pipeline_blocked_when_feature_off() {
    let app = router_with_features(Features::default()).await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/test/score-pipeline")
                .header("content-type", "application/json")
                .header("x-heeczer-tester", "1")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = body_json(resp).await;
    assert_eq!(body["error"]["kind"], "feature_disabled");
}

#[tokio::test]
async fn test_score_pipeline_blocked_without_tester_header() {
    let app = router_with_features(Features {
        test_orchestration: true,
    })
    .await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/test/score-pipeline")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    let body = body_json(resp).await;
    assert_eq!(body["error"]["kind"], "forbidden");
}

#[tokio::test]
async fn test_score_pipeline_runs_back_to_back() {
    let app = router_with_features(Features {
        test_orchestration: true,
    })
    .await;
    let event: Value = serde_json::from_str(&std::fs::read_to_string(CANONICAL).unwrap()).unwrap();
    let body = json!({ "event": event });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/test/score-pipeline")
                .header("content-type", "application/json")
                .header("x-heeczer-tester", "1")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body["ok"], true);
    assert!(body["score"]["final_estimated_minutes"].is_string());
}

#[tokio::test]
async fn ingest_event_is_idempotent_on_duplicate_event_id() {
    // PRD §19.4: posting the same event_id twice must return HTTP 200 both times.
    let app = router_with_features(Features::default()).await;
    let event: Value = serde_json::from_str(&std::fs::read_to_string(CANONICAL).unwrap()).unwrap();
    let req_body = json!({ "workspace_id": "ws_idem", "event": event });

    for attempt in 0..2 {
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/events")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "attempt {attempt}: duplicate event_id must return 200"
        );
    }
}

#[tokio::test]
async fn ingest_event_returns_existing_on_duplicate_event_id() {
    // PRD §19.4: the second POST of an identical event must return 200 OK and
    // echo back the same event_id that was stored on the first POST.
    let app = router_with_features(Features::default()).await;
    let event: Value = serde_json::from_str(&std::fs::read_to_string(CANONICAL).unwrap()).unwrap();
    let expected_event_id = event["event_id"].as_str().unwrap().to_owned();
    let req_body = json!({ "workspace_id": "ws_dedup_ret", "event": event });

    // First POST — stores the record.
    let resp1 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp1.status(), StatusCode::OK, "first POST must succeed");
    let body1 = body_json(resp1).await;
    assert_eq!(
        body1["event_id"], expected_event_id,
        "first response must echo event_id"
    );

    // Second POST — duplicate is silently ignored (INSERT OR IGNORE); 200 OK
    // and the same event_id must still be returned.
    let resp2 = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        resp2.status(),
        StatusCode::OK,
        "second POST must also return 200 (dedup)"
    );
    let body2 = body_json(resp2).await;
    assert_eq!(
        body2["event_id"], expected_event_id,
        "second response must echo the same event_id"
    );
}

#[tokio::test]
async fn ingest_event_rejects_conflicting_payload_for_same_event_id() {
    // PRD §19.4: duplicate event_id with a different normalized payload is a
    // conflict and must use a new event_id or replay API.
    let app = router_with_features(Features::default()).await;

    // Build two structurally distinct events sharing the same event_id.
    let mut event_a: Value =
        serde_json::from_str(&std::fs::read_to_string(CANONICAL).unwrap()).unwrap();
    let conflict_uuid = "00000000-0000-4000-8000-000000000099";
    event_a["event_id"] = json!(conflict_uuid);

    let mut event_b = event_a.clone();
    event_b["timestamp"] = json!("2026-01-01T00:00:00Z");
    event_b["task"]["category"] = json!("code_review");

    let body_a = json!({ "workspace_id": "ws_conflict", "event": event_a });
    let body_b = json!({ "workspace_id": "ws_conflict", "event": event_b });

    // First POST with payload A.
    let resp_a = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body_a).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp_a.status(), StatusCode::OK, "first POST must succeed");
    let resp_a_body = body_json(resp_a).await;
    assert_eq!(resp_a_body["event_id"], conflict_uuid);

    // Second POST with a different payload but the same event_id.
    let resp_b = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body_b).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        resp_b.status(),
        StatusCode::CONFLICT,
        "conflicting payload with same event_id must be rejected (409)"
    );
    let resp_b_body = body_json(resp_b).await;
    assert_eq!(
        resp_b_body["error"]["kind"], "conflict",
        "conflict response must use the structured error envelope"
    );
}

#[tokio::test]
async fn ingest_event_rejects_oversized_workspace_id() {
    let app = router_with_features(Features::default()).await;
    let event: Value = serde_json::from_str(&std::fs::read_to_string(CANONICAL).unwrap()).unwrap();
    // 129-character workspace_id should be rejected.
    let long_id = "a".repeat(129);
    let req_body = json!({ "workspace_id": long_id, "event": event });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = body_json(resp).await;
    assert_eq!(body["error"]["kind"], "bad_request");
}

#[tokio::test]
async fn ingest_event_rejects_illegal_characters_in_workspace_id() {
    let app = router_with_features(Features::default()).await;
    let event: Value = serde_json::from_str(&std::fs::read_to_string(CANONICAL).unwrap()).unwrap();
    // Newline and semicolon are not in the allowlist.
    let req_body = json!({ "workspace_id": "ws\ninjection;", "event": event });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = body_json(resp).await;
    assert_eq!(body["error"]["kind"], "bad_request");
}

#[tokio::test]
async fn metrics_endpoint_returns_200() {
    let app = router_with_features(Features::default()).await;
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    assert!(!bytes.is_empty(), "metrics body must not be empty");
}

#[tokio::test]
async fn conflict_ingest_writes_audit_log_entry() {
    let pool = heeczer_storage::sqlite::open("sqlite::memory:")
        .await
        .unwrap();
    heeczer_storage::sqlite::migrate(&pool).await.unwrap();
    let app = build_router(AppState::new(pool.clone(), Features::default()));

    let mut event_a: Value =
        serde_json::from_str(&std::fs::read_to_string(CANONICAL).unwrap()).unwrap();
    let conflict_uuid = "00000000-0000-4000-8000-000000000199";
    event_a["event_id"] = json!(conflict_uuid);
    let mut event_b = event_a.clone();
    event_b["task"]["category"] = json!("code_review");

    for event in [event_a, event_b] {
        let body = json!({ "workspace_id": "ws_conflict_audit", "event": event });
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/events")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
    }

    let (count,): (i64,) = sqlx_core::query_as::query_as(
        "SELECT COUNT(*) FROM heec_audit_log \
         WHERE workspace_id = 'ws_conflict_audit' AND action = 'ingest_conflict'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 1, "conflicting duplicate ingest must be audited");
}

#[tokio::test]
async fn auth_middleware_requires_api_key_and_audits_failure() {
    let (app, pool) = auth_enabled_router().await;
    let event: Value = serde_json::from_str(&std::fs::read_to_string(CANONICAL).unwrap()).unwrap();
    let body = json!({ "workspace_id": "ws_auth", "event": event });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let body = body_json(resp).await;
    assert_eq!(body["error"]["kind"], "unauthorized");

    let (count,): (i64,) = sqlx_core::query_as::query_as(
        "SELECT COUNT(*) FROM heec_audit_log WHERE action = 'auth_failed'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 1, "auth failure must be audited");
}

#[tokio::test]
async fn auth_middleware_scopes_workspace_to_api_key() {
    let (app, _) = auth_enabled_router().await;
    let event: Value = serde_json::from_str(&std::fs::read_to_string(CANONICAL).unwrap()).unwrap();

    let forbidden_body = json!({ "workspace_id": "other_ws", "event": event.clone() });
    let forbidden = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("content-type", "application/json")
                .header("x-heeczer-api-key", "test-api-key")
                .body(Body::from(serde_json::to_vec(&forbidden_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);

    let ok_body = json!({ "workspace_id": "ws_auth", "event": event });
    let ok = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("content-type", "application/json")
                .header("x-heeczer-api-key", "test-api-key")
                .body(Body::from(serde_json::to_vec(&ok_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
}

#[tokio::test]
async fn per_workspace_daily_quota_returns_429_with_headers() {
    let pool = heeczer_storage::sqlite::open("sqlite::memory:")
        .await
        .unwrap();
    heeczer_storage::sqlite::migrate(&pool).await.unwrap();
    sqlx_core::query::query(
        "INSERT INTO heec_workspaces (workspace_id, display_name, settings_json) \
         VALUES ('ws_quota', 'Quota', '{\"daily_event_quota\":0}')",
    )
    .execute(&pool)
    .await
    .unwrap();
    let app = build_router(AppState::new(pool, Features::default()));
    let event: Value = serde_json::from_str(&std::fs::read_to_string(CANONICAL).unwrap()).unwrap();
    let body = json!({ "workspace_id": "ws_quota", "event": event });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(
        resp.headers()
            .get("Retry-After")
            .and_then(|v| v.to_str().ok()),
        Some("86400")
    );
    assert_eq!(
        resp.headers()
            .get("X-Heeczer-Quota-Limit")
            .and_then(|v| v.to_str().ok()),
        Some("0")
    );
}

#[tokio::test]
async fn batch_idempotency_key_replays_byte_equal_response() {
    let pool = heeczer_storage::sqlite::open("sqlite::memory:")
        .await
        .unwrap();
    heeczer_storage::sqlite::migrate(&pool).await.unwrap();
    let app = build_router(AppState::new(pool.clone(), Features::default()));
    let mut event: Value =
        serde_json::from_str(&std::fs::read_to_string(CANONICAL).unwrap()).unwrap();
    event["event_id"] = json!("00000000-0000-4000-8000-000000000299");
    let body = json!({ "workspace_id": "ws_idem_batch", "events": [event] });
    let request_bytes = serde_json::to_vec(&body).unwrap();

    let first = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events:batch")
                .header("content-type", "application/json")
                .header("Idempotency-Key", "idem-1")
                .body(Body::from(request_bytes.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);
    let first_bytes = body_bytes(first).await;

    let second = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events:batch")
                .header("content-type", "application/json")
                .header("Idempotency-Key", "idem-1")
                .body(Body::from(request_bytes))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second.status(), StatusCode::OK);
    let second_bytes = body_bytes(second).await;
    assert_eq!(
        first_bytes, second_bytes,
        "idempotency replay must be byte-equal"
    );

    let (stored,): (i64,) = sqlx_core::query_as::query_as(
        "SELECT COUNT(*) FROM heec_idempotency_keys WHERE workspace_id = 'ws_idem_batch'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(stored, 1);
}

#[tokio::test]
async fn api_key_rate_limit_returns_429_headers() {
    let pool = heeczer_storage::sqlite::open("sqlite::memory:")
        .await
        .unwrap();
    heeczer_storage::sqlite::migrate(&pool).await.unwrap();
    sqlx_core::query::query(
        "INSERT INTO heec_workspaces (workspace_id, display_name) VALUES ('ws_auth', 'Auth')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx_core::query::query(
        "INSERT INTO heec_api_keys (api_key_id, workspace_id, hashed_key, label) \
         VALUES ('key-rate', 'ws_auth', ?1, 'test')",
    )
    .bind(hash_api_key("rate-api-key"))
    .execute(&pool)
    .await
    .unwrap();
    let mut state = AppState::new(pool, Features::default());
    state.auth = AuthConfig { enabled: true };
    state.rate_limit = RateLimitConfig {
        refill_per_second: 1,
        burst_size: 1,
    };
    let app = build_router(state);
    let event: Value = serde_json::from_str(&std::fs::read_to_string(CANONICAL).unwrap()).unwrap();
    let body = json!({ "workspace_id": "ws_auth", "event": event });
    let payload = serde_json::to_vec(&body).unwrap();

    for expected in [StatusCode::OK, StatusCode::TOO_MANY_REQUESTS] {
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/events")
                    .header("content-type", "application/json")
                    .header("x-heeczer-api-key", "rate-api-key")
                    .body(Body::from(payload.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), expected);
        if expected == StatusCode::TOO_MANY_REQUESTS {
            assert!(resp.headers().get("Retry-After").is_some());
        }
    }
}

#[tokio::test]
async fn openapi_yaml_is_served_and_contains_routes() {
    let app = router_with_features(Features::default()).await;
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/openapi.yaml")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = body_bytes(resp).await;
    let spec = String::from_utf8(bytes.to_vec()).expect("OpenAPI YAML should be UTF-8");
    for path in [
        "/v1/events",
        "/v1/events:batch",
        "/v1/events/{event_id}/rescore",
        "/v1/jobs/{job_id}",
    ] {
        assert!(spec.contains(path), "missing OpenAPI path {path}");
    }
}

// ─── GET /v1/ready ──────────────────────────────────────────────────────────

#[tokio::test]
async fn ready_returns_ok_when_db_reachable() {
    let app = router_with_features(Features::default()).await;
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/v1/ready")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body["ok"], true);
    assert_eq!(body["envelope_version"], "1");
}

// ─── spec_version negotiation (ADR-0002) ────────────────────────────────────

#[tokio::test]
async fn ingest_event_rejects_unsupported_spec_version_with_415() {
    let app = router_with_features(Features::default()).await;
    let event: Value = serde_json::from_str(&std::fs::read_to_string(CANONICAL).unwrap()).unwrap();
    // Override spec_version with a hypothetical future version.
    let mut ev = event.clone();
    ev["spec_version"] = json!("2.0");
    let body = json!({ "workspace_id": "ws_spec", "event": ev });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::UNSUPPORTED_MEDIA_TYPE,
        "unsupported spec_version must yield 415, not 400"
    );
    assert_eq!(
        resp.headers()
            .get("Supported-Spec-Versions")
            .and_then(|v| v.to_str().ok()),
        Some("1.0"),
        "415 response must advertise supported versions"
    );
    let body = body_json(resp).await;
    assert_eq!(body["error"]["kind"], "unsupported_spec_version");
}

#[tokio::test]
async fn ingest_batch_rejects_unsupported_spec_version_with_415() {
    let app = router_with_features(Features::default()).await;
    let mut event: Value =
        serde_json::from_str(&std::fs::read_to_string(CANONICAL).unwrap()).unwrap();
    event["spec_version"] = json!("2.0");
    let body = json!({ "workspace_id": "ws_batch_spec", "events": [event] });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events:batch")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(
        resp.headers()
            .get("Supported-Spec-Versions")
            .and_then(|v| v.to_str().ok()),
        Some("1.0")
    );
    let body = body_json(resp).await;
    assert_eq!(body["error"]["kind"], "unsupported_spec_version");
}

// ─── POST /v1/events/{event_id}:rescore ─────────────────────────────────────

#[tokio::test]
async fn rescore_returns_score_for_existing_event() {
    let app = router_with_features(Features::default()).await;
    let event: Value = serde_json::from_str(&std::fs::read_to_string(CANONICAL).unwrap()).unwrap();
    let event_id = event["event_id"].as_str().unwrap().to_owned();
    let ingest_body = json!({ "workspace_id": "ws_rescore", "event": event });

    // First, ingest the event.
    let ingest_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&ingest_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ingest_resp.status(), StatusCode::OK, "ingest must succeed");

    // Now rescore it.
    let rescore_body = json!({ "workspace_id": "ws_rescore" });
    let rescore_resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v1/events/{event_id}/rescore"))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&rescore_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        rescore_resp.status(),
        StatusCode::OK,
        "rescore must succeed"
    );
    let body = body_json(rescore_resp).await;
    assert_eq!(body["ok"], true);
    assert_eq!(body["event_id"], event_id);
    assert!(
        body["score"]["final_estimated_minutes"].is_string(),
        "rescore must return a score"
    );
}

#[tokio::test]
async fn rescore_with_tier_override_persists_new_score_and_audit() {
    let pool = heeczer_storage::sqlite::open("sqlite::memory:")
        .await
        .unwrap();
    heeczer_storage::sqlite::migrate(&pool).await.unwrap();
    let app = build_router(AppState::new(pool.clone(), Features::default()));

    let event: Value = serde_json::from_str(&std::fs::read_to_string(CANONICAL).unwrap()).unwrap();
    let event_id = event["event_id"].as_str().unwrap().to_owned();
    let ingest_body = json!({ "workspace_id": "ws_rescore_override", "event": event });
    let ingest_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&ingest_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ingest_resp.status(), StatusCode::OK);

    let rescore_body =
        json!({ "workspace_id": "ws_rescore_override", "tier_override": "tier_senior_eng" });
    let rescore_resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v1/events/{event_id}/rescore"))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&rescore_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rescore_resp.status(), StatusCode::OK);

    let (score_count,): (i64,) = sqlx_core::query_as::query_as(
        "SELECT COUNT(*) FROM heec_scores \
         WHERE workspace_id = 'ws_rescore_override' AND event_id = ?1",
    )
    .bind(&event_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        score_count, 2,
        "tier override must persist a distinct score row"
    );

    let (audit_count,): (i64,) = sqlx_core::query_as::query_as(
        "SELECT COUNT(*) FROM heec_audit_log \
         WHERE workspace_id = 'ws_rescore_override' AND action = 'rescore' AND target_id = ?1",
    )
    .bind(&event_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(audit_count, 1, "new re-score row must be audited");
}

#[tokio::test]
async fn rescore_returns_404_for_missing_event() {
    let app = router_with_features(Features::default()).await;
    let missing_id = "00000000-0000-4000-8000-000000000000";
    let body = json!({ "workspace_id": "ws_rescore_404" });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v1/events/{missing_id}/rescore"))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let body = body_json(resp).await;
    assert_eq!(body["error"]["kind"], "not_found");
}

// ─── GET /v1/jobs/{job_id} ──────────────────────────────────────────────────

#[tokio::test]
async fn get_job_returns_404_for_missing_job() {
    let app = router_with_features(Features::default()).await;
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/v1/jobs/no-such-job?workspace_id=ws_test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let body = body_json(resp).await;
    assert_eq!(body["error"]["kind"], "not_found");
}

#[tokio::test]
async fn get_job_returns_job_state_when_present() {
    // Seed a job row directly, then verify the endpoint returns it.
    let pool = heeczer_storage::sqlite::open("sqlite::memory:")
        .await
        .unwrap();
    heeczer_storage::sqlite::migrate(&pool).await.unwrap();

    sqlx_core::query::query(
        "INSERT INTO heec_workspaces (workspace_id, display_name) VALUES ('ws_jobs', 'Jobs')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx_core::query::query(
        "INSERT INTO heec_jobs (job_id, workspace_id, state, attempts) \
         VALUES ('job-abc-123', 'ws_jobs', 'pending', 0)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let app = build_router(AppState::new(pool, Features::default()));
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/v1/jobs/job-abc-123?workspace_id=ws_jobs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body["ok"], true);
    assert_eq!(body["job_id"], "job-abc-123");
    assert_eq!(body["state"], "pending");
    assert_eq!(body["attempts"], 0);
}

// ─── correlation_id persistence ─────────────────────────────────────────────

#[tokio::test]
async fn ingest_event_persists_correlation_id_column() {
    // The canonical fixture has correlation_id = "task-batch-001".
    // After ingest the heec_events.correlation_id column must be populated so
    // that the rescore handler and tracing fields correctly reflect it.
    let pool = heeczer_storage::sqlite::open("sqlite::memory:")
        .await
        .unwrap();
    heeczer_storage::sqlite::migrate(&pool).await.unwrap();

    let event: Value = serde_json::from_str(&std::fs::read_to_string(CANONICAL).unwrap()).unwrap();
    let event_id = event["event_id"].as_str().unwrap().to_owned();
    let req_body = json!({ "workspace_id": "ws_corr", "event": event });

    let app = build_router(AppState::new(pool.clone(), Features::default()));
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "ingest must succeed");

    // Verify the DB column directly.
    let row: Option<(Option<String>,)> = sqlx_core::query_as::query_as(
        "SELECT correlation_id FROM heec_events WHERE workspace_id = 'ws_corr' AND event_id = ?1",
    )
    .bind(&event_id)
    .fetch_optional(&pool)
    .await
    .unwrap();
    let corr = row.expect("event must exist").0;
    assert_eq!(
        corr.as_deref(),
        Some("task-batch-001"),
        "correlation_id must be stored in the dedicated column"
    );
}

// ─── GET /v1/events/{event_id}/scores — 404 for nonexistent event ────────────

#[tokio::test]
async fn get_event_scores_returns_404_for_missing_event() {
    let app = router_with_features(Features::default()).await;
    let missing_id = "00000000-0000-4000-8000-000000000001";
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/v1/events/{missing_id}/scores?workspace_id=ws_scores_miss"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let body = body_json(resp).await;
    assert_eq!(body["error"]["kind"], "not_found");
}

// ─── rescore no-op (idempotent re-score with same params) ───────────────────

#[tokio::test]
async fn rescore_is_idempotent_no_phantom_audit_entries() {
    // Rescoring twice with the same params (default profile, same scoring_version)
    // is a no-op because the heec_scores PK already exists from ingest.
    // Neither call should write a phantom audit entry.
    let pool = heeczer_storage::sqlite::open("sqlite::memory:")
        .await
        .unwrap();
    heeczer_storage::sqlite::migrate(&pool).await.unwrap();

    let event: Value = serde_json::from_str(&std::fs::read_to_string(CANONICAL).unwrap()).unwrap();
    let event_id = event["event_id"].as_str().unwrap().to_owned();
    let ingest_body = json!({ "workspace_id": "ws_rescore_idem", "event": event });
    let rescore_body = json!({ "workspace_id": "ws_rescore_idem" });

    let app = build_router(AppState::new(pool.clone(), Features::default()));

    // Ingest first — creates the initial score row via default profile.
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&ingest_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let count_before: (i64,) = sqlx_core::query_as::query_as(
        "SELECT COUNT(*) FROM heec_audit_log \
         WHERE workspace_id = 'ws_rescore_idem' AND action = 'rescore' AND target_id = ?1",
    )
    .bind(&event_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    // Rescore once with identical params (no-op INSERT OR IGNORE) — no new audit entry.
    let r1 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v1/events/{event_id}/rescore"))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&rescore_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r1.status(), StatusCode::OK, "first rescore must succeed");

    // Rescore again — still a no-op.
    let r2 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v1/events/{event_id}/rescore"))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&rescore_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        r2.status(),
        StatusCode::OK,
        "second rescore must also succeed"
    );

    // Audit count must not exceed the count before rescoring — no phantom entries.
    let count_after: (i64,) = sqlx_core::query_as::query_as(
        "SELECT COUNT(*) FROM heec_audit_log \
         WHERE workspace_id = 'ws_rescore_idem' AND action = 'rescore' AND target_id = ?1",
    )
    .bind(&event_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        count_after.0, count_before.0,
        "no-op rescores must not emit phantom audit entries"
    );
}
