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

fn plan_0016_fixture_dir() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.join("../../testing/tests/fixtures/skills")
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
fn plan_0016_skill_fixtures_materialise_valid_events() {
    let event_validator = EventValidator::new_v1();
    let default_profile = heeczer_core::ScoringProfile::default_v1();
    let fixture_dir = plan_0016_fixture_dir();
    let mut entries: Vec<_> = std::fs::read_dir(&fixture_dir)
        .unwrap_or_else(|error| panic!("read_dir {}: {error}", fixture_dir.display()))
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("json"))
        .collect();
    entries.sort();

    for (index, path) in entries.iter().enumerate() {
        let body = std::fs::read_to_string(path).unwrap();
        let fixture: serde_json::Value = serde_json::from_str(&body).unwrap();
        let skill = fixture["skill"].as_str().expect("skill key");
        let expected = &fixture["expected_event"];
        let category = expected["task"]["category"]
            .as_str()
            .expect("task.category");
        assert!(
            default_profile.category_multipliers.contains_key(category),
            "Plan 0016 skill fixture `{skill}` uses category `{category}` not present in the default scoring profile"
        );

        let metrics = &expected["metrics"];
        let mut task = expected["task"].clone();
        task["name"] = serde_json::json!(format!("{skill}: local stack turn"));
        let mut context = expected["context"].clone();
        context["tags"] = serde_json::json!(["local-stack", "test", skill]);
        let tool_trace: Vec<_> = fixture["mock_script"]
            .as_array()
            .expect("mock_script array")
            .iter()
            .map(|step| step["tool"].clone())
            .collect();
        let mut expected_workflow_steps = 0;
        let mut expected_artifact_count = 0;
        let mut expected_output_size = 0.0;
        for tool in &tool_trace {
            match tool.as_str().expect("tool name") {
                "web_search" | "code_executor" | "document_reader" | "plan_reviewer" => {
                    expected_workflow_steps += 1;
                }
                _ => {}
            }
            match tool.as_str().expect("tool name") {
                "code_executor" | "data_analyst" | "summarizer" | "diff_generator" => {
                    expected_artifact_count += 1;
                }
                _ => {}
            }
            expected_output_size += match tool.as_str().expect("tool name") {
                "code_executor" => 0.5,
                "data_analyst" => 1.0,
                "summarizer" => 0.8,
                "diff_generator" => 0.3,
                _ => 0.0,
            };
        }
        assert_eq!(
            metrics["tool_call_count"]
                .as_u64()
                .expect("tool_call_count"),
            tool_trace.len() as u64,
            "Plan 0016 skill fixture `{skill}` has inconsistent tool_call_count"
        );
        assert_eq!(
            metrics["workflow_steps"].as_u64().expect("workflow_steps"),
            expected_workflow_steps,
            "Plan 0016 skill fixture `{skill}` has inconsistent workflow_steps"
        );
        assert_eq!(
            metrics["artifact_count"].as_u64().expect("artifact_count"),
            expected_artifact_count,
            "Plan 0016 skill fixture `{skill}` has inconsistent artifact_count"
        );
        let output_size = metrics["output_size_proxy"]
            .as_f64()
            .expect("output_size_proxy");
        assert!(
            (output_size - expected_output_size).abs() < 0.000_001,
            "Plan 0016 skill fixture `{skill}` has inconsistent output_size_proxy"
        );
        let event = serde_json::json!({
            "spec_version": "1.0",
            "event_id": format!("00000000-0000-4000-8000-{index:012}"),
            "correlation_id": format!("test-session:{index}"),
            "timestamp": "2026-04-27T00:00:00Z",
            "framework_source": "chatbot-test",
            "workspace_id": "local-test-contract",
            "task": task,
            "metrics": {
                "duration_ms": 1234,
                "tokens_prompt": metrics["tokens_prompt_min"],
                "tokens_completion": metrics["tokens_completion_min"],
                "tool_call_count": metrics["tool_call_count"],
                "workflow_steps": metrics["workflow_steps"],
                "retries": metrics["retries"],
                "artifact_count": metrics["artifact_count"],
                "output_size_proxy": metrics["output_size_proxy"]
            },
            "context": context,
            "meta": {
                "sdk_language": "test",
                "sdk_version": "0.0.0",
                "scoring_profile": "default",
                "extensions": {
                    "chatbot.skill": skill,
                    "chatbot.turn": 1,
                    "chatbot.tool_trace": tool_trace
                }
            }
        });
        event_validator
            .validate(&event, Mode::Strict)
            .unwrap_or_else(|error| {
                panic!(
                    "Plan 0016 skill fixture `{}` materialised an invalid event: {error}",
                    path.display()
                )
            });
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

#[test]
fn extensions_reject_privacy_sensitive_content_fields() {
    let v = EventValidator::new_v1();
    let prompt_body = r#"{
        "spec_version": "1.0",
        "event_id": "00000000-0000-4000-8000-0000000000fd",
        "timestamp": "2026-04-22T10:00:00Z",
        "framework_source": "test",
        "workspace_id": "ws_x",
        "task": { "name": "t", "outcome": "success" },
        "metrics": { "duration_ms": 1 },
        "meta": {
            "sdk_language": "test",
            "sdk_version": "0.0.0",
            "extensions": { "prompt_text": "never store raw prompt content" }
        }
    }"#;
    let output_body = r#"{
        "spec_version": "1.0",
        "event_id": "00000000-0000-4000-8000-0000000000fc",
        "timestamp": "2026-04-22T10:00:00Z",
        "framework_source": "test",
        "workspace_id": "ws_x",
        "task": { "name": "t", "outcome": "success" },
        "metrics": { "duration_ms": 1 },
        "meta": {
            "sdk_language": "test",
            "sdk_version": "0.0.0",
            "extensions": { "nested": { "output_text": "never store model output" } }
        }
    }"#;
    let secret_body = r#"{
        "spec_version": "1.0",
        "event_id": "00000000-0000-4000-8000-0000000000fb",
        "timestamp": "2026-04-22T10:00:00Z",
        "framework_source": "test",
        "workspace_id": "ws_x",
        "task": { "name": "t", "outcome": "success" },
        "metrics": { "duration_ms": 1 },
        "meta": {
            "sdk_language": "test",
            "sdk_version": "0.0.0",
            "extensions": { "apiKeys": ["k1"], "nested": { "file_attachments": ["/tmp/x"] } }
        }
    }"#;

    assert!(v.validate_str(prompt_body, Mode::Strict).is_err());
    assert!(v.validate_str(output_body, Mode::Strict).is_err());
    assert!(v.validate_str(secret_body, Mode::Strict).is_err());
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

