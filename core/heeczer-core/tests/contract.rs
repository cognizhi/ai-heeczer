//! Contract tests for plan 0001 / ADR-0002.
//!
//! Verifies:
//! 1. Every valid fixture round-trips through the typed `Event` struct
//!    without data loss (parse → serialize → re-parse == original).
//! 2. Extension fields under `meta.extensions` survive a round-trip.
//! 3. Unknown top-level fields are rejected by both the schema validator
//!    (strict mode) and serde deserialization (`deny_unknown_fields`).

use heeczer_core::schema::{EventValidator, Mode};
use heeczer_core::Event;
use serde_json::Value;
use std::path::PathBuf;

fn fixture_dir(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../schema/fixtures/events")
        .join(rel)
}

fn load_valid_fixtures() -> Vec<(String, String)> {
    let dir = fixture_dir("valid");
    let mut out: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
        .map(|p| {
            let name = p.file_name().unwrap().to_string_lossy().into_owned();
            let body = std::fs::read_to_string(&p).unwrap();
            (name, body)
        })
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Round-trip contract: for each valid fixture, deserialize to `Event`,
/// serialize back to JSON, then parse both as `serde_json::Value` and compare.
/// Proves no data is silently dropped by the typed representation.
#[test]
fn every_valid_fixture_round_trips_losslessly() {
    for (name, body) in load_valid_fixtures() {
        let event: Event = serde_json::from_str(&body)
            .unwrap_or_else(|e| panic!("fixture `{name}` failed to deserialize: {e}"));

        let reserialised = serde_json::to_string(&event)
            .unwrap_or_else(|e| panic!("fixture `{name}` failed to serialize: {e}"));

        // Normalize both as serde_json::Value for semantic comparison.
        let original_val: Value = serde_json::from_str(&body)
            .unwrap_or_else(|e| panic!("fixture `{name}` original parse failed: {e}"));
        let roundtrip_val: Value = serde_json::from_str(&reserialised)
            .unwrap_or_else(|e| panic!("fixture `{name}` roundtrip parse failed: {e}"));

        // Optional fields absent from the fixture are not added (skip_serializing_if).
        // Optional fields present in the fixture survive (no data dropped).
        // We compare the round-trip output against the original: because Rust omits
        // null/absent optionals, the round-trip should be a subset-or-equal of the
        // original. We assert all keys present in the re-serialized form also appear
        // in the original with equal values.
        assert_eq!(
            original_val, roundtrip_val,
            "fixture `{name}` round-trip produced different value"
        );
    }
}

/// Extension fields under `meta.extensions` must survive a round-trip.
#[test]
fn meta_extensions_survive_round_trip() {
    let json = r#"{
        "spec_version": "1.0",
        "event_id": "00000000-0000-4000-8000-aabbccddeeff",
        "timestamp": "2026-04-22T10:00:00Z",
        "framework_source": "test",
        "workspace_id": "ws_ext",
        "task": { "name": "ext_test", "outcome": "success" },
        "metrics": { "duration_ms": 100 },
        "meta": {
            "sdk_language": "test",
            "sdk_version": "0.0.0",
            "extensions": { "custom_key": 42, "nested": { "x": true } }
        }
    }"#;

    let event: Event = serde_json::from_str(json).expect("deserialize");
    let ext = event
        .meta
        .extensions
        .as_ref()
        .expect("extensions must be Some");
    assert_eq!(ext["custom_key"], serde_json::json!(42));
    assert_eq!(ext["nested"]["x"], serde_json::json!(true));

    let reserialised = serde_json::to_string(&event).expect("serialize");
    let back: Value = serde_json::from_str(&reserialised).expect("re-parse");
    assert_eq!(
        back["meta"]["extensions"]["custom_key"],
        serde_json::json!(42)
    );
    assert_eq!(
        back["meta"]["extensions"]["nested"]["x"],
        serde_json::json!(true)
    );
}

/// The schema validator (strict mode) must reject an event with an unknown
/// top-level field. Also verified by `serde(deny_unknown_fields)` on Event.
#[test]
fn unknown_top_level_field_rejected_in_strict_mode() {
    let json = r#"{
        "spec_version": "1.0",
        "event_id": "00000000-0000-4000-8000-aabbccddeeff",
        "timestamp": "2026-04-22T10:00:00Z",
        "framework_source": "test",
        "workspace_id": "ws_strict",
        "task": { "name": "t", "outcome": "success" },
        "metrics": { "duration_ms": 100 },
        "meta": { "sdk_language": "test", "sdk_version": "0.0.0" },
        "forbidden_extra_field": "value"
    }"#;

    // Schema validator (EventValidator) must reject in strict mode.
    let validator = EventValidator::new_v1();
    let parsed: Value = serde_json::from_str(json).expect("parse JSON");
    assert!(
        validator.validate(&parsed, Mode::Strict).is_err(),
        "schema validator must reject unknown top-level field in strict mode"
    );

    // serde deserialization with deny_unknown_fields must also fail.
    assert!(
        serde_json::from_str::<Event>(json).is_err(),
        "serde must reject unknown top-level field (deny_unknown_fields)"
    );
}

/// Unknown fields inside `meta` (not in `extensions`) must be rejected.
#[test]
fn unknown_meta_field_rejected_in_strict_mode() {
    let json = r#"{
        "spec_version": "1.0",
        "event_id": "00000000-0000-4000-8000-aabbccddeeff",
        "timestamp": "2026-04-22T10:00:00Z",
        "framework_source": "test",
        "workspace_id": "ws_strict",
        "task": { "name": "t", "outcome": "success" },
        "metrics": { "duration_ms": 100 },
        "meta": {
            "sdk_language": "test",
            "sdk_version": "0.0.0",
            "unknown_meta_key": "oops"
        }
    }"#;

    let validator = EventValidator::new_v1();
    let parsed: Value = serde_json::from_str(json).expect("parse JSON");
    assert!(
        validator.validate(&parsed, Mode::Strict).is_err(),
        "schema validator must reject unknown field inside meta (use meta.extensions instead)"
    );
    assert!(
        serde_json::from_str::<Event>(json).is_err(),
        "serde must reject unknown field inside meta"
    );
}
