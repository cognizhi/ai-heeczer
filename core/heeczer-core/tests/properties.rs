//! Property-based tests for the scoring core (foundation backlog).
//!
//! These tests exercise invariants that any single golden fixture cannot
//! cover: rounding idempotence, scale preservation, score purity (the same
//! input must always yield the same output), and JSON round-trip stability.

use heeczer_core::{score, Event, ScoringProfile, TierSet};
use proptest::prelude::*;

fn canonical_event() -> Event {
    let body = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../schema/fixtures/events/valid/01-prd-canonical.json"),
    )
    .unwrap();
    serde_json::from_str(&body).unwrap()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Score is pure: identical inputs must produce identical outputs.
    #[test]
    fn score_is_deterministic(seed in 0u64..1024) {
        // Seed is unused arithmetically; we just want proptest to call us
        // many times with no shared mutable state.
        let _ = seed;
        let event = canonical_event();
        let profile = ScoringProfile::default_v1();
        let tiers = TierSet::default_v1();
        let a = score(&event, &profile, &tiers, None).unwrap();
        let b = score(&event, &profile, &tiers, None).unwrap();
        prop_assert_eq!(serde_json::to_string(&a).unwrap(), serde_json::to_string(&b).unwrap());
    }

    /// Re-serializing and re-deserializing a ScoreResult JSON yields the
    /// original — guards against asymmetric serde derives.
    #[test]
    fn score_result_json_round_trip(seed in 0u64..1024) {
        let _ = seed;
        let event = canonical_event();
        let r = score(&event, &ScoringProfile::default_v1(), &TierSet::default_v1(), None).unwrap();
        let s = serde_json::to_string(&r).unwrap();
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        let s2 = serde_json::to_string(&v).unwrap();
        // Compare canonicalised JSON values rather than byte-compare strings
        // (numeric formatting via Value preserves order but may renormalise
        // whitespace).
        let v2: serde_json::Value = serde_json::from_str(&s2).unwrap();
        let r_value = serde_json::to_value(&r).unwrap();
        prop_assert_eq!(r_value, v2);
    }

    /// Scaling token counts up by an integer factor scales the token BCU
    /// component by the same integer factor (within fixed-point precision).
    /// This catches accidental float arithmetic creeping in.
    #[test]
    fn token_bcu_is_linear(factor in 1u32..=8) {
        let mut event = canonical_event();
        let prompt = event.metrics.tokens_prompt.unwrap_or(0);
        let completion = event.metrics.tokens_completion.unwrap_or(0);
        if prompt + completion == 0 { return Ok(()); }

        let profile = ScoringProfile::default_v1();
        let tiers = TierSet::default_v1();
        let base = score(&event, &profile, &tiers, None).unwrap();

        event.metrics.tokens_prompt = Some(prompt * u64::from(factor));
        event.metrics.tokens_completion = Some(completion * u64::from(factor));
        let scaled = score(&event, &profile, &tiers, None).unwrap();

        let expected = base.bcu_breakdown.tokens * rust_decimal::Decimal::from(factor);
        prop_assert_eq!(scaled.bcu_breakdown.tokens, expected);
    }

    /// Final outputs must respect the configured decimal places (rounding idempotence).
    #[test]
    fn rounded_outputs_have_at_most_configured_dp(seed in 0u64..1024) {
        let _ = seed;
        let event = canonical_event();
        let profile = ScoringProfile::default_v1();
        let tiers = TierSet::default_v1();
        let r = score(&event, &profile, &tiers, None).unwrap();
        prop_assert!(
            r.final_estimated_minutes.scale() <= profile.rounding.minutes_dp,
            "minutes scale {} > configured {}",
            r.final_estimated_minutes.scale(),
            profile.rounding.minutes_dp
        );
        prop_assert!(
            r.financial_equivalent_cost.scale() <= profile.rounding.fec_dp,
            "fec scale {} > configured {}",
            r.financial_equivalent_cost.scale(),
            profile.rounding.fec_dp
        );
        prop_assert!(
            r.confidence_score.scale() <= profile.rounding.confidence_dp,
            "confidence scale {} > configured {}",
            r.confidence_score.scale(),
            profile.rounding.confidence_dp
        );
    }

    /// Confidence is bounded to [profile.confidence.min, profile.confidence.max].
    #[test]
    fn confidence_is_within_configured_bounds(seed in 0u64..1024) {
        let _ = seed;
        let event = canonical_event();
        let profile = ScoringProfile::default_v1();
        let tiers = TierSet::default_v1();
        let r = score(&event, &profile, &tiers, None).unwrap();
        prop_assert!(r.confidence_score >= profile.confidence.min);
        prop_assert!(r.confidence_score <= profile.confidence.max);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Monotonicity: more tokens must yield a BCU token component >= the base event.
    #[test]
    fn more_tokens_means_higher_or_equal_bcu_tokens(
        tokens_prompt in 100u64..=50_000,
        tokens_completion in 100u64..=50_000,
        multiplier in 2u32..=4,
    ) {
        let mut event = canonical_event();
        event.metrics.tokens_prompt = Some(tokens_prompt);
        event.metrics.tokens_completion = Some(tokens_completion);
        let profile = ScoringProfile::default_v1();
        let tiers = TierSet::default_v1();
        let base = score(&event, &profile, &tiers, None).unwrap();

        event.metrics.tokens_prompt = Some(tokens_prompt * u64::from(multiplier));
        event.metrics.tokens_completion = Some(tokens_completion * u64::from(multiplier));
        let scaled = score(&event, &profile, &tiers, None).unwrap();

        prop_assert!(scaled.bcu_breakdown.tokens >= base.bcu_breakdown.tokens);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Overflow safety: extreme inputs (PRD §29 max payload sizes) must not panic or overflow.
    #[test]
    fn no_overflow_on_extreme_inputs(
        tokens_prompt in 0u64..=2_000_000,
        tokens_completion in 0u64..=2_000_000,
        duration_secs in 0u64..=7_200,
        steps in 0u32..=10_000,
        tool_calls in 0u32..=1_000,
    ) {
        let mut event = canonical_event();
        event.metrics.tokens_prompt = Some(tokens_prompt);
        event.metrics.tokens_completion = Some(tokens_completion);
        event.metrics.duration_ms = duration_secs * 1_000;
        event.metrics.workflow_steps = Some(steps);
        event.metrics.tool_call_count = Some(tool_calls);
        let profile = ScoringProfile::default_v1();
        let tiers = TierSet::default_v1();
        prop_assert!(score(&event, &profile, &tiers, None).is_ok());
    }
}
