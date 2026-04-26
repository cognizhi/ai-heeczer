use heeczer::{Client, Event, IngestInput};
use std::fs;
use std::path::PathBuf;

// Load the canonical fixture once.
fn canonical_event() -> Event {
    let raw = include_str!("../../../core/schema/fixtures/events/valid/01-prd-canonical.json");
    serde_json::from_str(raw).unwrap()
}

fn valid_fixture_paths() -> Vec<PathBuf> {
    let dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../core/schema/fixtures/events/valid");
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)
        .expect("valid fixture dir should be readable")
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    paths
}

#[test]
fn native_score_canonical_event() {
    let client = Client::native();
    let result = client
        .score_event(IngestInput {
            workspace_id: "ws_test".into(),
            event: canonical_event(),
            profile: None,
            tier_set: None,
            tier_override: None,
        })
        .expect("score should succeed");

    // Smoke checks — exact values belong in golden tests at the core layer.
    assert!(!result.scoring_version.is_empty());
    assert!(!result.human_summary.is_empty());
    assert!(result.final_estimated_minutes > rust_decimal::Decimal::ZERO);
}

#[test]
fn native_score_is_deterministic() {
    let client = Client::native();
    let event = canonical_event();
    let r1 = client
        .score_event(IngestInput {
            workspace_id: "ws".into(),
            event: event.clone(),
            profile: None,
            tier_set: None,
            tier_override: None,
        })
        .unwrap();
    let r2 = client
        .score_event(IngestInput {
            workspace_id: "ws".into(),
            event,
            profile: None,
            tier_set: None,
            tier_override: None,
        })
        .unwrap();
    assert_eq!(r1.final_estimated_minutes, r2.final_estimated_minutes);
    assert_eq!(r1.confidence_band, r2.confidence_band);
}

#[test]
fn native_scores_every_valid_shared_fixture() {
    let client = Client::native();
    let fixtures = valid_fixture_paths();
    assert!(!fixtures.is_empty(), "expected at least one valid fixture");

    for path in fixtures {
        let raw = fs::read_to_string(&path).expect("fixture should be readable");
        let event: Event = serde_json::from_str(&raw).expect("fixture should parse as Event");
        let result = client
            .score_event(IngestInput {
                workspace_id: event.workspace_id.clone(),
                event,
                profile: None,
                tier_set: None,
                tier_override: None,
            })
            .unwrap_or_else(|err| panic!("{} should score: {err}", path.display()));
        assert!(
            result.final_estimated_minutes >= rust_decimal::Decimal::ZERO,
            "{} should produce non-negative minutes",
            path.display()
        );
    }
}
