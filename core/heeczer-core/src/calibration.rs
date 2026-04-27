//! Calibration helpers for reference benchmark packs (PRD §25).

use crate::confidence::ConfidenceBand;
use crate::event::{Context, Event, Meta, Metrics, Outcome, RiskClass, Task};
use crate::profile::ScoringProfile;
use crate::tier::TierSet;
use crate::{score, Error, Result, ScoreResult, SCORING_VERSION, SPEC_VERSION};
use rust_decimal::{Decimal, MathematicalOps, RoundingStrategy};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const MINUTES_DP: u32 = 2;
const RATIO_DP: u32 = 4;

/// Versioned benchmark pack used to calibrate a scoring profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkPack {
    /// Stable pack identifier.
    pub pack_id: String,
    /// Pack semantic version.
    pub version: String,
    /// Human-readable pack name.
    pub name: String,
    /// Pack description.
    pub description: String,
    /// Benchmark items scored during the run.
    pub items: Vec<BenchmarkItem>,
}

impl BenchmarkPack {
    /// Embedded reference benchmark pack (`core/schema/fixtures/calibration/reference-pack-v1.json`).
    pub fn reference_v1() -> Self {
        const REFERENCE_PACK: &str =
            include_str!("../../schema/fixtures/calibration/reference-pack-v1.json");
        serde_json::from_str(REFERENCE_PACK).expect("embedded reference pack must parse")
    }
}

/// Single benchmark definition within a pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkItem {
    /// Stable item identifier.
    pub item_id: String,
    /// Human-readable task name.
    pub name: String,
    /// Item description.
    pub description: String,
    /// Expected human-effort range for this task.
    pub expected_human_effort_minutes: ExpectedHumanEffortRange,
    /// Canonical task category to score against.
    pub task_category: String,
    /// Expected confidence band (`high`, `medium`, `low`, `very_low`).
    pub expected_confidence_band: String,
    /// Synthetic telemetry used to materialize a canonical event.
    pub telemetry_profile: BenchmarkTelemetryProfile,
}

/// Expected human-effort range for a benchmark item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedHumanEffortRange {
    /// Lower bound, in minutes.
    pub min: u64,
    /// Upper bound, in minutes.
    pub max: u64,
}

/// Synthetic telemetry profile used to generate a canonical event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkTelemetryProfile {
    /// Wall-clock duration in milliseconds.
    pub duration_ms: u64,
    /// Prompt token count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_prompt: Option<u64>,
    /// Completion token count.
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
    /// Output size proxy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_size_proxy: Option<f64>,
    /// Sampling temperature.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Risk class.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk_class: Option<RiskClass>,
    /// Whether a human remained in the loop.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub human_in_loop: Option<bool>,
    /// Whether review is required.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_required: Option<bool>,
}

/// Full calibration run report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalibrationRunReport {
    /// Pack identifier used for the run.
    pub pack_id: String,
    /// Pack version used for the run.
    pub pack_version: String,
    /// Profile identifier under test.
    pub profile_id: String,
    /// Profile version under test.
    pub profile_version: String,
    /// Suggested next profile version after applying adjustments.
    pub suggested_profile_version: String,
    /// Pinned scoring engine version.
    pub scoring_version: String,
    /// Canonical spec version used to synthesize events.
    pub spec_version: String,
    /// Per-item score and delta output.
    pub items: Vec<CalibrationItemReport>,
    /// Run-level aggregate metrics.
    pub summary: CalibrationSummary,
    /// Suggested category-multiplier updates.
    pub suggested_category_multipliers: Vec<CategoryAdjustmentSuggestion>,
}

