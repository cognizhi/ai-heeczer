//! HTTP handlers. See `lib.rs` for the route table.

use std::sync::OnceLock;

use axum::body::{Body, Bytes};
use axum::extract::{Extension, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::Response;
use axum::Json;
use heeczer_core::schema::{EventValidator, Mode};
use heeczer_core::{
    score, Event, ScoreResult, ScoringProfile, TierSet, SCORING_VERSION, SPEC_VERSION,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx_core::query::query;
use sqlx_core::query_as::query_as;
use uuid::Uuid;

use crate::auth::hash_api_key;
use crate::error::{ApiError, ApiResult};
use crate::state::{AppState, AuthContext};

fn validator() -> &'static EventValidator {
    static V: OnceLock<EventValidator> = OnceLock::new();
    V.get_or_init(EventValidator::new_v1)
}

fn validate_workspace_id(workspace_id: &str) -> ApiResult<()> {
    if workspace_id.is_empty()
        || workspace_id.len() > 64
        || !workspace_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(ApiError::BadRequest(
            "workspace_id must be 1-64 ASCII alphanumeric, dot, dash, or underscore chars".into(),
        ));
    }
    Ok(())
}

fn normalize_event_workspace(workspace_id: &str, raw_event: &mut Value) {
    if let Value::Object(map) = raw_event {
        map.insert(
            "workspace_id".into(),
            Value::String(workspace_id.to_owned()),
        );
    }
}

fn workspace_from_body(auth: &AuthContext, workspace_id: &str) -> ApiResult<String> {
    validate_workspace_id(workspace_id)?;
    if auth.authenticated && auth.workspace_id != workspace_id {
        return Err(ApiError::Forbidden(
            "workspace_id does not match authenticated API key".into(),
        ));
    }
    Ok(workspace_id.to_owned())
}

#[allow(clippy::implicit_hasher)]
fn workspace_from_query(
    auth: &AuthContext,
    params: &std::collections::HashMap<String, String>,
) -> ApiResult<String> {
    match params.get("workspace_id") {
        Some(workspace_id) => workspace_from_body(auth, workspace_id),
        None if auth.authenticated => Ok(auth.workspace_id.clone()),
        None => Err(ApiError::BadRequest(
            "workspace_id query param required".into(),
        )),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

fn score_profile_version_key(
    profile: &ScoringProfile,
    tiers: &TierSet,
    tier_override: Option<&str>,
) -> ApiResult<String> {
    if profile == &ScoringProfile::default_v1()
        && tiers == &TierSet::default_v1()
        && tier_override.is_none()
    {
        return Ok(profile.version.clone());
    }

    let config = serde_json::json!({
        "profile": profile,
        "tier_set": tiers,
        "tier_override": tier_override,
    });
    let config_bytes = serde_json::to_vec(&config).map_err(|e| ApiError::Storage(e.to_string()))?;
    let hash = sha256_hex(&config_bytes);
    Ok(format!("{}+{}", profile.version, &hash[..16]))
}

async fn ensure_workspace(pool: &sqlx_sqlite::SqlitePool, workspace_id: &str) -> ApiResult<()> {
    query("INSERT OR IGNORE INTO heec_workspaces (workspace_id, display_name) VALUES (?1, ?1)")
        .bind(workspace_id)
        .execute(pool)
        .await
        .map_err(|e| ApiError::Storage(e.to_string()))?;
    Ok(())
}

async fn workspace_daily_quota(state: &AppState, workspace_id: &str) -> ApiResult<i64> {
    let settings: Option<(String,)> =
        query_as("SELECT settings_json FROM heec_workspaces WHERE workspace_id = ?1")
            .bind(workspace_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| ApiError::Storage(e.to_string()))?;
    let Some((settings_json,)) = settings else {
        return Ok(state.quotas.daily_events);
    };
    let parsed: serde_json::Value = serde_json::from_str(&settings_json)
        .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::default()));
    Ok(parsed
        .get("daily_event_quota")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(state.quotas.daily_events))
}

