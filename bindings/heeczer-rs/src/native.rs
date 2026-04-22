//! Native (in-process) client using `heeczer-core` directly.

use crate::error::Result;
use crate::IngestInput;
use heeczer_core::{profile::ScoringProfile, scoring::score, tier::TierSet, explain::ScoreResult};

/// Synchronous in-process scoring client.
///
/// All calls are CPU-bound and complete without any network I/O.
/// Thread-safe: [`Client`] is `Send + Sync` and can be shared via `Arc`.
#[derive(Debug, Default, Clone)]
pub struct Client;

impl Client {
    /// Construct a new native client.
    pub fn native() -> Self {
        Self
    }

    /// Score a single event in-process and return the full [`ScoreResult`].
    pub fn score_event(&self, input: IngestInput) -> Result<ScoreResult> {
        let profile = input.profile.unwrap_or_else(ScoringProfile::default_v1);
        let tiers = input.tier_set.unwrap_or_else(TierSet::default_v1);
        let tier_override = input.tier_override.as_deref();
        let result = score(&input.event, &profile, &tiers, tier_override)?;
        Ok(result)
    }
}
