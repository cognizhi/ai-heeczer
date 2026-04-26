//! API-key authentication middleware for protected ingestion routes.

use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderMap, Request};
use axum::middleware::Next;
use axum::response::Response;
use sha2::{Digest, Sha256};
use sqlx_core::query::query;
use sqlx_core::query_as::query_as;
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::state::{AppState, AuthContext};

const API_KEY_HEADER: &str = "x-heeczer-api-key";

/// Hash a raw API key exactly as stored in `heec_api_keys.hashed_key`.
pub fn hash_api_key(api_key: &str) -> String {
    let digest = Sha256::digest(api_key.as_bytes());
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

fn raw_api_key(headers: &HeaderMap) -> ApiResult<&str> {
    headers
        .get(API_KEY_HEADER)
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| ApiError::Unauthorized("x-heeczer-api-key header required".into()))
}

async fn audit_auth_failure(
    state: &AppState,
    workspace_id: Option<&str>,
    reason: &str,
) -> ApiResult<()> {
    let payload = serde_json::json!({ "reason": reason });
    query(
        "INSERT INTO heec_audit_log \
         (audit_id, workspace_id, actor, action, target_table, target_id, payload_json) \
         VALUES (?1, ?2, 'anonymous', 'auth_failed', 'heec_api_keys', NULL, ?3)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(workspace_id)
    .bind(serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string()))
    .execute(&state.pool)
    .await
    .map_err(|e| ApiError::Storage(e.to_string()))?;
    Ok(())
}

/// Authenticate protected routes and attach `AuthContext` to request extensions.
///
/// When auth is disabled (test/dev mode), the middleware still inserts an
/// anonymous context so handlers have one workspace-scoping path.
pub async fn authenticate(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    if !state.auth.enabled {
        request.extensions_mut().insert(AuthContext::anonymous());
        return Ok(next.run(request).await);
    }

    let raw_key = match raw_api_key(request.headers()) {
        Ok(key) => key,
        Err(err) => {
            audit_auth_failure(&state, None, "missing_api_key").await?;
            return Err(err);
        }
    };
    let hashed_key = hash_api_key(raw_key);

    let row: Option<(String, String, Option<String>)> = query_as(
        "SELECT api_key_id, workspace_id, revoked_at \
         FROM heec_api_keys WHERE hashed_key = ?1",
    )
    .bind(&hashed_key)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::Storage(e.to_string()))?;

    let Some((api_key_id, workspace_id, revoked_at)) = row else {
        audit_auth_failure(&state, None, "unknown_api_key").await?;
        return Err(ApiError::Unauthorized("invalid API key".into()));
    };

    if revoked_at.is_some() {
        audit_auth_failure(&state, Some(&workspace_id), "revoked_api_key").await?;
        return Err(ApiError::Unauthorized("API key has been revoked".into()));
    }

    if let Err(decision) = state.rate_limiter.check(&api_key_id, state.rate_limit) {
        return Err(ApiError::RateLimited {
            message: "request rate exceeded".into(),
            retry_after_seconds: decision.retry_after_seconds,
            limit: i64::from(decision.limit),
            remaining: i64::from(decision.remaining),
            reset_after_seconds: decision.retry_after_seconds,
        });
    }

    request.extensions_mut().insert(AuthContext {
        workspace_id,
        api_key_id,
        authenticated: true,
    });

    Ok(next.run(request).await)
}
