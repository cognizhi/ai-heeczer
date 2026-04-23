//! Tier (role) definitions (PRD §17).

use crate::error::{Error, Result};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A single human role tier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tier {
    /// Tier identifier (`tier_mid_eng`, ...).
    pub tier_id: String,
    /// Human-readable name.
    pub display_name: String,
    /// Productivity multiplier; AI minutes / multiplier = human-equivalent minutes.
    pub productivity_multiplier: Decimal,
    /// Hourly rate in `currency`.
    pub hourly_rate: Decimal,
    /// Working hours per day, used to convert minutes to days.
    pub working_hours_per_day: Decimal,
}

/// A versioned set of [`Tier`]s.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TierSet {
    /// Tier set identifier.
    pub tier_set_id: String,
    /// Tier set version.
    pub version: String,
    /// When the tier set takes effect.
    pub effective_at: String,
    /// ISO 4217 currency code (`USD`, ...).
    pub currency: String,
    /// All tiers in the set.
    pub tiers: Vec<Tier>,
}

impl TierSet {
    /// Embedded default v1 tier set.
    pub fn default_v1() -> Self {
        const DEFAULT: &str = include_str!("../schema/tiers/default.v1.json");
        serde_json::from_str(DEFAULT).expect("embedded default tier set must parse")
    }

    /// Look up a tier by id.
    pub fn get(&self, tier_id: &str) -> Result<&Tier> {
        self.tiers
            .iter()
            .find(|t| t.tier_id == tier_id)
            .ok_or_else(|| Error::UnknownTier(tier_id.to_string()))
    }

    /// Resolve the tier to use given an event identity and a fallback id.
    pub fn resolve<'a>(&'a self, requested: Option<&str>, fallback: &str) -> Result<&'a Tier> {
        self.get(requested.unwrap_or(fallback))
    }
}
