//! Structured error type that maps cleanly into HTTP responses.
//!
//! Error bodies follow a closed-shape envelope so SDKs can match on `kind`
//! without fragile string parsing (mirrors the C ABI envelope contract from
//! ADR-0011).

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

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
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("feature disabled: {0}")]
    FeatureDisabled(String),
}

impl ApiError {
    fn kind(&self) -> &'static str {
        match self {
            ApiError::Schema(_) => "schema",
            ApiError::BadRequest(_) => "bad_request",
            ApiError::Scoring(_) => "scoring",
            ApiError::Storage(_) => "storage",
            ApiError::NotFound(_) => "not_found",
            ApiError::Forbidden(_) => "forbidden",
            ApiError::FeatureDisabled(_) => "feature_disabled",
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            ApiError::Schema(_) | ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Scoring(_) | ApiError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::FeatureDisabled(_) => StatusCode::SERVICE_UNAVAILABLE,
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
        let body = ErrorBody {
            ok: false,
            envelope_version: "1",
            error: ErrorPayload {
                kind: self.kind(),
                message: self.to_string(),
            },
        };
        (self.status(), Json(body)).into_response()
    }
}

pub type ApiResult<T> = std::result::Result<T, ApiError>;
