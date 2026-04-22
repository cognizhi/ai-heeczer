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
