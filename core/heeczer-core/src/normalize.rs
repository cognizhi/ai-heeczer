//! Normalization rules (PRD §14.2.1). Coerces optional inputs to deterministic
//! defaults before scoring math runs. Pure data transform; no I/O.

use crate::event::{Context, Event, Metrics, Outcome, RiskClass};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Sentinel used when `task.category` is missing or null.
pub const UNCATEGORIZED: &str = "uncategorized";

/// Result of normalizing an [`Event`] into deterministic typed values.
#[derive(Debug, Clone, PartialEq)]
pub struct Normalized<'a> {
    /// Resolved category (never empty; falls back to `uncategorized`).
    pub category: &'a str,
    /// `tokens_prompt + tokens_completion`, missing parts treated as zero.
    pub total_tokens: Decimal,
    /// Duration in seconds as a `Decimal` (preserves sub-second precision).
    pub duration_seconds: Decimal,
    /// Workflow steps; absent → 0.
    pub workflow_steps: Decimal,
    /// Tool call count; absent → 0.
    pub tool_call_count: Decimal,
    /// Artifact count (uncapped here; cap is applied in scoring).
    pub artifact_count: Decimal,
    /// Output size proxy; absent → 0.
    pub output_size_proxy: Decimal,
    /// Retries; absent → 0.
    pub retries: Decimal,
    /// Resolved review_required flag.
    pub review_required: bool,
    /// Resolved human_in_loop flag.
    pub human_in_loop: bool,
    /// Resolved temperature.
    pub temperature: Decimal,
    /// Resolved risk class (defaults to `Medium`).
    pub risk_class: RiskClass,
    /// Outcome.
    pub outcome: Outcome,
    /// Whether `task.category` was missing in the input (drives confidence penalty).
    pub category_was_missing: bool,
    /// Whether token metrics were missing.
    pub tokens_were_missing: bool,
    /// Whether workflow_steps was missing.
    pub steps_were_missing: bool,
    /// Whether tool_call_count was missing.
    pub tools_were_missing: bool,
}

const EMPTY_CONTEXT: Context = Context {
    human_in_loop: None,
    review_required: None,
    temperature: None,
    risk_class: None,
    tags: None,
};

/// Normalize an event. Pure function; never mutates the event.
#[allow(clippy::cast_precision_loss)]
pub fn normalize(event: &Event) -> Normalized<'_> {
    let metrics: &Metrics = &event.metrics;
    let context: &Context = event.context.as_ref().unwrap_or(&EMPTY_CONTEXT);

    let category_was_missing = event.task.category.is_none();
    let category = event.task.category.as_deref().unwrap_or(UNCATEGORIZED);

    let tokens_prompt = metrics.tokens_prompt.map_or(Decimal::ZERO, Decimal::from);
    let tokens_completion = metrics
        .tokens_completion
        .map_or(Decimal::ZERO, Decimal::from);
    let total_tokens = tokens_prompt + tokens_completion;
    let tokens_were_missing =
        metrics.tokens_prompt.is_none() && metrics.tokens_completion.is_none();

    // duration_ms is u64; convert with 4-fractional-digit Decimal scale.
    let duration_seconds = Decimal::from(metrics.duration_ms) / dec!(1000);

    let workflow_steps = metrics.workflow_steps.map_or(Decimal::ZERO, Decimal::from);
    let steps_were_missing = metrics.workflow_steps.is_none();

    let tool_call_count = metrics.tool_call_count.map_or(Decimal::ZERO, Decimal::from);
    let tools_were_missing = metrics.tool_call_count.is_none();

    let artifact_count = metrics.artifact_count.map_or(Decimal::ZERO, Decimal::from);
    let retries = metrics.retries.map_or(Decimal::ZERO, Decimal::from);

    // f64 → Decimal via round-trip string to maintain determinism. The schema
    // bounds output_size_proxy and temperature so this is well-defined.
    let output_size_proxy = decimal_from_f64(metrics.output_size_proxy.unwrap_or(0.0));
    let temperature = decimal_from_f64(context.temperature.unwrap_or(0.0));

    Normalized {
        category,
        total_tokens,
        duration_seconds,
        workflow_steps,
        tool_call_count,
        artifact_count,
        output_size_proxy,
        retries,
        review_required: context.review_required.unwrap_or(false),
        human_in_loop: context.human_in_loop.unwrap_or(false),
        temperature,
        risk_class: context.risk_class.unwrap_or(RiskClass::Medium),
        outcome: event.task.outcome,
        category_was_missing,
        tokens_were_missing,
        steps_were_missing,
        tools_were_missing,
    }
}

/// Convert an `f64` to a `Decimal` deterministically by going via its shortest
/// round-tripping string form. Schema bounds keep this safe; callers should
/// already have validated the event with the JSON schema.
fn decimal_from_f64(v: f64) -> Decimal {
    if !v.is_finite() {
        return Decimal::ZERO;
    }
    // ryu produces the shortest round-trip string; serde_json uses ryu under the hood.
    let s = serde_json::to_string(&v).unwrap_or_else(|_| "0".to_string());
    s.parse::<Decimal>().unwrap_or(Decimal::ZERO)
}