/// Per-item score and delta output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalibrationItemReport {
    /// Stable item identifier.
    pub item_id: String,
    /// Canonical task category used for scoring.
    pub task_category: String,
    /// Lower bound of expected human effort, in minutes.
    pub expected_min_minutes: Decimal,
    /// Upper bound of expected human effort, in minutes.
    pub expected_max_minutes: Decimal,
    /// Midpoint of the expected human-effort range.
    pub expected_midpoint_minutes: Decimal,
    /// Estimated minutes produced by the scoring engine.
    pub estimated_minutes: Decimal,
    /// Signed delta from the expected range. `0` means in range.
    pub delta_from_range_minutes: Decimal,
    /// Signed delta from the expected midpoint.
    pub delta_from_midpoint_minutes: Decimal,
    /// Signed midpoint delta divided by midpoint.
    pub delta_from_midpoint_ratio: Decimal,
    /// Whether the estimate landed inside the expected range.
    pub within_expected_range: bool,
    /// Expected confidence band from the benchmark item.
    pub expected_confidence_band: String,
    /// Actual confidence band produced by scoring.
    pub actual_confidence_band: ConfidenceBand,
    /// Whether expected and actual confidence bands match.
    pub confidence_band_match: bool,
    /// Raw score result for explainability and downstream inspection.
    pub score: ScoreResult,
}

/// Run-level aggregate calibration metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalibrationSummary {
    /// Number of items in the run.
    pub total_items: usize,
    /// Number of estimates inside their expected range.
    pub within_expected_range: usize,
    /// Number of confidence-band matches.
    pub confidence_band_matches: usize,
    /// Root-mean-square error against expected midpoints.
    pub rmse_minutes: Decimal,
    /// Mean absolute error against expected ranges.
    pub mae_range_minutes: Decimal,
    /// Mean absolute error against expected midpoints.
    pub mae_midpoint_minutes: Decimal,
    /// Signed mean error against expected midpoints.
    pub bias_minutes: Decimal,
    /// Coefficient of determination against expected midpoints.
    pub r_squared: Decimal,
}

/// Suggested update for a category multiplier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CategoryAdjustmentSuggestion {
    /// Canonical task category.
    pub category: String,
    /// Existing category multiplier, or the uncategorized fallback.
    pub current_multiplier: Decimal,
    /// Multiplicative factor implied by the benchmark pack.
    pub calibration_factor: Decimal,
    /// Suggested replacement multiplier.
    pub suggested_multiplier: Decimal,
    /// Number of benchmark items contributing to this suggestion.
    pub supporting_items: usize,
    /// Whether the category was absent and would be newly added.
    pub adds_new_category: bool,
}

