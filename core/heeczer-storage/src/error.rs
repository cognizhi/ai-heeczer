//! Storage errors.

use thiserror::Error;

/// Errors raised by the storage layer.
#[derive(Debug, Error)]
pub enum Error {
    /// Underlying sqlx error.
    #[error(transparent)]
    Sqlx(#[from] sqlx_core::error::Error),

    /// Migration error.
    #[error(transparent)]
    Migrate(#[from] sqlx_core::migrate::MigrateError),

    /// Append-only invariant was violated by a caller.
    #[error("append-only invariant violated: {0}")]
    AppendOnly(&'static str),

    /// Conflicting payload supplied for an existing `event_id` (PRD §19.4).
    #[error("event_id `{0}` already exists with a different normalized payload")]
    Conflict(String),

    /// Serialization failure.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

/// Crate-wide result alias.
pub type Result<T> = std::result::Result<T, Error>;