// ---- TierSetValidator -------------------------------------------------------

#[test]
fn embedded_default_tier_set_validates() {
    let v = heeczer_core::schema::TierSetValidator::new_v1();
    let body = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../schema/tiers/default.v1.json"),
    )
    .unwrap();
    v.validate_str(&body, Mode::Strict)
        .expect("default tier set must validate against its own schema");
}

#[test]
fn tier_set_validator_rejects_missing_required_field() {
    let v = heeczer_core::schema::TierSetValidator::new_v1();
    // Missing `currency`, `tiers`, and `effective_at`.
    let body = r#"{
        "tier_set_id": "ts-test",
        "version": "1.0.0"
    }"#;
    assert!(
        v.validate_str(body, Mode::Strict).is_err(),
        "tier-set without required fields must be rejected"
    );
}

#[test]
fn tier_set_validator_rejects_empty_tiers_array() {
    let v = heeczer_core::schema::TierSetValidator::new_v1();
    let body = r#"{
        "tier_set_id": "ts-test",
        "version": "1.0.0",
        "effective_at": "2026-01-01T00:00:00Z",
        "currency": "USD",
        "tiers": []
    }"#;
    assert!(
        v.validate_str(body, Mode::Strict).is_err(),
        "tier-set with empty tiers array must be rejected"
    );
}

#[test]
fn tier_set_validator_rejects_unknown_top_level_field() {
    let v = heeczer_core::schema::TierSetValidator::new_v1();
    let body = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../schema/tiers/default.v1.json"),
    )
    .unwrap();
    let mut value: serde_json::Value = serde_json::from_str(&body).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .insert("rogue".to_string(), serde_json::json!(true));
    assert!(
        v.validate(&value, Mode::Strict).is_err(),
        "tier-set with unknown field must be rejected in strict mode"
    );
}
