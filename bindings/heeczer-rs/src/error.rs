//! SDK error type.

use thiserror::Error;

/// SDK-level error, wrapping core or transport failures.
#[derive(Debug, Error)]
pub enum Error {
    /// Wraps any error from `heeczer-core`.
    #[error("scoring: {0}")]
    Core(#[from] heeczer_core::error::Error),

    /// HTTP transport error (feature `http` only).
    #[error("http: {0}")]
    Http(String),

    /// JSON (de)serialisation error.
    #[error("serde: {0}")]
    Serde(#[from] serde_json::Error),
}

/// Convenience alias.
pub type Result<T, E = Error> = std::result::Result<T, E>;
