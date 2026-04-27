use heeczer_core::{
    build_suggested_profile, run_calibration, BenchmarkPack, ScoringProfile, TierSet,
};
use rust_decimal_macros::dec;

#[test]
fn reference_pack_calibration_report_is_stable() {
    let report = run_calibration(
        &BenchmarkPack::reference_v1(),
        &ScoringProfile::default_v1(),
        &TierSet::default_v1(),
    )
    .expect("reference pack calibration");

    assert_eq!(report.pack_id, "reference-pack");
    assert_eq!(report.profile_id, "default");
    assert_eq!(report.suggested_profile_version, "1.0.1");
    assert_eq!(report.summary.total_items, 5);
    assert_eq!(report.summary.within_expected_range, 3);
    assert_eq!(report.summary.confidence_band_matches, 2);
    assert_eq!(report.summary.rmse_minutes, dec!(53.9463));
    assert_eq!(report.summary.mae_range_minutes, dec!(9.42));
    assert_eq!(report.summary.mae_midpoint_minutes, dec!(41.65));
    assert_eq!(report.summary.bias_minutes, dec!(23.41));
    assert_eq!(report.summary.r_squared, dec!(-0.0957));

    let analysis = report
        .suggested_category_multipliers
        .iter()
        .find(|suggestion| suggestion.category == "analysis")
        .expect("analysis suggestion");
    assert!(analysis.adds_new_category);
    assert_eq!(analysis.current_multiplier, dec!(1.0000));
    assert_eq!(analysis.calibration_factor, dec!(0.5447));
    assert_eq!(analysis.suggested_multiplier, dec!(0.5447));
}

#[test]
fn calibration_suggestions_build_a_new_profile_version() {
    let profile = ScoringProfile::default_v1();
    let report = run_calibration(
        &BenchmarkPack::reference_v1(),
        &profile,
        &TierSet::default_v1(),
    )
    .expect("reference pack calibration");

    let suggested = build_suggested_profile(
        &profile,
        &report.suggested_category_multipliers,
        "2026-04-27T00:00:00Z",
    );

    assert_eq!(suggested.profile_id, "default");
    assert_eq!(suggested.version, "1.0.1");
    assert_eq!(suggested.effective_at, "2026-04-27T00:00:00Z");
    assert!(suggested.superseded_at.is_none());
    assert_eq!(
        suggested.category_multipliers.get("analysis"),
        Some(&dec!(0.5447))
    );
    assert_eq!(
        suggested.category_multipliers.get("summarization"),
        Some(&dec!(0.7499))
    );
    assert_eq!(
        suggested.category_multipliers.get("code_generation"),
        Some(&dec!(1.9355))
    );
}
