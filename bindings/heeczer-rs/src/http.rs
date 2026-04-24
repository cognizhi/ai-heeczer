//! HTTP client transport for the ai-heeczer Rust SDK.
//!
//! This module is gated behind the `http` feature flag. It provides an async
//! HTTP client that speaks the `envelope_version=1` contract with the
//! ai-heeczer ingestion service (ADR-0011).

use crate::error::Error;
use heeczer_core::event::Event;
use heeczer_core::explain::ScoreResult;
use reqwest::Client as ReqwestClient;
use serde::{Deserialize, Serialize};

/// Async HTTP client for the ai-heeczer ingestion service.
///
/// # Example
/// ```rust,no_run
/// # #[tokio::main]
/// # async fn main() -> Result<(), heeczer::Error> {
/// let client = heeczer::http::Client::new("https://ingest.example.com", "my-api-key");
/// # let event: heeczer_core::event::Event = unimplemented!();
/// let result = client.score_event("ws_default", &event).await?;
/// println!("{}", result.human_summary);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Client {
    base_url: String,
    api_key: String,
    inner: ReqwestClient,
}

/// Body for `POST /v1/events`.
#[derive(Serialize)]
struct IngestBody<'a> {
    workspace_id: &'a str,
    event: &'a Event,
}

/// Partial structure of the envelope response from `POST /v1/events`.
#[derive(Deserialize)]
struct IngestEnvelope {
    ok: bool,
    #[serde(default)]
    score: Option<ScoreResult>,
    #[serde(default)]
    error: Option<ApiErrorPayload>,
}

#[derive(Deserialize)]
struct ApiErrorPayload {
    kind: String,
    message: String,
}

impl Client {
    /// Create a new HTTP client.
    ///
    /// # Arguments
    /// * `base_url` – Base URL of the ingestion service (trailing slash is stripped).
    /// * `api_key`  – API key sent as `x-heeczer-api-key`.
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            api_key: api_key.into(),
            inner: ReqwestClient::new(),
        }
    }

    /// Score an event via the ingestion service HTTP API.
    ///
    /// Posts to `POST /v1/events` and returns the `ScoreResult` from the
    /// response envelope.
    ///
    /// # Errors
    /// Returns [`Error::Http`] on transport or non-OK envelope errors.
    pub async fn score_event(
        &self,
        workspace_id: &str,
        event: &Event,
    ) -> Result<ScoreResult, Error> {
        let url = format!("{}/v1/events", self.base_url);
        let body = IngestBody {
            workspace_id,
            event,
        };

        let resp = self
            .inner
            .post(&url)
            .header("x-heeczer-api-key", &self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Http(e.to_string()))?;

        let status = resp.status();
        let envelope: IngestEnvelope = resp
            .json()
            .await
            .map_err(|e| Error::Http(format!("failed to decode response: {e}")))?;

        if envelope.ok {
            envelope
                .score
                .ok_or_else(|| Error::Http("envelope ok=true but missing score".to_owned()))
        } else {
            let msg = envelope
                .error
                .map(|e| format!("{}: {}", e.kind, e.message))
                .unwrap_or_else(|| format!("HTTP {status}"));
            Err(Error::Http(msg))
        }
    }
}
