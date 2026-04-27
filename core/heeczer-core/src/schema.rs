//! JSON Schema validation against the embedded canonical schemas.

use crate::error::{Error, Result};

/// Embedded canonical event schema (`event.v1.json`). Compiled into the binary so
/// every consumer validates against the same bytes.
pub const EVENT_SCHEMA_V1: &str = include_str!("../schema/event.v1.json");

/// Embedded scoring profile schema (`scoring_profile.v1.json`).
pub const SCORING_PROFILE_SCHEMA_V1: &str = include_str!("../schema/scoring_profile.v1.json");

/// Embedded tier-set schema (`tier_set.v1.json`).
pub const TIER_SET_SCHEMA_V1: &str = include_str!("../schema/tier_set.v1.json");

/// Validation mode. PRD §13.
///
/// Only `Strict` exists today. A future `Compatibility` mode (drop a known
/// allowlist of unknown top-level fields with a warning) is planned but is not
/// implemented; we refuse to expose a no-op variant and silently mislead
/// callers. See `docs/plan/0001-schema-and-contracts.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Mode {
    /// Strict mode rejects every unknown top-level field outside `meta.extensions`.
    #[default]
    Strict,
}

/// Compile-once event-schema validator. Cheap to clone (`Arc`-backed internally).
pub struct EventValidator {
    validator: jsonschema::Validator,
}

impl EventValidator {
    /// Build a validator against the embedded `event.v1.json`.
    pub fn new_v1() -> Self {
        let schema: serde_json::Value =
            serde_json::from_str(EVENT_SCHEMA_V1).expect("embedded event schema must parse");
        let validator = jsonschema::options()
            .with_draft(jsonschema::Draft::Draft202012)
            .should_validate_formats(true)
            .build(&schema)
            .expect("embedded event schema must compile");
        Self { validator }
    }

    /// Validate a parsed JSON value against the schema.
    pub fn validate(&self, value: &serde_json::Value, _mode: Mode) -> Result<()> {
        // jsonschema 0.30: iter_errors gives all errors; we surface the first one
        // because callers want a deterministic, single, actionable message.
        if let Some(err) = self.validator.iter_errors(value).next() {
            return Err(Error::Schema {
                path: err.instance_path().to_string(),
                message: err.to_string(),
            });
        }
        reject_privacy_sensitive_extensions(value)?;
        Ok(())
    }

    /// Validate a JSON string.
    pub fn validate_str(&self, json: &str, mode: Mode) -> Result<()> {
        let value: serde_json::Value = serde_json::from_str(json)?;
        self.validate(&value, mode)
    }
}

impl Default for EventValidator {
    fn default() -> Self {
        Self::new_v1()
    }
}

fn reject_privacy_sensitive_extensions(value: &serde_json::Value) -> Result<()> {
    let Some(extensions) = value.get("meta").and_then(|meta| meta.get("extensions")) else {
        return Ok(());
    };

    if let Some(path) = find_privacy_sensitive_path(extensions, "/meta/extensions") {
        return Err(Error::Schema {
            path,
            message: "privacy-sensitive content fields are not allowed; store only telemetry and metadata".into(),
        });
    }

    Ok(())
}

fn find_privacy_sensitive_path(value: &serde_json::Value, path: &str) -> Option<String> {
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                let child_path = format!("{path}/{}", escape_json_pointer_segment(key));
                if is_privacy_sensitive_extension_key(key) && !child.is_null() {
                    return Some(child_path);
                }
                if let Some(found) = find_privacy_sensitive_path(child, &child_path) {
                    return Some(found);
                }
            }
            None
        }
        serde_json::Value::Array(items) => items.iter().enumerate().find_map(|(index, child)| {
            find_privacy_sensitive_path(child, &format!("{path}/{index}"))
        }),
        _ => None,
    }
}

fn is_privacy_sensitive_extension_key(key: &str) -> bool {
    matches!(
        normalize_extension_key(key).as_str(),
        "prompt"
            | "prompttext"
            | "promptcontent"
            | "rawprompt"
            | "systemprompt"
            | "output"
            | "outputtext"
            | "outputcontent"
            | "modeloutput"
            | "completion"
            | "completiontext"
            | "responsetext"
            | "responsecontent"
            | "attachment"
            | "attachments"
            | "fileattachment"
            | "fileattachments"
            | "secret"
            | "secrets"
            | "accesstoken"
            | "accesstokens"
            | "apikey"
            | "apikeys"
    )
}

fn normalize_extension_key(key: &str) -> String {
    key.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|ch| ch.to_ascii_lowercase())
        .collect()
}

fn escape_json_pointer_segment(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

/// Compile-once scoring-profile-schema validator. Mirrors [`EventValidator`].
///
/// Foundation backlog item (security): profiles loaded via `--profile` or any
/// future control-plane API previously bypassed JSON Schema validation, so
/// malformed profiles only failed at serde-deserialize time with cryptic
/// errors. `ProfileValidator` runs the strict, embedded `scoring_profile.v1`
/// schema before any deserialization.
pub struct ProfileValidator {
    validator: jsonschema::Validator,
}

impl ProfileValidator {
    /// Build a validator against the embedded `scoring_profile.v1.json`.
    pub fn new_v1() -> Self {
        let schema: serde_json::Value = serde_json::from_str(SCORING_PROFILE_SCHEMA_V1)
            .expect("embedded scoring profile schema must parse");
        let validator = jsonschema::options()
            .with_draft(jsonschema::Draft::Draft202012)
            .should_validate_formats(true)
            .build(&schema)
            .expect("embedded scoring profile schema must compile");
        Self { validator }
    }

    /// Validate a parsed JSON value against the schema.
    pub fn validate(&self, value: &serde_json::Value, _mode: Mode) -> Result<()> {
        if let Some(err) = self.validator.iter_errors(value).next() {
            return Err(Error::Schema {
                path: err.instance_path().to_string(),
                message: err.to_string(),
            });
        }
        Ok(())
    }

    /// Validate a JSON string.
    pub fn validate_str(&self, json: &str, mode: Mode) -> Result<()> {
        let value: serde_json::Value = serde_json::from_str(json)?;
        self.validate(&value, mode)
    }
}

impl Default for ProfileValidator {
    fn default() -> Self {
        Self::new_v1()
    }
}

/// Compile-once tier-set-schema validator. Mirrors [`EventValidator`].
pub struct TierSetValidator {
    validator: jsonschema::Validator,
}

impl TierSetValidator {
    /// Build a validator against the embedded `tier_set.v1.json`.
    pub fn new_v1() -> Self {
        let schema: serde_json::Value =
            serde_json::from_str(TIER_SET_SCHEMA_V1).expect("embedded tier set schema must parse");
        let validator = jsonschema::options()
            .with_draft(jsonschema::Draft::Draft202012)
            .should_validate_formats(true)
            .build(&schema)
            .expect("embedded tier set schema must compile");
        Self { validator }
    }

    /// Validate a parsed JSON value against the schema.
    pub fn validate(&self, value: &serde_json::Value, _mode: Mode) -> Result<()> {
        if let Some(err) = self.validator.iter_errors(value).next() {
            return Err(Error::Schema {
                path: err.instance_path().to_string(),
                message: err.to_string(),
            });
        }
        Ok(())
    }

    /// Validate a JSON string.
    pub fn validate_str(&self, json: &str, mode: Mode) -> Result<()> {
        let value: serde_json::Value = serde_json::from_str(json)?;
        self.validate(&value, mode)
    }
}

impl Default for TierSetValidator {
    fn default() -> Self {
        Self::new_v1()
    }
}
