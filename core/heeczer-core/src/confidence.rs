//! Confidence model (PRD §15).

use crate::normalize::Normalized;
use crate::profile::ConfidenceParams;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Confidence band derived from the unrounded confidence score (PRD §15).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidenceBand {
    /// 0.85 – 1.00
    High,
    /// 0.60 – 0.84
    Medium,
    /// 0.40 – 0.59
    Low,
    /// below 0.40
    VeryLow,
}

impl ConfidenceBand {
    /// Map an unrounded score to its band.
    pub fn from_score(score: Decimal) -> Self {
        use rust_decimal_macros::dec;
        if score >= dec!(0.85) {
            Self::High
        } else if score >= dec!(0.60) {
            Self::Medium
        } else if score >= dec!(0.40) {
            Self::Low
        } else {
            Self::VeryLow
        }
    }
}

/// Compute the unrounded confidence score for a normalized event.
pub fn compute(norm: &Normalized<'_>, params: &ConfidenceParams) -> Decimal {
    use crate::event::RiskClass;

    let mut score = params.base;

    if norm.category_was_missing {
        score -= params.missing_category_penalty;
    }
    if norm.tokens_were_missing {
        score -= params.missing_tokens_penalty;
    }
    if norm.steps_were_missing {
        score -= params.missing_steps_penalty;
    }
    if norm.tools_were_missing {
        score -= params.missing_tools_penalty;
    }

    // Retry penalty (capped).
    let retry_pen = (norm.retries * params.retry_penalty_per_unit).min(params.retry_penalty_cap);
    score -= retry_pen;

    // Cap for high-risk tasks (only if no other floor pushes us lower).
    if norm.risk_class == RiskClass::High && score > params.high_risk_cap {
        score = params.high_risk_cap;
    }

    // Clamp to [min, max].
    score.clamp(params.min, params.max)
}
