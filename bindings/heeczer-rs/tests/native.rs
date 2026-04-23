use heeczer::{Client, Event, IngestInput};

// Load the canonical fixture once.
fn canonical_event() -> Event {
    let raw = include_str!("../../../core/schema/fixtures/events/valid/01-prd-canonical.json");
    serde_json::from_str(raw).unwrap()
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
