//! HTTP handlers. See `lib.rs` for the route table.

use std::sync::OnceLock;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use heeczer_core::schema::{EventValidator, Mode};
use heeczer_core::{
    score, Event, ScoreResult, ScoringProfile, TierSet, SCORING_VERSION, SPEC_VERSION,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

fn validator() -> &'static EventValidator {
    static V: OnceLock<EventValidator> = OnceLock::new();
    V.get_or_init(EventValidator::new_v1)
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub ok: bool,
    pub envelope_version: &'static str,
}

pub async fn healthz() -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        envelope_version: "1",
    })
}

#[derive(Serialize)]
pub struct VersionResponse {
    pub ok: bool,
    pub envelope_version: &'static str,
    pub scoring_version: &'static str,
    pub spec_version: &'static str,
    pub service: &'static str,
}

pub async fn version() -> Json<VersionResponse> {
    Json(VersionResponse {
        ok: true,
        envelope_version: "1",
        scoring_version: SCORING_VERSION,
        spec_version: SPEC_VERSION,
        service: env!("CARGO_PKG_VERSION"),
    })
}

/// Body for `POST /v1/events`. The `event` field is the canonical event JSON.
/// `workspace_id` is required; in a real deployment it would come from the
/// authenticated API key, but for the bootstrap surface we accept it in-band.
#[derive(Deserialize)]
pub struct IngestEventBody {
    pub workspace_id: String,
    pub event: Value,
}

#[derive(Serialize)]
pub struct IngestEventResponse {
    pub ok: bool,
    pub envelope_version: &'static str,
    pub event_id: String,
    pub score: ScoreResult,
}

pub async fn ingest_event(
    State(state): State<AppState>,
    Json(body): Json<IngestEventBody>,
) -> ApiResult<Json<IngestEventResponse>> {
    if body.workspace_id.is_empty() {
        return Err(ApiError::BadRequest("workspace_id is required".into()));
    }
    validator()
        .validate(&body.event, Mode::Strict)
        .map_err(|e| ApiError::Schema(e.to_string()))?;
    let event: Event = serde_json::from_value(body.event.clone())
        .map_err(|e| ApiError::BadRequest(format!("materialising Event: {e}")))?;
    let event_id = event.event_id.to_string();

    let profile = ScoringProfile::default_v1();
    let tiers = TierSet::default_v1();
    let result =
        score(&event, &profile, &tiers, None).map_err(|e| ApiError::Scoring(e.to_string()))?;

    let payload =
        serde_json::to_string(&body.event).map_err(|e| ApiError::Storage(e.to_string()))?;
    let result_json =
        serde_json::to_string(&result).map_err(|e| ApiError::Storage(e.to_string()))?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| ApiError::Storage(e.to_string()))?;

    sqlx::query(
        "INSERT OR IGNORE INTO aih_workspaces (workspace_id, display_name) VALUES (?1, ?1)",
    )
    .bind(&body.workspace_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Storage(e.to_string()))?;

    // PRD §19.4: duplicate event_id → return existing record. Use INSERT OR IGNORE
    // and rely on the (workspace_id, event_id) PK to dedupe.
    sqlx::query(
        "INSERT OR IGNORE INTO aih_events
         (event_id, workspace_id, spec_version, framework_source, payload, received_at)
         VALUES (?1, ?2, ?3, ?4, ?5, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
    )
    .bind(&event_id)
    .bind(&body.workspace_id)
    .bind(&result.spec_version)
    .bind(event.framework_source.as_str())
    .bind(&payload)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Storage(e.to_string()))?;

    sqlx::query(
        "INSERT OR IGNORE INTO aih_scores
         (workspace_id, event_id, scoring_version, scoring_profile_id, profile_version,
          tier_id, tier_version, rates_version, result_json,
          final_minutes, final_fec, confidence, confidence_band, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                 strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
    )
    .bind(&body.workspace_id)
    .bind(&event_id)
    .bind(&result.scoring_version)
    .bind(&result.scoring_profile)
    .bind(&profile.version)
    .bind(&result.tier.id)
    .bind(&tiers.version)
    .bind(&tiers.version) // rates_version: tier-set carries rates in the bootstrap profile
    .bind(&result_json)
    .bind(result.final_estimated_minutes.to_string())
    .bind(result.financial_equivalent_cost.to_string())
    .bind(result.confidence_score.to_string())
    .bind(format!("{:?}", result.confidence_band))
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Storage(e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| ApiError::Storage(e.to_string()))?;

    Ok(Json(IngestEventResponse {
        ok: true,
        envelope_version: "1",
        event_id,
        score: result,
    }))
}

/// Body for `POST /v1/test/score-pipeline` (ADR-0012). Optional profile / tier
/// overrides allow the dashboard to exercise candidate configs without
/// touching live data.
#[derive(Deserialize)]
pub struct TestPipelineBody {
    pub event: Value,
    pub profile: Option<Value>,
    pub tier_set: Option<Value>,
    pub tier_override: Option<String>,
}

#[derive(Serialize)]
pub struct TestPipelineResponse {
    pub ok: bool,
    pub envelope_version: &'static str,
    pub score: ScoreResult,
}

/// Back-to-back scoring used by the dashboard test-orchestration view. Never
/// touches the database. Requires:
///   - `Features::test_orchestration` enabled on the running process; and
///   - the `x-heeczer-tester: 1` header on the request (RBAC stub; the real
///     dashboard will mint a short-lived token mapped to a `Tester` role).
pub async fn test_score_pipeline(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<TestPipelineResponse>> {
    if !state.features.test_orchestration {
        return Err(ApiError::FeatureDisabled(
            "test_orchestration is not enabled on this deployment".into(),
        ));
    }
    if headers
        .get("x-heeczer-tester")
        .and_then(|v| v.to_str().ok())
        != Some("1")
    {
        return Err(ApiError::Forbidden(
            "x-heeczer-tester header required for test-orchestration endpoints".into(),
        ));
    }

    let body: TestPipelineBody = serde_json::from_slice(&body)
        .map_err(|e| ApiError::BadRequest(format!("parsing request body: {e}")))?;
    validator()
        .validate(&body.event, Mode::Strict)
        .map_err(|e| ApiError::Schema(e.to_string()))?;
    let event: Event = serde_json::from_value(body.event)
        .map_err(|e| ApiError::BadRequest(format!("materialising Event: {e}")))?;
    let profile = match body.profile {
        Some(v) => serde_json::from_value(v)
            .map_err(|e| ApiError::BadRequest(format!("materialising ScoringProfile: {e}")))?,
        None => ScoringProfile::default_v1(),
    };
    let tiers = match body.tier_set {
        Some(v) => serde_json::from_value(v)
            .map_err(|e| ApiError::BadRequest(format!("materialising TierSet: {e}")))?,
        None => TierSet::default_v1(),
    };
    let result = score(&event, &profile, &tiers, body.tier_override.as_deref())
        .map_err(|e| ApiError::Scoring(e.to_string()))?;
    Ok(Json(TestPipelineResponse {
        ok: true,
        envelope_version: "1",
        score: result,
    }))
}
