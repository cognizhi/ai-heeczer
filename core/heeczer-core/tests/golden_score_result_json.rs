//! Byte-stable golden ScoreResult JSON test (foundation backlog).
//!
//! Locks the serialized JSON shape and field ordering of a ScoreResult so any
//! accidental serde rename, field reorder, or numeric-formatting change is
//! caught at PR time. To intentionally update the golden, regenerate via:
//!
//!     ./target/debug/heec score \
//!         core/schema/fixtures/events/valid/01-prd-canonical.json \
//!         --format pretty \
//!         > core/schema/fixtures/golden/01-prd-canonical.score_result.json
//!
//! and bump SCORING_VERSION + amend ADR-0003 if the change is mathematical.

use heeczer_core::{score, Event, ScoringProfile, TierSet};

const GOLDEN_PATH: &str = "../schema/fixtures/golden/01-prd-canonical.score_result.json";
const EVENT_PATH: &str = "../schema/fixtures/events/valid/01-prd-canonical.json";

fn manifest_relative(p: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(p)
}

#[test]
fn canonical_score_result_matches_golden_json() {
    let body = std::fs::read_to_string(manifest_relative(EVENT_PATH)).unwrap();
    let event: Event = serde_json::from_str(&body).unwrap();
    let result = score(
        &event,
        &ScoringProfile::default_v1(),
        &TierSet::default_v1(),
        None,
    )
    .unwrap();

    let actual = serde_json::to_string_pretty(&result).unwrap();
    let expected = std::fs::read_to_string(manifest_relative(GOLDEN_PATH))
        .expect("golden file missing — run heec score to regenerate");

    // Normalise trailing whitespace/newlines so platform line-endings don't trip the test.
    assert_eq!(
        actual.trim_end(),
        expected.trim_end(),
        "ScoreResult JSON drifted from golden. If intentional, regenerate per the test header doc."
    );
}
