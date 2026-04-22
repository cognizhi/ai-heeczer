//! HTTP client transport for the ai-heeczer Rust SDK.
//!
//! This module is gated behind the `http` feature flag. It provides an async
//! HTTP client that speaks the `envelope_version=1` contract with the
//! ai-heeczer ingestion service (ADR-0011).
//!
//! **Status:** not yet implemented. The `http` feature compiles but all entry
//! points return `Error::Http("not implemented")` until plan 0008 §HTTP
//! transport is complete.

use crate::error::Error;
use heeczer_core::event::Event;
use heeczer_core::explain::ScoreResult;

/// Async HTTP client for the ai-heeczer ingestion service.
///
/// # Example
/// ```rust,no_run
/// # #[tokio::main]
/// # async fn main() -> Result<(), heeczer::Error> {
/// let client = heeczer::http::Client::new("https://ingest.example.com", "my-api-key");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Client {
    base_url: String,
    api_key: String,
}

impl Client {
    /// Create a new HTTP client.
    ///
    /// # Arguments
    /// * `base_url` – Base URL of the ingestion service.
    /// * `api_key`  – API key for authentication.
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: api_key.into(),
        }
    }

    /// Score an event via the ingestion service HTTP API.
    ///
    /// **Not yet implemented.** Returns `Err(Error::Http("not implemented"))`.
    pub async fn score_event(&self, _event: &Event) -> Result<ScoreResult, Error> {
        Err(Error::Http("http transport not yet implemented".to_owned()))
    }
}
