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

/// Contract: every fixture under `core/schema/fixtures/events/valid/` must
/// not only schema-validate (covered in `schema_validation.rs`) but also
/// score cleanly under the embedded default profile + tier set. Adding a
/// new use-case fixture that fails to score should fail this test.
#[test]
fn every_valid_fixture_scores_under_default_profile() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dir = manifest.join("../schema/fixtures/events/valid");
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
        .collect();
    entries.sort();
    assert!(
        !entries.is_empty(),
        "valid/ must contain at least the PRD canonical fixture"
    );

    let profile = ScoringProfile::default_v1();
    let tiers = TierSet::default_v1();
    for path in entries {
        let body = std::fs::read_to_string(&path).unwrap();
        let event: Event = serde_json::from_str(&body)
            .unwrap_or_else(|e| panic!("deserialize {}: {e}", path.display()));
        let r = score(&event, &profile, &tiers, None)
            .unwrap_or_else(|e| panic!("score {} failed: {e:?}", path.display()));
        assert_eq!(r.scoring_version, "1.0.0");
        assert_eq!(r.spec_version, "1.0");
        assert!(
            r.confidence_score >= dec("0") && r.confidence_score <= dec("1"),
            "confidence out of [0,1] for {}: {}",
            path.display(),
            r.confidence_score
        );
        assert!(
            r.final_estimated_minutes >= dec("0"),
            "negative minutes for {}: {}",
            path.display(),
            r.final_estimated_minutes
        );
    }
}

#[test]
fn minimum_payload_scores_without_error() {
    let event = fixture("valid/08-minimum-payload.json");
    let profile = ScoringProfile::default_v1();
    let tiers = TierSet::default_v1();
    let r = score(&event, &profile, &tiers, None).unwrap();
    // Minimum payload: no category → uncategorized multiplier 1.0, confidence penalty applied.
    // We only assert invariants, not exact values, since defaults may evolve.
    assert!(r.confidence_score >= dec("0.0"));
    assert!(r.confidence_score <= dec("1.0"));
    assert!(r.final_estimated_minutes > dec("0"));
    assert!(!r.scoring_version.is_empty());
}

#[test]
fn failure_outcome_does_not_panic() {
    let event = fixture("valid/09-outcome-failure.json");
    let profile = ScoringProfile::default_v1();
    let tiers = TierSet::default_v1();
    let r = score(&event, &profile, &tiers, None).unwrap();
    assert!(r.final_estimated_minutes > dec("0"));
}

#[test]
fn partial_success_outcome_scores_without_error() {
    let event = fixture("valid/10-outcome-partial.json");
    let profile = ScoringProfile::default_v1();
    let tiers = TierSet::default_v1();
    let r = score(&event, &profile, &tiers, None).unwrap();
    assert!(r.final_estimated_minutes > dec("0"));
}
