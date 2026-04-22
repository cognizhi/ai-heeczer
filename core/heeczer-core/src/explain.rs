//! Explainability trace (PRD §16). Public types are stable across SDKs and
//! versioned alongside [`crate::SCORING_VERSION`].

use crate::confidence::ConfidenceBand;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Per-component BCU contributions before category and context multipliers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BcuBreakdown {
    /// Token component.
    pub tokens: Decimal,
    /// Duration component.
    pub duration: Decimal,
    /// Workflow steps component.
    pub steps: Decimal,
    /// Tool calls component.
    pub tools: Decimal,
    /// Artifacts component (post-cap).
    pub artifacts: Decimal,
    /// Output size component.
    pub output: Decimal,
    /// Review component (0 if `review_required == false`).
    pub review: Decimal,
}

impl BcuBreakdown {
    /// Sum of all components.
    pub fn total(&self) -> Decimal {
        self.tokens
            + self.duration
            + self.steps
            + self.tools
            + self.artifacts
            + self.output
            + self.review
    }
}

/// Multiplier breakdown applied after BCU.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextMultiplierTrace {
    /// Retry multiplier.
    pub retry: Decimal,
    /// Ambiguity (high-temperature) multiplier.
    pub ambiguity: Decimal,
    /// Risk class multiplier.
    pub risk: Decimal,
    /// Human-in-loop multiplier.
    pub human_in_loop: Decimal,
    /// Outcome multiplier.
    pub outcome: Decimal,
}

impl ContextMultiplierTrace {
    /// Combined multiplicative product.
    pub fn product(&self) -> Decimal {
        self.retry * self.ambiguity * self.risk * self.human_in_loop * self.outcome
    }
}

/// Tier-resolution trace block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TierTrace {
    /// Tier identifier used.
    pub id: String,
    /// Tier productivity multiplier.
    pub multiplier: Decimal,
    /// Hourly rate used for FEC.
    pub hourly_rate: Decimal,
    /// Currency code from the tier set.
    pub currency: String,
}

/// Full scoring result. Public, stable, versioned.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreResult {
    /// Engine version that produced this result.
    pub scoring_version: String,
    /// Schema spec version of the input event.
    pub spec_version: String,
    /// Profile id used.
    pub scoring_profile: String,
    /// Per-component BCU breakdown.
    pub bcu_breakdown: BcuBreakdown,
    /// Resolved category (post-normalization).
    pub category: String,
    /// Category multiplier applied.
    pub category_multiplier: Decimal,
    /// Context multiplier trace.
    pub context_multiplier: ContextMultiplierTrace,
    /// Pre-tier baseline minutes (BCU * category * context).
    pub baseline_human_minutes: Decimal,
    /// Tier block.
    pub tier: TierTrace,
    /// Final estimated minutes after tier adjustment (rounded).
    pub final_estimated_minutes: Decimal,
    /// Final estimated hours (rounded).
    pub estimated_hours: Decimal,
    /// Final estimated days (rounded; uses `working_hours_per_day`).
    pub estimated_days: Decimal,
    /// Financial equivalent cost (rounded).
    pub financial_equivalent_cost: Decimal,
    /// Confidence score (rounded).
    pub confidence_score: Decimal,
    /// Confidence band (derived from the **unrounded** score per PRD §15.1).
    pub confidence_band: ConfidenceBand,
    /// Human-readable single-line summary.
    pub human_summary: String,
}