async fn enforce_daily_quota(
    state: &AppState,
    workspace_id: &str,
    incoming_new_events: usize,
) -> ApiResult<()> {
    if incoming_new_events == 0 {
        return Ok(());
    }
    let limit = workspace_daily_quota(state, workspace_id).await?;
    if limit < 0 {
        return Ok(());
    }
    let (used,): (i64,) = query_as(
        "SELECT COUNT(*) FROM heec_events \
         WHERE workspace_id = ?1 AND received_at >= date('now') || 'T00:00:00Z'",
    )
    .bind(workspace_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| ApiError::Storage(e.to_string()))?;
    let incoming = i64::try_from(incoming_new_events).unwrap_or(i64::MAX);
    if used.saturating_add(incoming) > limit {
        return Err(ApiError::RateLimited {
            message: "workspace daily event quota exceeded".into(),
            retry_after_seconds: 86_400,
            limit,
            remaining: (limit - used).max(0),
            reset_after_seconds: 86_400,
        });
    }
    Ok(())
}

async fn fetch_existing_score(
    state: &AppState,
    workspace_id: &str,
    event_id: &str,
) -> ApiResult<Option<ScoreResult>> {
    let row: Option<(String,)> = query_as(
        "SELECT result_json FROM heec_scores \
         WHERE workspace_id = ?1 AND event_id = ?2 \
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(workspace_id)
    .bind(event_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::Storage(e.to_string()))?;
    row.map(|(json,)| serde_json::from_str(&json).map_err(|e| ApiError::Storage(e.to_string())))
        .transpose()
}

async fn audit_ingest_conflict(
    state: &AppState,
    workspace_id: &str,
    event_id: &str,
    payload_hash: &str,
) -> ApiResult<()> {
    let payload = serde_json::json!({
        "event_id": event_id,
        "payload_hash": payload_hash,
        "reason": "event_id_payload_conflict",
    });
    query(
        "INSERT INTO heec_audit_log \
         (audit_id, workspace_id, actor, action, target_table, target_id, payload_json) \
         VALUES (?1, ?2, 'service', 'ingest_conflict', 'heec_events', ?3, ?4)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(workspace_id)
    .bind(event_id)
    .bind(serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string()))
    .execute(&state.pool)
    .await
    .map_err(|e| ApiError::Storage(e.to_string()))?;
    Ok(())
}

async fn audit_test_orchestration(
    state: &AppState,
    auth: &AuthContext,
    allowed: bool,
    reason: &str,
) -> ApiResult<()> {
    let payload = serde_json::json!({
        "allowed": allowed,
        "reason": reason,
    });
    let workspace_id = auth.authenticated.then_some(auth.workspace_id.as_str());
    query(
        "INSERT INTO heec_audit_log \
         (audit_id, workspace_id, actor, action, target_table, target_id, payload_json) \
         VALUES (?1, ?2, ?3, 'test_score_pipeline', NULL, NULL, ?4)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(workspace_id)
    .bind(&auth.api_key_id)
    .bind(serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string()))
    .execute(&state.pool)
    .await
    .map_err(|e| ApiError::Storage(e.to_string()))?;
    Ok(())
}

async fn reserve_or_replay_idempotent_batch(
    state: &AppState,
    workspace_id: &str,
    idempotency_key: &str,
    request_hash: &str,
) -> ApiResult<Option<Response>> {
    let inserted = query(
        "INSERT OR IGNORE INTO heec_idempotency_keys \
         (workspace_id, idempotency_key, request_hash, status_code, response_body, created_at, expires_at) \
         VALUES (?1, ?2, ?3, 0, '', \
                 strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), \
                 strftime('%Y-%m-%dT%H:%M:%fZ', 'now', ?4))",
    )
    .bind(workspace_id)
    .bind(idempotency_key)
    .bind(request_hash)
    .bind(format!("+{} hours", state.idempotency.retention_hours))
    .execute(&state.pool)
    .await
    .map_err(|e| ApiError::Storage(e.to_string()))?;
    if inserted.rows_affected() > 0 {
        return Ok(None);
    }

    let row: Option<(String, i64, String)> = query_as(
        "SELECT request_hash, status_code, response_body FROM heec_idempotency_keys \
         WHERE workspace_id = ?1 AND idempotency_key = ?2 \
           AND expires_at > strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
    )
    .bind(workspace_id)
    .bind(idempotency_key)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::Storage(e.to_string()))?;

    let Some((stored_hash, status_code, response_body)) = row else {
        return Ok(None);
    };
    if stored_hash != request_hash {
        return Err(ApiError::Conflict(
            "Idempotency-Key was already used with a different request body".into(),
        ));
    }
    if status_code == 0 {
        return Err(ApiError::Conflict(
            "Idempotency-Key request is already in progress".into(),
        ));
    }

    let mut response = Response::new(Body::from(response_body));
    *response.status_mut() =
        StatusCode::from_u16(u16::try_from(status_code).unwrap_or(200)).unwrap_or(StatusCode::OK);
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    Ok(Some(response))
}

