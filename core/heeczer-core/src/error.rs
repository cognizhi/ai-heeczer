//! Typed errors. All public APIs return [`crate::Result`].

use thiserror::Error;

/// Errors raised by validation, normalization, and scoring.
#[derive(Debug, Error)]
pub enum Error {
    /// Schema validation failed; carries the JSON Pointer path of the first
    /// failing instance and a human-readable message.
    #[error("schema validation failed at `{path}`: {message}")]
    Schema {
        /// JSON Pointer (RFC 6901) of the failing field.
        path: String,
        /// Human-readable failure description.
        message: String,
    },

    /// Required non-derivable field missing per PRD §14.2.1.
    #[error("required field missing: {0}")]
    MissingRequired(&'static str),

    /// Unknown enum value supplied where a closed enum is expected.
    #[error("unknown enum value `{value}` for field `{field}`")]
    UnknownEnum {
        /// The invalid input value.
        value: String,
        /// The field that received the invalid value.
        field: &'static str,
    },

    /// Tier referenced in event or argument does not exist in the supplied [`crate::TierSet`].
    #[error("unknown tier_id: {0}")]
    UnknownTier(String),

    /// Decimal arithmetic overflowed the supported range.
    #[error("arithmetic overflow during scoring")]
    Overflow,

    /// JSON (de)serialization failure; preserved verbatim for diagnostics.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Crate-wide [`Result`] alias.
pub type Result<T> = std::result::Result<T, Error>;
