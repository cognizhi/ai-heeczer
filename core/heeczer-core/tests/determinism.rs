//! Determinism and version constants.

use heeczer_core::{score, Event, ScoringProfile, TierSet, SCORING_VERSION, SPEC_VERSION};
use std::path::PathBuf;

fn fixture(name: &str) -> Event {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let p = manifest.join("../schema/fixtures/events").join(name);
    let body = std::fs::read_to_string(&p).unwrap();
    serde_json::from_str(&body).unwrap()
}

#[test]
fn version_constants_are_pinned() {
    // Bumping these requires fixture diffs and ADR amendments.
    assert_eq!(SPEC_VERSION, "1.0");
    assert_eq!(SCORING_VERSION, "1.0.0");
}

#[test]
fn scoring_is_deterministic_across_runs() {
    let event = fixture("valid/01-prd-canonical.json");
    let profile = ScoringProfile::default_v1();
    let tiers = TierSet::default_v1();

    let a = score(&event, &profile, &tiers, None).unwrap();
    let b = score(&event, &profile, &tiers, None).unwrap();
    let c = score(&event, &profile, &tiers, None).unwrap();
    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn scoring_handles_minimum_required_event() {
    let event = fixture("edge/01-minimum-required.json");
    let profile = ScoringProfile::default_v1();
    let tiers = TierSet::default_v1();
    let r = score(&event, &profile, &tiers, None).unwrap();
    // No telemetry => baseline minutes is 0; final FEC is 0; confidence is reduced.
    assert!(r.financial_equivalent_cost.is_zero());
    assert!(r.final_estimated_minutes.is_zero());
    assert!(r.confidence_score < r.tier.multiplier); // sanity: dropped from base
}

#[test]
fn missing_category_normalizes_to_uncategorized_with_penalty() {
    let event = fixture("edge/02-missing-category.json");
    let profile = ScoringProfile::default_v1();
    let tiers = TierSet::default_v1();
    let r = score(&event, &profile, &tiers, None).unwrap();
    assert_eq!(r.category, "uncategorized");
    // base 0.95 - missing_category 0.10 - missing_steps 0.05 - missing_tools 0.05 = 0.75
    assert_eq!(r.confidence_score.to_string(), "0.7500");
}
