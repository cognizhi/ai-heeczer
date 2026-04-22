//! Canonical event types (PRD §13).

use serde::{Deserialize, Serialize};

/// Top-level canonical event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    /// Schema version; must be `"1.0"` for v1.
    pub spec_version: String,
    /// RFC 4122 UUID; primary idempotency key.
    pub event_id: String,
    /// Optional parent / batch correlation id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    /// RFC 3339 / ISO 8601 timestamp in UTC.
    pub timestamp: String,
    /// Originating framework (`langgraph`, `google_adk`, `pydantic_ai`, ...).
    pub framework_source: String,
    /// Tenant id.
    pub workspace_id: String,
    /// Optional project id within the tenant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    /// Optional identity block.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity: Option<Identity>,
    /// Task descriptor.
    pub task: Task,
    /// Required telemetry metrics.
    pub metrics: Metrics,
    /// Optional execution context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<Context>,
    /// SDK metadata block.
    pub meta: Meta,
}

/// Identity block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    /// User identifier (opaque to the engine).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// Team identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>,
    /// Business unit identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_unit_id: Option<String>,
    /// Tier identifier; resolved against the supplied [`crate::TierSet`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tier_id: Option<String>,
}

/// Task descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    /// Task name (caller-defined).
    pub name: String,
    /// Optional category; missing or null normalizes to `uncategorized`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Optional sub-category.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sub_category: Option<String>,
    /// Outcome enum.
    pub outcome: Outcome,
}

/// Outcome enum (PRD §14.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// Task completed successfully.
    Success,
    /// Task partially completed.
    PartialSuccess,
    /// Task failed.
    Failure,
    /// Task timed out.
    Timeout,
}

/// Risk class enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskClass {
    /// Low risk.
    Low,
    /// Medium risk (default for context multiplier purposes).
    Medium,
    /// High risk.
    High,
}

/// Telemetry metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Metrics {
    /// Wall-clock duration in milliseconds.
    pub duration_ms: u64,
    /// Prompt tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_prompt: Option<u64>,
    /// Completion tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_completion: Option<u64>,
    /// Tool call count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_count: Option<u32>,
    /// Workflow step count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_steps: Option<u32>,
    /// Retry count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retries: Option<u32>,
    /// Artifact count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_count: Option<u32>,
    /// Output size proxy (caller-defined unit; multiplied by category output weight).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_size_proxy: Option<f64>,
}

/// Execution context (optional).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Context {
    /// True if a human substantially reviewed the task.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub human_in_loop: Option<bool>,
    /// True if review is required (drives `review_component`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_required: Option<bool>,
    /// Sampling temperature (drives ambiguity multiplier).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Risk class.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk_class: Option<RiskClass>,
    /// Free-form tags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// SDK metadata. `extensions` is the sole permitted location for unknown fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Meta {
    /// SDK language identifier (`rust`, `node`, `python`, `go`, `java`, `cli`, `test`).
    pub sdk_language: String,
    /// SDK semver string.
    pub sdk_version: String,
    /// Optional override of the scoring profile id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scoring_profile: Option<String>,
    /// Sole permitted unknown-field bucket (PRD §13).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,
}