/// Run calibration on a benchmark pack and scoring profile.
pub fn run_calibration(
    pack: &BenchmarkPack,
    profile: &ScoringProfile,
    tiers: &TierSet,
) -> Result<CalibrationRunReport> {
    let suggested_profile_version = bump_patch_version(&profile.version);
    let mut items = Vec::with_capacity(pack.items.len());
    let mut category_factors: BTreeMap<String, Vec<Decimal>> = BTreeMap::new();
    let mut midpoint_sum = Decimal::ZERO;
    let mut squared_error_sum = Decimal::ZERO;
    let mut absolute_range_sum = Decimal::ZERO;
    let mut absolute_midpoint_sum = Decimal::ZERO;
    let mut signed_midpoint_sum = Decimal::ZERO;
    let mut within_expected_range = 0usize;
    let mut confidence_band_matches = 0usize;

    for (index, item) in pack.items.iter().enumerate() {
        let score = score(
            &synthesize_event(item, index, &profile.profile_id),
            profile,
            tiers,
            None,
        )?;
        let expected_min = Decimal::from(item.expected_human_effort_minutes.min);
        let expected_max = Decimal::from(item.expected_human_effort_minutes.max);
        let expected_midpoint = round_minutes((expected_min + expected_max) / dec!(2));
        let estimated_minutes = score.final_estimated_minutes;
        let delta_from_range = round_minutes(delta_from_range(
            estimated_minutes,
            expected_min,
            expected_max,
        ));
        let delta_from_midpoint = round_minutes(estimated_minutes - expected_midpoint);
        let delta_from_midpoint_ratio = round_ratio(if expected_midpoint.is_zero() {
            Decimal::ZERO
        } else {
            delta_from_midpoint / expected_midpoint
        });
        let in_range = delta_from_range.is_zero();
        let expected_confidence_band = parse_confidence_band(&item.expected_confidence_band)?;
        let confidence_band_match = expected_confidence_band == score.confidence_band;

        if in_range {
            within_expected_range += 1;
        }
        if confidence_band_match {
            confidence_band_matches += 1;
        }

        midpoint_sum += expected_midpoint;
        squared_error_sum += delta_from_midpoint * delta_from_midpoint;
        absolute_range_sum += abs(delta_from_range);
        absolute_midpoint_sum += abs(delta_from_midpoint);
        signed_midpoint_sum += delta_from_midpoint;

        category_factors
            .entry(item.task_category.clone())
            .or_default()
            .push(if estimated_minutes.is_zero() {
                Decimal::ONE
            } else {
                expected_midpoint / estimated_minutes
            });

        items.push(CalibrationItemReport {
            item_id: item.item_id.clone(),
            task_category: item.task_category.clone(),
            expected_min_minutes: round_minutes(expected_min),
            expected_max_minutes: round_minutes(expected_max),
            expected_midpoint_minutes: expected_midpoint,
            estimated_minutes,
            delta_from_range_minutes: delta_from_range,
            delta_from_midpoint_minutes: delta_from_midpoint,
            delta_from_midpoint_ratio,
            within_expected_range: in_range,
            expected_confidence_band: item.expected_confidence_band.clone(),
            actual_confidence_band: score.confidence_band,
            confidence_band_match,
            score,
        });
    }

    let total_items = items.len();
    let divisor = Decimal::from(total_items as u64);
    let mean_midpoint = if total_items == 0 {
        Decimal::ZERO
    } else {
        midpoint_sum / divisor
    };
    let ss_tot = items.iter().fold(Decimal::ZERO, |acc, item| {
        let delta = item.expected_midpoint_minutes - mean_midpoint;
        acc + (delta * delta)
    });
    let rmse = if total_items == 0 {
        Decimal::ZERO
    } else {
        (squared_error_sum / divisor)
            .sqrt()
            .unwrap_or(Decimal::ZERO)
    };
    let r_squared = if ss_tot.is_zero() {
        if squared_error_sum.is_zero() {
            Decimal::ONE
        } else {
            Decimal::ZERO
        }
    } else {
        Decimal::ONE - (squared_error_sum / ss_tot)
    };
    let uncategorized = profile
        .category_multipliers
        .get("uncategorized")
        .copied()
        .unwrap_or(Decimal::ONE);
    let suggested_category_multipliers = category_factors
        .into_iter()
        .map(|(category, factors)| {
            let supporting_items = items
                .iter()
                .filter(|item| item.task_category == category)
                .count();
            let current_multiplier = profile
                .category_multipliers
                .get(&category)
                .copied()
                .unwrap_or(uncategorized);
            let calibration_factor = round_ratio(median(factors));
            CategoryAdjustmentSuggestion {
                adds_new_category: !profile.category_multipliers.contains_key(&category),
                category,
                current_multiplier: round_ratio(current_multiplier),
                calibration_factor,
                suggested_multiplier: round_ratio(current_multiplier * calibration_factor),
                supporting_items,
            }
        })
        .collect();

    Ok(CalibrationRunReport {
        pack_id: pack.pack_id.clone(),
        pack_version: pack.version.clone(),
        profile_id: profile.profile_id.clone(),
        profile_version: profile.version.clone(),
        suggested_profile_version,
        scoring_version: SCORING_VERSION.to_string(),
        spec_version: SPEC_VERSION.to_string(),
        items,
        summary: CalibrationSummary {
            total_items,
            within_expected_range,
            confidence_band_matches,
            rmse_minutes: round_ratio(rmse),
            mae_range_minutes: average_or_zero(absolute_range_sum, total_items),
            mae_midpoint_minutes: average_or_zero(absolute_midpoint_sum, total_items),
            bias_minutes: average_or_zero(signed_midpoint_sum, total_items),
            r_squared: round_ratio(r_squared),
        },
        suggested_category_multipliers,
    })
}

/// Apply calibration suggestions to a scoring profile and bump the patch version.
pub fn build_suggested_profile(
    profile: &ScoringProfile,
    suggestions: &[CategoryAdjustmentSuggestion],
    effective_at: &str,
) -> ScoringProfile {
    let mut suggested = profile.clone();
    suggested.version = bump_patch_version(&profile.version);
    suggested.effective_at = effective_at.to_string();
    suggested.superseded_at = None;
    for suggestion in suggestions {
        suggested
            .category_multipliers
            .insert(suggestion.category.clone(), suggestion.suggested_multiplier);
    }
    suggested
}

