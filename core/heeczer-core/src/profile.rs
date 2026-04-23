//! Scoring profile (PRD §14, ADR-0003). All numeric fields are stringly typed
//! `Decimal` to guarantee fixed-point arithmetic across SDKs.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A complete scoring profile. Append-only and versioned (ADR-0003).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScoringProfile {
    /// Profile identifier.
    pub profile_id: String,
    /// Profile semver.
    pub version: String,
    /// Pinned scoring engine version this profile is compatible with.
    pub scoring_version: String,
    /// When this profile takes effect (RFC 3339).
    pub effective_at: String,
    /// When this profile was superseded (the only mutable field).
    #[serde(default)]
    pub superseded_at: Option<String>,
    /// Numeric component weights.
    pub components: Components,
    /// Per-category scoring multipliers; must contain `uncategorized`.
    pub category_multipliers: BTreeMap<String, Decimal>,
    /// Per-category output weights (used by `output_component`).
    #[serde(default)]
    pub category_output_weights: BTreeMap<String, Decimal>,
    /// Per-category review weights (used by `review_component`).
    #[serde(default)]
    pub category_review_weights: BTreeMap<String, Decimal>,
    /// Context multiplier configuration.
    pub context_multipliers: ContextMultipliers,
    /// Confidence model parameters.
    pub confidence: ConfidenceParams,
    /// Rounding configuration for persisted output.
    pub rounding: Rounding,
}

/// Numeric component weights driving BCU computation (PRD §14.2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Components {
    /// Divisor for `total_tokens`.
    pub token_divisor: Decimal,
    /// Divisor for duration in seconds.
    pub duration_seconds_divisor: Decimal,
    /// Per-step BCU contribution.
    pub step_weight: Decimal,
    /// Per-tool-call BCU contribution.
    pub tool_weight: Decimal,
    /// Per-artifact BCU contribution (after cap).
    pub artifact_weight: Decimal,
    /// Maximum artifact count counted.
    pub artifact_cap: u32,
    /// Default output weight when category is unknown.
    #[serde(default = "Components::default_output_weight")]
    pub output_default_weight: Decimal,
    /// Default review weight when category is unknown.
    pub review_weight: Decimal,
}

impl Components {
    fn default_output_weight() -> Decimal {
        rust_decimal_macros::dec!(1)
    }
}

/// Context multiplier configuration (PRD §14.4).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextMultipliers {
    /// `1 + retries * retry_per_unit`, capped at `retry_cap`.
    pub retry_per_unit: Decimal,
    /// Maximum value of the retry multiplier.
    pub retry_cap: Decimal,
    /// Multiplier applied when `temperature > ambiguity_temp_threshold`.
    pub ambiguity_high_temp: Decimal,
    /// Threshold above which ambiguity multiplier kicks in.
    pub ambiguity_temp_threshold: Decimal,
    /// Multiplier when `risk_class == high`.
    pub risk_high: Decimal,
    /// Multiplier when `risk_class == medium`.
    pub risk_medium: Decimal,
    /// Multiplier when `risk_class == low`.
    pub risk_low: Decimal,
    /// Multiplier applied when `human_in_loop == true`.
    pub human_in_loop: Decimal,
    /// Per-outcome multipliers.
    pub outcome: OutcomeMultipliers,
}

/// Outcome-specific multipliers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutcomeMultipliers {
    /// Multiplier for `success`.
    pub success: Decimal,
    /// Multiplier for `partial_success`.
    pub partial_success: Decimal,
    /// Multiplier for `failure`.
    pub failure: Decimal,
    /// Multiplier for `timeout`.
    pub timeout: Decimal,
}

/// Confidence model parameters (PRD §15).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfidenceParams {
    /// Starting confidence with full telemetry.
    pub base: Decimal,
    /// Penalty when category is missing.
    pub missing_category_penalty: Decimal,
    /// Penalty when token metrics are missing.
    pub missing_tokens_penalty: Decimal,
    /// Penalty when workflow steps are missing.
    pub missing_steps_penalty: Decimal,
    /// Penalty when tool call count is missing.
    pub missing_tools_penalty: Decimal,
    /// Per-retry confidence penalty.
    pub retry_penalty_per_unit: Decimal,
    /// Cap on cumulative retry confidence penalty.
    pub retry_penalty_cap: Decimal,
    /// Maximum confidence permitted for high-risk tasks.
    pub high_risk_cap: Decimal,
    /// Lower bound (0.0).
    pub min: Decimal,
    /// Upper bound (1.0).
    pub max: Decimal,
}

/// Rounding configuration (PRD §14.2.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rounding {
    /// Decimal places for minutes outputs.
    pub minutes_dp: u32,
    /// Decimal places for hours.
    pub hours_dp: u32,
    /// Decimal places for days.
    pub days_dp: u32,
    /// Decimal places for FEC.
    pub fec_dp: u32,
    /// Decimal places for confidence.
    pub confidence_dp: u32,
}

impl ScoringProfile {
    /// Embedded default v1 profile (`core/schema/profiles/default.v1.json`).
    pub fn default_v1() -> Self {
        const DEFAULT_PROFILE: &str = include_str!("../schema/profiles/default.v1.json");
        serde_json::from_str(DEFAULT_PROFILE).expect("embedded default profile must parse")
    }
}
