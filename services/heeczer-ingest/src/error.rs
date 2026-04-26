//! Structured error type that maps cleanly into HTTP responses.
//!
//! Error bodies follow a closed-shape envelope so SDKs can match on `kind`
//! without fragile string parsing (mirrors the C ABI envelope contract from
//! ADR-0011).

use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

/// Supported spec versions advertised in `Supported-Spec-Versions` headers.
const SUPPORTED_SPEC_VERSIONS: &str = "1.0";

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("schema validation failed: {0}")]
    Schema(String),
    #[error("malformed request body: {0}")]
    BadRequest(String),
    #[error("scoring failed: {0}")]
    Scoring(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("payload too large: {0}")]
    PayloadTooLarge(String),
    #[error("rate limited: {message}")]
    RateLimited {
        message: String,
        retry_after_seconds: u64,
        limit: i64,
        remaining: i64,
        reset_after_seconds: u64,
    },
    #[error("feature disabled: {0}")]
    FeatureDisabled(String),
    /// 415 — `spec_version` not supported by this server. Callers should
    /// inspect the `Supported-Spec-Versions` response header (ADR-0002).
    #[error("unsupported spec_version: {0}")]
    UnsupportedSpecVersion(String),
    /// 503 — the service is alive but not ready to handle traffic (e.g. DB
    /// unreachable). Used exclusively by `GET /v1/ready`.
    #[error("service unavailable: {0}")]
    Unavailable(String),
}

impl ApiError {
    fn kind(&self) -> &'static str {
        match self {
            ApiError::Schema(_) => "schema",
            ApiError::BadRequest(_) => "bad_request",
            ApiError::Scoring(_) => "scoring",
            ApiError::Storage(_) => "storage",
            ApiError::NotFound(_) => "not_found",
            ApiError::Unauthorized(_) => "unauthorized",
            ApiError::Forbidden(_) => "forbidden",
            ApiError::Conflict(_) => "conflict",
            ApiError::PayloadTooLarge(_) => "payload_too_large",
            ApiError::RateLimited { .. } => "rate_limit_exceeded",
            ApiError::FeatureDisabled(_) => "feature_disabled",
            ApiError::UnsupportedSpecVersion(_) => "unsupported_spec_version",
            ApiError::Unavailable(_) => "unavailable",
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            ApiError::Schema(_) | ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Scoring(_) | ApiError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
            ApiError::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            ApiError::UnsupportedSpecVersion(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ApiError::FeatureDisabled(_) | ApiError::Unavailable(_) => {
                StatusCode::SERVICE_UNAVAILABLE
            }
        }
    }
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    ok: bool,
    envelope_version: &'a str,
    error: ErrorPayload<'a>,
}

#[derive(Serialize)]
struct ErrorPayload<'a> {
    kind: &'a str,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // Log the full internal detail for Storage and Scoring errors so operators
        // can diagnose problems without exposing internals to callers.
        let client_message = match &self {
            ApiError::Storage(detail) => {
                tracing::error!(kind = "storage", detail = %detail, "storage error");
                "a storage error occurred".to_string()
            }
            ApiError::Scoring(detail) => {
                tracing::error!(kind = "scoring", detail = %detail, "scoring error");
                "a scoring error occurred".to_string()
            }
            other => other.to_string(),
        };
        let status = self.status();
        let body = ErrorBody {
            ok: false,
            envelope_version: "1",
            error: ErrorPayload {
                kind: self.kind(),
                message: client_message,
            },
        };
        let mut response = (status, Json(body)).into_response();
        // ADR-0002: 415 responses advertise the supported spec versions so callers
        // can upgrade without consulting docs.
        if status == StatusCode::UNSUPPORTED_MEDIA_TYPE {
            response.headers_mut().insert(
                "Supported-Spec-Versions",
                HeaderValue::from_static(SUPPORTED_SPEC_VERSIONS),
            );
        }
        if let ApiError::RateLimited {
            retry_after_seconds,
            limit,
            remaining,
            reset_after_seconds,
            ..
        } = self
        {
            if let Ok(value) = HeaderValue::from_str(&retry_after_seconds.to_string()) {
                response.headers_mut().insert("Retry-After", value);
            }
            if let Ok(value) = HeaderValue::from_str(&limit.to_string()) {
                response
                    .headers_mut()
                    .insert("X-Heeczer-Quota-Limit", value);
            }
            if let Ok(value) = HeaderValue::from_str(&remaining.to_string()) {
                response
                    .headers_mut()
                    .insert("X-Heeczer-Quota-Remaining", value);
            }
            if let Ok(value) = HeaderValue::from_str(&reset_after_seconds.to_string()) {
                response
                    .headers_mut()
                    .insert("X-Heeczer-Quota-Reset-After", value);
            }
        }
        response
    }
}

pub type ApiResult<T> = std::result::Result<T, ApiError>;
