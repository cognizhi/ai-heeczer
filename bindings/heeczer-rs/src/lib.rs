//! `heeczer` — idiomatic Rust SDK for ai-heeczer (plan 0008).
//!
//! Two modes:
//! - **native** (default): in-process scoring via `heeczer-core`. Zero network hop.
//! - **http** (feature flag): async HTTP client targeting the ingestion service.
//!
//! # Native (in-process) usage
//! ```no_run
//! use heeczer::{Client, IngestInput};
//!
//! # fn main() -> heeczer::Result<()> {
//! # let my_event: heeczer::Event = unimplemented!();
//! let client = Client::native();
//! let result = client.score_event(IngestInput {
//!     workspace_id: "ws_default".into(),
//!     event: my_event,
//!     profile: None,
//!     tier_set: None,
//!     tier_override: None,
//! })?;
//! println!("{}", result.human_summary);
//! # Ok(())
//! # }
//! ```

pub use heeczer_core::{
    confidence::ConfidenceBand, error::Error as CoreError, event::Event, explain::ScoreResult,
    profile::ScoringProfile, tier::TierSet,
};

mod error;
mod native;

#[cfg(feature = "http")]
pub mod http;

pub use error::{Error, Result};
pub use native::Client;

/// Input for a single scoring operation.
#[derive(Debug, Clone)]
pub struct IngestInput {
    /// Tenant workspace identifier.
    pub workspace_id: String,
    /// The canonical event to score.
    pub event: Event,
    /// Optional scoring profile override; `None` uses [`ScoringProfile::default_v1`].
    pub profile: Option<ScoringProfile>,
    /// Optional tier set override; `None` uses [`TierSet::default_v1`].
    pub tier_set: Option<TierSet>,
    /// Optional tier identifier override (bypasses event identity resolution).
    pub tier_override: Option<String>,
}