fn synthesize_event(item: &BenchmarkItem, index: usize, scoring_profile: &str) -> Event {
    Event {
        spec_version: SPEC_VERSION.to_string(),
        event_id: format!("00000000-0000-4000-8000-{index:012x}"),
        correlation_id: Some(format!("calibration:{}", item.item_id)),
        timestamp: format!("2026-04-27T00:00:{:02}Z", index % 60),
        framework_source: "calibration_reference_pack".to_string(),
        workspace_id: "calibration".to_string(),
        project_id: None,
        identity: None,
        task: Task {
            name: item.name.clone(),
            category: Some(item.task_category.clone()),
            sub_category: None,
            outcome: Outcome::Success,
        },
        metrics: Metrics {
            duration_ms: item.telemetry_profile.duration_ms,
            tokens_prompt: item.telemetry_profile.tokens_prompt,
            tokens_completion: item.telemetry_profile.tokens_completion,
            tool_call_count: item.telemetry_profile.tool_call_count,
            workflow_steps: item.telemetry_profile.workflow_steps,
            retries: item.telemetry_profile.retries,
            artifact_count: item.telemetry_profile.artifact_count,
            output_size_proxy: item.telemetry_profile.output_size_proxy,
        },
        context: Some(Context {
            human_in_loop: item.telemetry_profile.human_in_loop,
            review_required: item.telemetry_profile.review_required,
            temperature: item.telemetry_profile.temperature,
            risk_class: item.telemetry_profile.risk_class,
            tags: None,
        }),
        meta: Meta {
            sdk_language: "cli".to_string(),
            sdk_version: env!("CARGO_PKG_VERSION").to_string(),
            scoring_profile: Some(scoring_profile.to_string()),
            extensions: None,
        },
    }
}

fn parse_confidence_band(value: &str) -> Result<ConfidenceBand> {
    match value.trim().to_ascii_lowercase().as_str() {
        "high" => Ok(ConfidenceBand::High),
        "medium" => Ok(ConfidenceBand::Medium),
        "low" => Ok(ConfidenceBand::Low),
        "very_low" | "very-low" | "very low" => Ok(ConfidenceBand::VeryLow),
        other => Err(Error::UnknownEnum {
            value: other.to_string(),
            field: "expected_confidence_band",
        }),
    }
}

fn delta_from_range(estimated: Decimal, min: Decimal, max: Decimal) -> Decimal {
    if estimated < min {
        estimated - min
    } else if estimated > max {
        estimated - max
    } else {
        Decimal::ZERO
    }
}

fn abs(value: Decimal) -> Decimal {
    if value.is_sign_negative() {
        -value
    } else {
        value
    }
}

fn average_or_zero(sum: Decimal, count: usize) -> Decimal {
    if count == 0 {
        Decimal::ZERO
    } else {
        round_minutes(sum / Decimal::from(count as u64))
    }
}

fn median(mut values: Vec<Decimal>) -> Decimal {
    if values.is_empty() {
        return Decimal::ONE;
    }
    values.sort_unstable();
    let middle = values.len() / 2;
    if values.len() % 2 == 1 {
        values[middle]
    } else {
        (values[middle - 1] + values[middle]) / dec!(2)
    }
}

fn bump_patch_version(version: &str) -> String {
    let mut parts = version.split('.');
    let major = parts.next().and_then(|value| value.parse::<u64>().ok());
    let minor = parts.next().and_then(|value| value.parse::<u64>().ok());
    let patch = parts.next().and_then(|value| value.parse::<u64>().ok());
    if let (Some(major), Some(minor), Some(patch), None) = (major, minor, patch, parts.next()) {
        format!("{major}.{minor}.{}", patch + 1)
    } else {
        format!("{version}.1")
    }
}

fn round_minutes(value: Decimal) -> Decimal {
    round_decimal(value, MINUTES_DP)
}

fn round_ratio(value: Decimal) -> Decimal {
    round_decimal(value, RATIO_DP)
}

fn round_decimal(value: Decimal, dp: u32) -> Decimal {
    let mut rounded = value.round_dp_with_strategy(dp, RoundingStrategy::MidpointAwayFromZero);
    rounded.rescale(dp);
    rounded
}
