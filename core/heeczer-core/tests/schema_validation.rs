//! Schema validation tests using the shared fixtures.

use heeczer_core::schema::{EventValidator, Mode};
use std::path::PathBuf;

fn fixture_dir(rel: &str) -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.join("../schema/fixtures/events").join(rel)
}

fn load_jsons(dir: &str) -> Vec<(String, String)> {
    let p = fixture_dir(dir);
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&p).unwrap_or_else(|e| panic!("read_dir {}: {e}", p.display())) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let body = std::fs::read_to_string(&path).unwrap();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            out.push((name, body));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

#[test]
fn every_valid_fixture_validates() {
    let v = EventValidator::new_v1();
    for (name, body) in load_jsons("valid") {
        v.validate_str(&body, Mode::Strict)
            .unwrap_or_else(|e| panic!("valid fixture `{name}` failed: {e}"));
    }
}

#[test]
fn every_edge_fixture_validates() {
    let v = EventValidator::new_v1();
    for (name, body) in load_jsons("edge") {
        v.validate_str(&body, Mode::Strict)
            .unwrap_or_else(|e| panic!("edge fixture `{name}` failed: {e}"));
    }
}

#[test]
fn every_invalid_fixture_rejects() {
    let v = EventValidator::new_v1();
    for (name, body) in load_jsons("invalid") {
        let result = v.validate_str(&body, Mode::Strict);
        assert!(
            result.is_err(),
            "invalid fixture `{name}` was unexpectedly accepted"
        );
    }
}

#[test]
fn unknown_top_level_field_is_rejected_in_strict_mode() {
    let v = EventValidator::new_v1();
    let body = r#"{
        "spec_version": "1.0",
        "event_id": "00000000-0000-4000-8000-0000000000ff",
        "timestamp": "2026-04-22T10:00:00Z",
        "framework_source": "test",
        "workspace_id": "ws_x",
        "task": { "name": "t", "outcome": "success" },
        "metrics": { "duration_ms": 1 },
        "meta": { "sdk_language": "test", "sdk_version": "0.0.0" },
        "rogue": true
    }"#;
    assert!(v.validate_str(body, Mode::Strict).is_err());
}

#[test]
fn extensions_passthrough_is_valid() {
    let v = EventValidator::new_v1();
    let body = r#"{
        "spec_version": "1.0",
        "event_id": "00000000-0000-4000-8000-0000000000fe",
        "timestamp": "2026-04-22T10:00:00Z",
        "framework_source": "test",
        "workspace_id": "ws_x",
        "task": { "name": "t", "outcome": "success" },
        "metrics": { "duration_ms": 1 },
        "meta": {
            "sdk_language": "test",
            "sdk_version": "0.0.0",
            "extensions": { "vendor.thing": [1, 2, 3] }
        }
    }"#;
    v.validate_str(body, Mode::Strict).unwrap();
}

// ---- ProfileValidator (foundation backlog) ---------------------------------

#[test]
fn embedded_default_profile_validates() {
    let v = heeczer_core::schema::ProfileValidator::new_v1();
    let body = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../schema/profiles/default.v1.json"),
    )
    .unwrap();
    v.validate_str(&body, Mode::Strict)
        .expect("embedded default profile must validate against its own schema");
}

#[test]
fn profile_validator_rejects_missing_required_section() {
    let v = heeczer_core::schema::ProfileValidator::new_v1();
    // Missing required `components`, `category_multipliers`, etc.
    let body = r#"{
        "profile_id": "x",
        "version": "1.0.0",
        "effective_at": "2026-04-22T00:00:00Z"
    }"#;
    assert!(v.validate_str(body, Mode::Strict).is_err());
}

#[test]
fn profile_validator_rejects_unknown_top_level_field() {
    let v = heeczer_core::schema::ProfileValidator::new_v1();
    let body = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../schema/profiles/default.v1.json"),
    )
    .unwrap();
    let mut value: serde_json::Value = serde_json::from_str(&body).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .insert("rogue".to_string(), serde_json::json!(true));
    assert!(v.validate(&value, Mode::Strict).is_err());
}

#[test]
fn scoring_profile_struct_rejects_unknown_field_via_serde() {
    // deny_unknown_fields on the top-level ScoringProfile struct (foundation
    // backlog: only sub-structs were guarded before).
    let body = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../schema/profiles/default.v1.json"),
    )
    .unwrap();
    let mut value: serde_json::Value = serde_json::from_str(&body).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .insert("rogue_top".to_string(), serde_json::json!("x"));
    let res: Result<heeczer_core::ScoringProfile, _> = serde_json::from_value(value);
    assert!(
        res.is_err(),
        "ScoringProfile must reject unknown top-level fields"
    );
}
