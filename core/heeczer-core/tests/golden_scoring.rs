//! Golden fixture tests asserting exact persisted decimal outputs (PRD §14.7).
//!
//! Hand-computed against PRD §14 formulas. Bumping any of these requires
//! incrementing `SCORING_VERSION` and an ADR-0003 amendment.

use heeczer_core::{score, Event, ScoringProfile, TierSet};
use rust_decimal::Decimal;
use std::path::PathBuf;
use std::str::FromStr;

fn dec(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

fn fixture(name: &str) -> Event {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let p = manifest.join("../schema/fixtures/events").join(name);
    serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap()
}

#[test]
fn prd_canonical_event_produces_expected_score() {
    let event = fixture("valid/01-prd-canonical.json");
    let profile = ScoringProfile::default_v1();
    let tiers = TierSet::default_v1();
    let r = score(&event, &profile, &tiers, None).unwrap();

    // BCU breakdown (PRD §14.2 with default profile).
    assert_eq!(r.bcu_breakdown.tokens, dec("10.4")); // 5200 / 500
    assert_eq!(r.bcu_breakdown.duration, dec("7.25")); // 14.5 / 2
    assert_eq!(r.bcu_breakdown.steps, dec("10")); // 5 * 2
    assert_eq!(r.bcu_breakdown.tools, dec("9")); // 3 * 3
    assert_eq!(r.bcu_breakdown.artifacts, dec("6.0")); // min(4,20) * 1.5
    assert_eq!(r.bcu_breakdown.output, dec("3.00")); // 2.5 * 1.2
    assert_eq!(r.bcu_breakdown.review, dec("5")); // review_required, code_gen review_weight

    // Multipliers.
    assert_eq!(r.category_multiplier, dec("1.2"));
    assert_eq!(r.context_multiplier.retry, dec("1.25"));
    assert_eq!(r.context_multiplier.ambiguity, dec("1.0"));
    assert_eq!(r.context_multiplier.risk, dec("1.0"));
    assert_eq!(r.context_multiplier.human_in_loop, dec("1.0"));
    assert_eq!(r.context_multiplier.outcome, dec("1.0"));

    // Final outputs (rounded per default profile).
    assert_eq!(r.final_estimated_minutes, dec("75.98"));
    assert_eq!(r.estimated_hours, dec("1.27"));
    assert_eq!(r.estimated_days, dec("0.16"));
    assert_eq!(r.financial_equivalent_cost, dec("94.97"));
    assert_eq!(r.confidence_score, dec("0.9000"));
    assert_eq!(r.scoring_version, "1.0.0");
    assert_eq!(r.spec_version, "1.0");
    assert_eq!(r.tier.id, "tier_mid_eng");
    assert_eq!(r.tier.currency, "USD");
}

#[test]
fn high_risk_caps_confidence_even_with_full_telemetry() {
    let mut event = fixture("valid/01-prd-canonical.json");
    if let Some(ctx) = event.context.as_mut() {
        ctx.risk_class = Some(heeczer_core::event::RiskClass::High);
    }
    let profile = ScoringProfile::default_v1();
    let tiers = TierSet::default_v1();
    let r = score(&event, &profile, &tiers, None).unwrap();
    // Without retry penalty we'd be 0.95; high_risk_cap is 0.85.
    // With one retry: 0.95 - 0.05 = 0.90, still > 0.85 → cap fires.
    assert_eq!(r.confidence_score, dec("0.8500"));
}

#[test]
fn failure_outcome_collapses_value() {
    let mut event = fixture("valid/01-prd-canonical.json");
    event.task.outcome = heeczer_core::event::Outcome::Failure;
    let profile = ScoringProfile::default_v1();
    let tiers = TierSet::default_v1();
    let r = score(&event, &profile, &tiers, None).unwrap();
    assert_eq!(r.context_multiplier.outcome, dec("0.25"));
    // 50.65 * 1.2 * (1.25 * 0.25) = 50.65 * 1.2 * 0.3125 = 18.99375 → 18.99
    assert_eq!(r.final_estimated_minutes, dec("18.99"));
}

#[test]
fn output_is_byte_stable_when_serialised_to_json() {
    let event = fixture("valid/01-prd-canonical.json");
    let profile = ScoringProfile::default_v1();
    let tiers = TierSet::default_v1();
    let a = serde_json::to_string(&score(&event, &profile, &tiers, None).unwrap()).unwrap();
    let b = serde_json::to_string(&score(&event, &profile, &tiers, None).unwrap()).unwrap();
    assert_eq!(a, b);
}
