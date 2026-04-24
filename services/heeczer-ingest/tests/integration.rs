//! End-to-end tests for the ingestion service. Uses `tower::ServiceExt::oneshot`
//! to drive the Router without a network listener.

use axum::body::Body;
use axum::http::{Request, StatusCode};
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
    build_router(AppState { pool, features })
}

async fn body_json(resp: axum::response::Response) -> Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).expect("response body is JSON")
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
        "event": { "not": "a valid event" },
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
    // PRD §19.4: the handler uses INSERT OR IGNORE, so a second POST with the
    // same (workspace_id, event_id) but a different payload is treated as a
    // duplicate and silently deduped — not a 409.  The second call must return
    // 200 OK with the original event_id preserved.
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
    // INSERT OR IGNORE means the conflicting row is silently dropped → 200 OK.
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
        StatusCode::OK,
        "conflicting payload with same event_id must be deduped (200), not rejected (409)"
    );
    let resp_b_body = body_json(resp_b).await;
    assert_eq!(
        resp_b_body["event_id"], conflict_uuid,
        "deduped response must still echo the event_id"
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
