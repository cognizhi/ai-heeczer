//! Scoring orchestrator. Composes [`crate::normalize`], [`crate::confidence`],
//! and the formulas in PRD §14.2–§14.6 into a deterministic [`ScoreResult`].

use crate::confidence::{self, ConfidenceBand};
use crate::error::Result;
use crate::event::{Event, RiskClass};
use crate::explain::{BcuBreakdown, ContextMultiplierTrace, ScoreResult, TierTrace};
use crate::normalize::{self, Normalized};
use crate::profile::{ContextMultipliers, ScoringProfile};
use crate::tier::TierSet;
use crate::version::{SCORING_VERSION, SPEC_VERSION};
use rust_decimal::{Decimal, RoundingStrategy};
use rust_decimal_macros::dec;

/// Score an event against a profile and tier set.
///
/// `tier_override` lets callers (CLI, tests, ingestion service) force a tier
/// regardless of what the event identity declares. When `None`, the resolution
/// order is: `event.identity.tier_id` → `tier_mid_eng` (mid-level fallback).
pub fn score(
    event: &Event,
    profile: &ScoringProfile,
    tiers: &TierSet,
    tier_override: Option<&str>,
) -> Result<ScoreResult> {
    let norm = normalize::normalize(event);
    let breakdown = compute_breakdown(&norm, profile);
    let category_mult = lookup_category_multiplier(&norm, profile);
    let context_trace = compute_context_multiplier(&norm, &profile.context_multipliers);

    let bcu_total = breakdown.total();
    let baseline_minutes = bcu_total * category_mult * context_trace.product();

    let requested_tier =
        tier_override.or(event.identity.as_ref().and_then(|i| i.tier_id.as_deref()));
    let tier = tiers.resolve(requested_tier, "tier_mid_eng")?;

    // Avoid divide-by-zero for malformed profiles; treat 0 as 1.
    let productivity = if tier.productivity_multiplier.is_zero() {
        Decimal::ONE
    } else {
        tier.productivity_multiplier
    };
    let raw_minutes = baseline_minutes / productivity;
    let raw_hours = raw_minutes / dec!(60);
    let working_hours = if tier.working_hours_per_day.is_zero() {
        dec!(8)
    } else {
        tier.working_hours_per_day
    };
    let raw_days = raw_hours / working_hours;
    let raw_fec = raw_hours * tier.hourly_rate;

    let raw_confidence = confidence::compute(&norm, &profile.confidence);
    let band = ConfidenceBand::from_score(raw_confidence);

    let r = &profile.rounding;
    let final_minutes = round(raw_minutes, r.minutes_dp);
    let final_hours = round(raw_hours, r.hours_dp);
    let final_days = round(raw_days, r.days_dp);
    let final_fec = round(raw_fec, r.fec_dp);
    let final_confidence = round(raw_confidence, r.confidence_dp);

    let category_owned = norm.category.to_string();
    let summary = build_summary(
        &category_owned,
        final_minutes,
        final_fec,
        &tier.display_name,
        band,
    );

    Ok(ScoreResult {
        scoring_version: SCORING_VERSION.to_string(),
        spec_version: SPEC_VERSION.to_string(),
        scoring_profile: profile.profile_id.clone(),
        bcu_breakdown: breakdown,
        category: category_owned,
        category_multiplier: category_mult,
        context_multiplier: context_trace,
        baseline_human_minutes: round(baseline_minutes, r.minutes_dp),
        tier: TierTrace {
            id: tier.tier_id.clone(),
            multiplier: tier.productivity_multiplier,
            hourly_rate: tier.hourly_rate,
            currency: tiers.currency.clone(),
        },
        final_estimated_minutes: final_minutes,
        estimated_hours: final_hours,
        estimated_days: final_days,
        financial_equivalent_cost: final_fec,
        confidence_score: final_confidence,
        confidence_band: band,
        human_summary: summary,
    })
}

fn compute_breakdown(n: &Normalized<'_>, p: &ScoringProfile) -> BcuBreakdown {
    let c = &p.components;

    let tokens = n.total_tokens / c.token_divisor;
    let duration = n.duration_seconds / c.duration_seconds_divisor;
    let steps = n.workflow_steps * c.step_weight;
    let tools = n.tool_call_count * c.tool_weight;

    let capped_artifacts = n.artifact_count.min(Decimal::from(c.artifact_cap));
    let artifacts = capped_artifacts * c.artifact_weight;

    let output_weight = p
        .category_output_weights
        .get(n.category)
        .copied()
        .unwrap_or(c.output_default_weight);
    let output = n.output_size_proxy * output_weight;

    let review_weight = p
        .category_review_weights
        .get(n.category)
        .copied()
        .unwrap_or(c.review_weight);
    let review = if n.review_required {
        review_weight
    } else {
        Decimal::ZERO
    };

    BcuBreakdown {
        tokens,
        duration,
        steps,
        tools,
        artifacts,
        output,
        review,
    }
}

fn lookup_category_multiplier(n: &Normalized<'_>, p: &ScoringProfile) -> Decimal {
    p.category_multipliers
        .get(n.category)
        .copied()
        .or_else(|| {
            p.category_multipliers
                .get(normalize::UNCATEGORIZED)
                .copied()
        })
        .unwrap_or(Decimal::ONE)
}

fn compute_context_multiplier(
    n: &Normalized<'_>,
    cm: &ContextMultipliers,
) -> ContextMultiplierTrace {
    use crate::event::Outcome;

    let retry = (Decimal::ONE + n.retries * cm.retry_per_unit).min(cm.retry_cap);

    let ambiguity = if n.temperature > cm.ambiguity_temp_threshold {
        cm.ambiguity_high_temp
    } else {
        Decimal::ONE
    };

    let risk = match n.risk_class {
        RiskClass::High => cm.risk_high,
        RiskClass::Medium => cm.risk_medium,
        RiskClass::Low => cm.risk_low,
    };

    let human_in_loop = if n.human_in_loop {
        cm.human_in_loop
    } else {
        Decimal::ONE
    };

    let outcome = match n.outcome {
        Outcome::Success => cm.outcome.success,
        Outcome::PartialSuccess => cm.outcome.partial_success,
        Outcome::Failure => cm.outcome.failure,
        Outcome::Timeout => cm.outcome.timeout,
    };

    ContextMultiplierTrace {
        retry,
        ambiguity,
        risk,
        human_in_loop,
        outcome,
    }
}

/// Round half away from zero to `dp` decimal places (PRD §14.2.1) and rescale
/// so the persisted string representation always carries exactly `dp` digits.
fn round(v: Decimal, dp: u32) -> Decimal {
    let mut out = v.round_dp_with_strategy(dp, RoundingStrategy::MidpointAwayFromZero);
    out.rescale(dp);
    out
}

fn build_summary(
    category: &str,
    minutes: Decimal,
    fec: Decimal,
    tier_name: &str,
    band: ConfidenceBand,
) -> String {
    let band_str = match band {
        ConfidenceBand::High => "high",
        ConfidenceBand::Medium => "medium",
        ConfidenceBand::Low => "low",
        ConfidenceBand::VeryLow => "very low",
    };
    format!(
        "Estimated {minutes} {tier_name}-equivalent minutes (~{fec} cost) for `{category}`; confidence {band_str}."
    )
}
