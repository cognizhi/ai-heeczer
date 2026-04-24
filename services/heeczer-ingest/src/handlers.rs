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
use sqlx_core::query::query;
use sqlx_core::query_as::query_as;

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
    // Constrain workspace_id to safe characters and reasonable length.
    // Accepted: alphanumeric, '-', '_'. Max 128 chars.
    if body.workspace_id.len() > 128
        || !body
            .workspace_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(ApiError::BadRequest(
            "workspace_id must be 1–128 ASCII alphanumeric, dash, or underscore chars".into(),
        ));
    }
    validator()
        .validate(&body.event, Mode::Strict)
        .map_err(|e| ApiError::Schema(e.to_string()))?;
    let event: Event = serde_json::from_value(body.event.clone())
        .map_err(|e| ApiError::BadRequest(format!("materialising Event: {e}")))?;
    let event_id = event.event_id.clone();

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

    query("INSERT OR IGNORE INTO heec_workspaces (workspace_id, display_name) VALUES (?1, ?1)")
        .bind(&body.workspace_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::Storage(e.to_string()))?;

    // PRD §19.4: duplicate event_id → return existing record. Use INSERT OR IGNORE
    // and rely on the (workspace_id, event_id) PK to dedupe.
    query(
        "INSERT OR IGNORE INTO heec_events
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

    query(
        "INSERT OR IGNORE INTO heec_scores
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

// ─── GET /v1/events/{event_id} ──────────────────────────────────────────────

/// Response for `GET /v1/events/{event_id}`.
#[derive(Serialize)]
pub struct GetEventResponse {
    pub ok: bool,
    pub envelope_version: &'static str,
    pub event_id: String,
    pub workspace_id: String,
    pub received_at: String,
    pub payload: serde_json::Value,
}

#[allow(clippy::implicit_hasher)]
pub async fn get_event(
    State(state): State<AppState>,
    axum::extract::Path(event_id): axum::extract::Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> ApiResult<Json<GetEventResponse>> {
    let workspace_id = params
        .get("workspace_id")
        .ok_or_else(|| ApiError::BadRequest("workspace_id query param required".into()))?
        .clone();

    let row = query_as::<_, (String, String, String, String)>(
        "SELECT event_id, workspace_id, received_at, payload \
         FROM heec_events WHERE workspace_id = ?1 AND event_id = ?2",
    )
    .bind(&workspace_id)
    .bind(&event_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::Storage(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound(format!("event {event_id} not found")))?;

    let payload: serde_json::Value =
        serde_json::from_str(&row.3).unwrap_or(serde_json::Value::Null);
    Ok(Json(GetEventResponse {
        ok: true,
        envelope_version: "1",
        event_id: row.0,
        workspace_id: row.1,
        received_at: row.2,
        payload,
    }))
}

// ─── GET /v1/events/{event_id}/scores ───────────────────────────────────────

/// Response for `GET /v1/events/{event_id}/scores`.
#[derive(Serialize)]
pub struct ScoreSummary {
    pub scoring_version: String,
    pub scoring_profile_id: String,
    pub final_minutes: String,
    pub final_fec: String,
    pub confidence: String,
    pub confidence_band: String,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct GetScoresResponse {
    pub ok: bool,
    pub envelope_version: &'static str,
    pub event_id: String,
    pub scores: Vec<ScoreSummary>,
}

#[allow(clippy::implicit_hasher)]
pub async fn get_event_scores(
    State(state): State<AppState>,
    axum::extract::Path(event_id): axum::extract::Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> ApiResult<Json<GetScoresResponse>> {
    let workspace_id = params
        .get("workspace_id")
        .ok_or_else(|| ApiError::BadRequest("workspace_id query param required".into()))?
        .clone();

    let rows = query_as::<_, (String, String, String, String, String, String, String)>(
        "SELECT scoring_version, scoring_profile_id, final_minutes, final_fec, \
                confidence, confidence_band, created_at \
         FROM heec_scores WHERE workspace_id = ?1 AND event_id = ?2 \
         ORDER BY created_at DESC",
    )
    .bind(&workspace_id)
    .bind(&event_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| ApiError::Storage(e.to_string()))?;

    let scores = rows
        .into_iter()
        .map(|r| ScoreSummary {
            scoring_version: r.0,
            scoring_profile_id: r.1,
            final_minutes: r.2,
            final_fec: r.3,
            confidence: r.4,
            confidence_band: r.5,
            created_at: r.6,
        })
        .collect();

    Ok(Json(GetScoresResponse {
        ok: true,
        envelope_version: "1",
        event_id,
        scores,
    }))
}

// ─── POST /v1/events:batch ──────────────────────────────────────────────────

/// Body for `POST /v1/events:batch`.
#[derive(Deserialize)]
pub struct IngestBatchBody {
    pub workspace_id: String,
    pub events: Vec<serde_json::Value>,
}

#[derive(Serialize)]
pub struct BatchResult {
    pub event_id: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct IngestBatchResponse {
    pub ok: bool,
    pub envelope_version: &'static str,
    pub results: Vec<BatchResult>,
    pub accepted: usize,
    pub rejected: usize,
}

/// Batch ingest: up to 100 events in a single request.
/// Partial success: events that fail validation are rejected individually;
/// the rest are committed in a single transaction.
pub async fn ingest_batch(
    State(state): State<AppState>,
    Json(body): Json<IngestBatchBody>,
) -> ApiResult<Json<IngestBatchResponse>> {
    const MAX_BATCH: usize = 100;

    if body.workspace_id.is_empty() {
        return Err(ApiError::BadRequest("workspace_id is required".into()));
    }
    if body.workspace_id.len() > 128
        || !body
            .workspace_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(ApiError::BadRequest(
            "workspace_id must be 1–128 ASCII alphanumeric, dash, or underscore chars".into(),
        ));
    }
    if body.events.is_empty() {
        return Err(ApiError::BadRequest("events array must not be empty".into()));
    }
    if body.events.len() > MAX_BATCH {
        return Err(ApiError::BadRequest(format!(
            "batch size {len} exceeds maximum {MAX_BATCH}",
            len = body.events.len(),
        )));
    }

    let v = validator();
    let profile = ScoringProfile::default_v1();
    let tiers = TierSet::default_v1();

    // Validate + score each event (outside the transaction).
    let mut results: Vec<BatchResult> = Vec::with_capacity(body.events.len());
    let mut to_persist: Vec<(String, serde_json::Value, Event, ScoreResult)> = Vec::new();

    for raw_event in &body.events {
        if let Err(e) = v.validate(raw_event, Mode::Strict) {
            results.push(BatchResult {
                event_id: raw_event
                    .get("event_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                ok: false,
                error: Some(e.to_string()),
            });
            continue;
        }
        let event: Event = match serde_json::from_value(raw_event.clone()) {
            Ok(e) => e,
            Err(e) => {
                results.push(BatchResult {
                    event_id: raw_event
                        .get("event_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    ok: false,
                    error: Some(format!("materialising Event: {e}")),
                });
                continue;
            }
        };
        match score(&event, &profile, &tiers, None) {
            Ok(r) => {
                let eid = event.event_id.clone();
                to_persist.push((eid, raw_event.clone(), event, r));
            }
            Err(e) => {
                results.push(BatchResult {
                    event_id: event.event_id.clone(),
                    ok: false,
                    error: Some(format!("scoring: {e}")),
                });
            }
        }
    }

    // Persist all valid events in a single transaction.
    if !to_persist.is_empty() {
        let mut tx = state
            .pool
            .begin()
            .await
            .map_err(|e| ApiError::Storage(e.to_string()))?;

        query(
            "INSERT OR IGNORE INTO heec_workspaces (workspace_id, display_name) VALUES (?1, ?1)",
        )
        .bind(&body.workspace_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::Storage(e.to_string()))?;

        for (event_id, raw, event, result) in &to_persist {
            let payload =
                serde_json::to_string(raw).map_err(|e| ApiError::Storage(e.to_string()))?;
            let result_json =
                serde_json::to_string(result).map_err(|e| ApiError::Storage(e.to_string()))?;

            query(
                "INSERT OR IGNORE INTO heec_events \
                 (event_id, workspace_id, spec_version, framework_source, payload, received_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
            )
            .bind(event_id)
            .bind(&body.workspace_id)
            .bind(&result.spec_version)
            .bind(event.framework_source.as_str())
            .bind(&payload)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Storage(e.to_string()))?;

            query(
                "INSERT OR IGNORE INTO heec_scores \
                 (workspace_id, event_id, scoring_version, scoring_profile_id, profile_version, \
                  tier_id, tier_version, rates_version, result_json, \
                  final_minutes, final_fec, confidence, confidence_band, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, \
                         strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
            )
            .bind(&body.workspace_id)
            .bind(event_id)
            .bind(&result.scoring_version)
            .bind(&result.scoring_profile)
            .bind(&profile.version)
            .bind(&result.tier.id)
            .bind(&tiers.version)
            .bind(&tiers.version)
            .bind(&result_json)
            .bind(result.final_estimated_minutes.to_string())
            .bind(result.financial_equivalent_cost.to_string())
            .bind(result.confidence_score.to_string())
            .bind(format!("{:?}", result.confidence_band))
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Storage(e.to_string()))?;

            results.push(BatchResult {
                event_id: event_id.clone(),
                ok: true,
                error: None,
            });
        }

        tx.commit()
            .await
            .map_err(|e| ApiError::Storage(e.to_string()))?;
    }

    let accepted = results.iter().filter(|r| r.ok).count();
    let rejected = results.len() - accepted;

    Ok(Json(IngestBatchResponse {
        ok: true,
        envelope_version: "1",
        results,
        accepted,
        rejected,
    }))
}