async fn store_idempotent_batch(
    state: &AppState,
    workspace_id: &str,
    idempotency_key: &str,
    request_hash: &str,
    status: StatusCode,
    response_body: &str,
) -> ApiResult<()> {
    let updated = query(
        "UPDATE heec_idempotency_keys \
         SET status_code = ?4, response_body = ?5 \
         WHERE workspace_id = ?1 AND idempotency_key = ?2 AND request_hash = ?3",
    )
    .bind(workspace_id)
    .bind(idempotency_key)
    .bind(request_hash)
    .bind(i64::from(status.as_u16()))
    .bind(response_body)
    .execute(&state.pool)
    .await
    .map_err(|e| ApiError::Storage(e.to_string()))?;
    if updated.rows_affected() == 0 {
        return Err(ApiError::Conflict(
            "Idempotency-Key reservation changed before response storage".into(),
        ));
    }
    Ok(())
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
    Extension(auth): Extension<AuthContext>,
    body: Bytes,
) -> ApiResult<Json<IngestEventResponse>> {
    if body.len() > state.payload_limits.event_bytes {
        return Err(ApiError::PayloadTooLarge(format!(
            "single-event payload exceeds {} bytes",
            state.payload_limits.event_bytes
        )));
    }
    let mut body: IngestEventBody = serde_json::from_slice(&body)
        .map_err(|e| ApiError::BadRequest(format!("parsing request body: {e}")))?;
    let workspace_id = workspace_from_body(&auth, &body.workspace_id)?;
    normalize_event_workspace(&workspace_id, &mut body.event);

    // Extract and route on spec_version before full schema validation (ADR-0002).
    // spec_version is the routing key: reject unsupported versions early with a
    // clear, actionable error message.
    let spec_version = body
        .event
        .get("spec_version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::BadRequest("spec_version is required in event".into()))?;
    if spec_version != "1.0" {
        return Err(ApiError::UnsupportedSpecVersion(format!(
            "unsupported spec_version `{spec_version}`; this server accepts: 1.0"
        )));
    }

    validator()
        .validate(&body.event, Mode::Strict)
        .map_err(|e| ApiError::Schema(e.to_string()))?;
    let event: Event = serde_json::from_value(body.event.clone())
        .map_err(|e| ApiError::BadRequest(format!("materialising Event: {e}")))?;
    let event_id = event.event_id.clone();

    // Record event metadata in the current tracing span so downstream log
    // aggregation can correlate all records for a given event_id / correlation_id
    // (plan 0004). We use tracing::info! here rather than a child span guard
    // because EnteredSpan is !Send and cannot be held across .await points.
    tracing::info!(
        event_id = %event_id,
        correlation_id = event.correlation_id.as_deref().unwrap_or_default(),
        workspace_id = %workspace_id,
        request_id = %auth.api_key_id,
        "ingest_event: processing",
    );

    let profile = ScoringProfile::default_v1();
    let tiers = TierSet::default_v1();
    let result =
        score(&event, &profile, &tiers, None).map_err(|e| ApiError::Scoring(e.to_string()))?;

    let payload =
        serde_json::to_string(&body.event).map_err(|e| ApiError::Storage(e.to_string()))?;
    let payload_hash = hash_api_key(&payload);

    if let Some((existing_hash,)) = query_as::<_, (String,)>(
        "SELECT payload_hash FROM heec_events WHERE workspace_id = ?1 AND event_id = ?2",
    )
    .bind(&workspace_id)
    .bind(&event_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::Storage(e.to_string()))?
    {
        if existing_hash == payload_hash || existing_hash.is_empty() {
            if let Some(existing_score) =
                fetch_existing_score(&state, &workspace_id, &event_id).await?
            {
                return Ok(Json(IngestEventResponse {
                    ok: true,
                    envelope_version: "1",
                    event_id,
                    score: existing_score,
                }));
            }
        }
        audit_ingest_conflict(&state, &workspace_id, &event_id, &payload_hash).await?;
        return Err(ApiError::Conflict(format!(
            "event_id {event_id} already exists with a different normalized payload"
        )));
    }

    enforce_daily_quota(&state, &workspace_id, 1).await?;
    let result_json =
        serde_json::to_string(&result).map_err(|e| ApiError::Storage(e.to_string()))?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| ApiError::Storage(e.to_string()))?;

    query("INSERT OR IGNORE INTO heec_workspaces (workspace_id, display_name) VALUES (?1, ?1)")
        .bind(&workspace_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::Storage(e.to_string()))?;

    // PRD §19.4: duplicate event_id → return existing record. Use INSERT OR IGNORE
    // and rely on the (workspace_id, event_id) PK to dedupe.
    let event_insert = query(
        "INSERT OR IGNORE INTO heec_events
            (event_id, workspace_id, spec_version, framework_source, correlation_id, payload, payload_hash, received_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
    )
    .bind(&event_id)
    .bind(&workspace_id)
    .bind(&result.spec_version)
    .bind(event.framework_source.as_str())
    .bind(event.correlation_id.as_deref())
    .bind(&payload)
    .bind(&payload_hash)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Storage(e.to_string()))?;

    if event_insert.rows_affected() == 0 {
        tx.rollback()
            .await
            .map_err(|e| ApiError::Storage(e.to_string()))?;
        let Some((existing_hash,)) = query_as::<_, (String,)>(
            "SELECT payload_hash FROM heec_events WHERE workspace_id = ?1 AND event_id = ?2",
        )
        .bind(&workspace_id)
        .bind(&event_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| ApiError::Storage(e.to_string()))?
        else {
            return Err(ApiError::Conflict(format!(
                "event_id {event_id} was concurrently written but could not be replayed"
            )));
        };
        if existing_hash == payload_hash || existing_hash.is_empty() {
            if let Some(existing_score) =
                fetch_existing_score(&state, &workspace_id, &event_id).await?
            {
                return Ok(Json(IngestEventResponse {
                    ok: true,
                    envelope_version: "1",
                    event_id,
                    score: existing_score,
                }));
            }
        }
        audit_ingest_conflict(&state, &workspace_id, &event_id, &payload_hash).await?;
        return Err(ApiError::Conflict(format!(
            "event_id {event_id} already exists with a different normalized payload"
        )));
    }

    query(
        "INSERT OR IGNORE INTO heec_scores
         (workspace_id, event_id, scoring_version, scoring_profile_id, profile_version,
          tier_id, tier_version, rates_version, result_json,
          final_minutes, final_fec, confidence, confidence_band, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                 strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
    )
    .bind(&workspace_id)
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
    Extension(auth): Extension<AuthContext>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<TestPipelineResponse>> {
    if !state.features.test_orchestration {
        audit_test_orchestration(&state, &auth, false, "feature_disabled").await?;
        return Err(ApiError::FeatureDisabled(
            "test_orchestration is not enabled on this deployment".into(),
        ));
    }
    if headers
        .get("x-heeczer-tester")
        .and_then(|v| v.to_str().ok())
        != Some("1")
    {
        audit_test_orchestration(&state, &auth, false, "tester_header_missing").await?;
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
    audit_test_orchestration(&state, &auth, true, "allowed").await?;
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
    Extension(auth): Extension<AuthContext>,
    axum::extract::Path(event_id): axum::extract::Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> ApiResult<Json<GetEventResponse>> {
    let workspace_id = workspace_from_query(&auth, &params)?;

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
    Extension(auth): Extension<AuthContext>,
    axum::extract::Path(event_id): axum::extract::Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> ApiResult<Json<GetScoresResponse>> {
    let workspace_id = workspace_from_query(&auth, &params)?;

    // Verify the event exists before querying scores; otherwise the scores query
    // returns an empty array for both "event not found" and "zero scores stored",
    // which callers cannot distinguish.
    let event_exists: Option<(String,)> =
        query_as("SELECT event_id FROM heec_events WHERE workspace_id = ?1 AND event_id = ?2")
            .bind(&workspace_id)
            .bind(&event_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| ApiError::Storage(e.to_string()))?;
    if event_exists.is_none() {
        return Err(ApiError::NotFound(format!("event {event_id} not found")));
    }

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
    Extension(auth): Extension<AuthContext>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Response> {
    const MAX_BATCH: usize = 100;

    if body.len() > state.payload_limits.batch_bytes {
        return Err(ApiError::PayloadTooLarge(format!(
            "batch payload exceeds {} bytes",
            state.payload_limits.batch_bytes
        )));
    }
    let request_hash = sha256_hex(&body);
    let body: IngestBatchBody = serde_json::from_slice(&body)
        .map_err(|e| ApiError::BadRequest(format!("parsing request body: {e}")))?;
    let workspace_id = workspace_from_body(&auth, &body.workspace_id)?;

    let idempotency_key = headers
        .get("Idempotency-Key")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);

    if body.events.is_empty() {
        return Err(ApiError::BadRequest(
            "events array must not be empty".into(),
        ));
    }
    if body.events.len() > MAX_BATCH {
        return Err(ApiError::BadRequest(format!(
            "batch size {len} exceeds maximum {MAX_BATCH}",
            len = body.events.len(),
        )));
    }

    ensure_workspace(&state.pool, &workspace_id).await?;

    if let Some(key) = &idempotency_key {
        if key.len() > 128 {
            return Err(ApiError::BadRequest(
                "Idempotency-Key must be at most 128 characters".into(),
            ));
        }
        if let Some(response) =
            reserve_or_replay_idempotent_batch(&state, &workspace_id, key, &request_hash).await?
        {
            return Ok(response);
        }
    }

    tracing::info!(
        workspace_id = %workspace_id,
        batch_size = body.events.len(),
        request_id = %auth.api_key_id,
        "ingest_batch: processing",
    );

    let v = validator();
    let profile = ScoringProfile::default_v1();
    let tiers = TierSet::default_v1();

    // Validate + score each event (outside the transaction).
    let mut results: Vec<BatchResult> = Vec::with_capacity(body.events.len());
    let mut to_persist: Vec<(String, serde_json::Value, Event, ScoreResult, String)> = Vec::new();

    for raw_event in &body.events {
        let mut raw_event = raw_event.clone();
        normalize_event_workspace(&workspace_id, &mut raw_event);
        if let Some(spec_version) = raw_event.get("spec_version").and_then(|v| v.as_str()) {
            if spec_version != "1.0" {
                return Err(ApiError::UnsupportedSpecVersion(format!(
                    "unsupported spec_version `{spec_version}`; this server accepts: 1.0"
                )));
            }
        }
        if let Err(e) = v.validate(&raw_event, Mode::Strict) {
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
                let payload = serde_json::to_string(&raw_event)
                    .map_err(|e| ApiError::Storage(e.to_string()))?;
                let payload_hash = hash_api_key(&payload);
                if let Some((existing_hash,)) = query_as::<_, (String,)>(
                    "SELECT payload_hash FROM heec_events \
                     WHERE workspace_id = ?1 AND event_id = ?2",
                )
                .bind(&workspace_id)
                .bind(&eid)
                .fetch_optional(&state.pool)
                .await
                .map_err(|e| ApiError::Storage(e.to_string()))?
                {
                    if existing_hash == payload_hash || existing_hash.is_empty() {
                        results.push(BatchResult {
                            event_id: eid,
                            ok: true,
                            error: None,
                        });
                    } else {
                        audit_ingest_conflict(&state, &workspace_id, &eid, &payload_hash).await?;
                        results.push(BatchResult {
                            event_id: eid,
                            ok: false,
                            error: Some(
                                "event_id already exists with a different normalized payload"
                                    .into(),
                            ),
                        });
                    }
                    continue;
                }
                to_persist.push((eid, raw_event, event, r, payload_hash));
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

    enforce_daily_quota(&state, &workspace_id, to_persist.len()).await?;

    // Persist all valid events in a single transaction.
    if !to_persist.is_empty() {
        let mut tx = state
            .pool
            .begin()
            .await
            .map_err(|e| ApiError::Storage(e.to_string()))?;

        query("INSERT OR IGNORE INTO heec_workspaces (workspace_id, display_name) VALUES (?1, ?1)")
            .bind(&workspace_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Storage(e.to_string()))?;

        for (event_id, raw, event, result, payload_hash) in &to_persist {
            let payload =
                serde_json::to_string(raw).map_err(|e| ApiError::Storage(e.to_string()))?;
            let result_json =
                serde_json::to_string(result).map_err(|e| ApiError::Storage(e.to_string()))?;

            let event_insert = query(
                "INSERT OR IGNORE INTO heec_events \
                  (event_id, workspace_id, spec_version, framework_source, correlation_id, payload, payload_hash, received_at) \
                  VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
            )
            .bind(event_id)
            .bind(&workspace_id)
            .bind(&result.spec_version)
            .bind(event.framework_source.as_str())
            .bind(event.correlation_id.as_deref())
            .bind(&payload)
            .bind(payload_hash)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Storage(e.to_string()))?;

            if event_insert.rows_affected() == 0 {
                let existing_hash: Option<(String,)> = query_as(
                    "SELECT payload_hash FROM heec_events \
                     WHERE workspace_id = ?1 AND event_id = ?2",
                )
                .bind(&workspace_id)
                .bind(event_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| ApiError::Storage(e.to_string()))?;
                if existing_hash.as_ref().is_some_and(|(existing_hash,)| {
                    existing_hash == payload_hash || existing_hash.is_empty()
                }) {
                    results.push(BatchResult {
                        event_id: event_id.clone(),
                        ok: true,
                        error: None,
                    });
                } else {
                    let audit_payload = serde_json::json!({
                        "event_id": event_id,
                        "payload_hash": payload_hash,
                        "reason": "event_id_payload_conflict",
                    });
                    query(
                        "INSERT INTO heec_audit_log \
                         (audit_id, workspace_id, actor, action, target_table, target_id, payload_json) \
                         VALUES (?1, ?2, 'service', 'ingest_conflict', 'heec_events', ?3, ?4)",
                    )
                    .bind(Uuid::new_v4().to_string())
                    .bind(&workspace_id)
                    .bind(event_id)
                    .bind(serde_json::to_string(&audit_payload).unwrap_or_else(|_| "{}".to_string()))
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| ApiError::Storage(e.to_string()))?;
                    results.push(BatchResult {
                        event_id: event_id.clone(),
                        ok: false,
                        error: Some(
                            "event_id already exists with a different normalized payload".into(),
                        ),
                    });
                }
                continue;
            }

            query(
                "INSERT OR IGNORE INTO heec_scores \
                 (workspace_id, event_id, scoring_version, scoring_profile_id, profile_version, \
                  tier_id, tier_version, rates_version, result_json, \
                  final_minutes, final_fec, confidence, confidence_band, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, \
                         strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
            )
            .bind(&workspace_id)
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

    let response = IngestBatchResponse {
        ok: true,
        envelope_version: "1",
        results,
        accepted,
        rejected,
    };
    let response_body =
        serde_json::to_string(&response).map_err(|e| ApiError::Storage(e.to_string()))?;
    if let Some(key) = &idempotency_key {
        store_idempotent_batch(
            &state,
            &workspace_id,
            key,
            &request_hash,
            StatusCode::OK,
            &response_body,
        )
        .await?;
    }
    let mut http_response = Response::new(Body::from(response_body));
    http_response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    Ok(http_response)
}

// ─── GET /v1/ready ──────────────────────────────────────────────────────────

/// Readiness probe. Returns 200 when the service can accept traffic (DB
/// reachable). Returns 503 otherwise. Kubernetes / load-balancer health checks
/// should prefer `/v1/ready` over `/healthz`; liveness (`/healthz`) only
/// checks that the process is alive.
pub async fn ready(State(state): State<AppState>) -> ApiResult<Json<HealthResponse>> {
    sqlx_core::query::query("SELECT 1")
        .execute(&state.pool)
        .await
        .map_err(|e| ApiError::Unavailable(format!("readiness check: {e}")))?;
    Ok(Json(HealthResponse {
        ok: true,
        envelope_version: "1",
    }))
}

// ─── POST /v1/events/{event_id}:rescore ─────────────────────────────────────

/// Optional overrides for the re-score. When absent the service defaults are
/// used. This intentionally mirrors the body of `POST /v1/test/score-pipeline`
/// so the dashboard can exercise the same paths against live data.
#[derive(Deserialize)]
pub struct RescoreBody {
    pub workspace_id: String,
    pub profile: Option<serde_json::Value>,
    pub tier_set: Option<serde_json::Value>,
    pub tier_override: Option<String>,
}

#[derive(Serialize)]
pub struct RescoreResponse {
    pub ok: bool,
    pub envelope_version: &'static str,
    pub event_id: String,
    pub score: ScoreResult,
}

/// Re-score an existing event. Inserts a new `heec_scores` row for the
/// provided scoring parameters and writes an audit log entry. If a score
/// already exists for the exact same `(event_id, scoring_version,
/// scoring_profile_id, profile_version)` tuple, the existing row is returned
/// and no duplicate is written (append-only invariant; plan 0003).
pub async fn rescore_event(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    axum::extract::Path(event_id): axum::extract::Path<String>,
    Json(body): Json<RescoreBody>,
) -> ApiResult<Json<RescoreResponse>> {
    let workspace_id = workspace_from_body(&auth, &body.workspace_id)?;

    // Reject if the event has been hard-deleted (tombstoned).
    let tombstoned: Option<(String,)> =
        query_as("SELECT event_id FROM heec_tombstones WHERE workspace_id = ?1 AND event_id = ?2")
            .bind(&workspace_id)
            .bind(&event_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| ApiError::Storage(e.to_string()))?;
    if tombstoned.is_some() {
        return Err(ApiError::NotFound(format!(
            "event {event_id} has been deleted"
        )));
    }

    // Look up the stored canonical payload.
    let row: Option<(String, Option<String>)> = query_as(
        "SELECT payload, correlation_id FROM heec_events \
         WHERE workspace_id = ?1 AND event_id = ?2",
    )
    .bind(&workspace_id)
    .bind(&event_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::Storage(e.to_string()))?;
    let (payload_str, correlation_id) =
        row.ok_or_else(|| ApiError::NotFound(format!("event {event_id} not found")))?;

    let raw_event: serde_json::Value = serde_json::from_str(&payload_str)
        .map_err(|e| ApiError::Storage(format!("deserialising stored payload: {e}")))?;
    let event: Event = serde_json::from_value(raw_event)
        .map_err(|e| ApiError::Storage(format!("materialising Event from stored payload: {e}")))?;

    tracing::info!(
        event_id = %event_id,
        correlation_id = correlation_id.as_deref().unwrap_or_default(),
        workspace_id = %workspace_id,
        request_id = %auth.api_key_id,
        "rescore_event: processing",
    );

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
    let tier_override = body.tier_override.as_deref();
    let result = score(&event, &profile, &tiers, tier_override)
        .map_err(|e| ApiError::Scoring(e.to_string()))?;
    let profile_version_key = score_profile_version_key(&profile, &tiers, tier_override)?;
    let result_json =
        serde_json::to_string(&result).map_err(|e| ApiError::Storage(e.to_string()))?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| ApiError::Storage(e.to_string()))?;

    let score_insert = query(
        "INSERT OR IGNORE INTO heec_scores
         (workspace_id, event_id, scoring_version, scoring_profile_id, profile_version,
          tier_id, tier_version, rates_version, result_json,
          final_minutes, final_fec, confidence, confidence_band, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                 strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
    )
    .bind(&workspace_id)
    .bind(&event_id)
    .bind(&result.scoring_version)
    .bind(&result.scoring_profile)
    .bind(&profile_version_key)
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

    // Only write the audit log entry when a new score row was actually inserted.
    // INSERT OR IGNORE is a no-op when the (PK) tuple already exists; emitting
    // an audit entry in that case would create a phantom record that falsely
    // implies a re-scoring event occurred (plan 0003 append-only invariant).
    if score_insert.rows_affected() > 0 {
        let audit_payload = serde_json::json!({
            "scoring_version": &result.scoring_version,
            "scoring_profile": &result.scoring_profile,
            "profile_version": &profile_version_key,
            "tier_id": &result.tier.id,
        });
        query(
            "INSERT INTO heec_audit_log \
             (audit_id, workspace_id, actor, action, target_table, target_id, payload_json) \
               VALUES (?1, ?2, 'service', 'rescore', 'heec_scores', ?3, ?4)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(&workspace_id)
        .bind(&event_id)
        .bind(serde_json::to_string(&audit_payload).unwrap_or_else(|_| "{}".to_string()))
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::Storage(e.to_string()))?;
    }

    tx.commit()
        .await
        .map_err(|e| ApiError::Storage(e.to_string()))?;

    Ok(Json(RescoreResponse {
        ok: true,
        envelope_version: "1",
        event_id,
        score: result,
    }))
}

// ─── GET /v1/jobs/{job_id} ──────────────────────────────────────────────────

#[derive(Serialize)]
pub struct GetJobResponse {
    pub ok: bool,
    pub envelope_version: &'static str,
    pub job_id: String,
    pub workspace_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    pub state: String,
    pub attempts: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    pub enqueued_at: String,
    pub available_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<String>,
}

#[allow(clippy::implicit_hasher)]
pub async fn get_job(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    axum::extract::Path(job_id): axum::extract::Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> ApiResult<Json<GetJobResponse>> {
    let workspace_id = workspace_from_query(&auth, &params)?;

    let row = query_as::<
        _,
        (
            String,
            String,
            Option<String>,
            String,
            i64,
            Option<String>,
            String,
            String,
            Option<String>,
        ),
    >(
        "SELECT job_id, workspace_id, event_id, state, attempts, last_error, \
                enqueued_at, available_at, finished_at \
         FROM heec_jobs WHERE workspace_id = ?1 AND job_id = ?2",
    )
    .bind(&workspace_id)
    .bind(&job_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::Storage(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound(format!("job {job_id} not found")))?;

    Ok(Json(GetJobResponse {
        ok: true,
        envelope_version: "1",
        job_id: row.0,
        workspace_id: row.1,
        event_id: row.2,
        state: row.3,
        attempts: row.4,
        last_error: row.5,
        enqueued_at: row.6,
        available_at: row.7,
        finished_at: row.8,
    }))
}
