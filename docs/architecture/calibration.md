# Calibration Architecture

Calibration tunes the scoring profile against known human-effort baselines.
The current Plan 0015 implementation ships a deterministic reference-pack
workflow that synthesizes canonical events from telemetry profiles, scores
them with the Rust core, reports calibration drift, suggests category
multiplier updates, and can persist completed runs for audit review.

## Goals

1. **Accuracy** — estimates should converge toward expected human effort.
2. **Reproducibility** — the pure calibration math is deterministic for the same pack and source profile.
3. **Auditability** — calibration runs and suggested profiles can be stored alongside an audit entry.

## Data model

### Reference pack

A reference pack is a versioned JSON document that describes representative
tasks using expected effort bounds plus synthetic telemetry, rather than a
fully captured canonical event:

```json
{
    "pack_id": "reference-pack",
    "version": "1.0.0",
    "name": "Reference Benchmark Pack",
    "description": "PRD §25 reference tasks used for calibration.",
    "items": [
        {
            "item_id": "ci-triage",
            "name": "CI failure triage",
            "description": "Diagnose a flaky CI run and propose a fix plan.",
            "task_category": "analysis",
            "expected_human_effort_minutes": { "min": 15, "max": 45 },
            "expected_confidence_band": "medium",
            "telemetry_profile": {
                "duration_ms": 12000,
                "tokens_prompt": 3000,
                "tokens_completion": 1200,
                "tool_call_count": 8,
                "workflow_steps": 6,
                "retries": 1
            }
        }
    ]
}
```

The embedded reference pack lives at `core/schema/fixtures/calibration/reference-pack-v1.json`.

### Persisted records

When the CLI is given `--database-url`, it persists:

- `heec_benchmark_packs` for the shared pack definition (`workspace_id = NULL`)
- `heec_calibration_runs` for the completed report JSON
- `heec_scoring_profiles` for the suggested next profile version
- `heec_audit_log` for both `scoring_profile_calibrated` and `calibration_run_completed`

The current persistence path is SQLite-backed because the CLI uses the embedded
storage migrator directly.

## Calibration workflow

```text
reference pack (telemetry profiles + expected ranges)
  │
  ▼
heec calibrate run --pack reference-pack --profile default
  │
  ├── synthesize deterministic canonical events from each telemetry profile
  ├── score each event with the selected profile and default tiers
  ├── compute per-item deltas against the expected range and midpoint
  ├── summarize RMSE / MAE / bias / R² across the pack
  ├── suggest category-multiplier updates from category-level drift
  └── optionally write a newly versioned suggested profile artifact
```

Implemented CLI surface:

```text
heec calibrate run \
  --pack <reference-pack|path/to/pack.json> \
  --profile <default|path/to/profile.json> \
  [--output-profile path/to/calibrated-profile.json] \
  [--database-url sqlite:///tmp/heec.sqlite?mode=rwc] \
  [--workspace default]
```

The command emits a JSON report to stdout. `--output-profile` writes a patch-
bumped profile file; the source profile is never mutated in place. When the
same workspace persists repeated calibration runs, the CLI allocates the next
unused patch version so suggested profiles remain append-only. Persisted
reports may therefore vary by workspace database state even when the scoring
math for a given pack and source profile stays unchanged.

## Metrics

| Metric                 | Description                                                                                |
| ---------------------- | ------------------------------------------------------------------------------------------ |
| `rmse_minutes`         | Root-mean-square error versus each benchmark item's expected midpoint.                     |
| `mae_range_minutes`    | Mean absolute distance from the expected range. `0` means every item landed in range.      |
| `mae_midpoint_minutes` | Mean absolute distance from the expected midpoint.                                         |
| `bias_minutes`         | Signed mean error versus the expected midpoint. Positive means systematic over-estimation. |
| `r_squared`            | Coefficient of determination versus expected midpoints.                                    |

Each item in the report also includes:

- estimated minutes
- signed delta from the expected range
- signed delta from the expected midpoint
- expected vs actual confidence band
- the full `ScoreResult` for explainability

## Suggested adjustments

The current implementation suggests category multiplier updates only. For each
category present in the pack, the calibration engine computes a median
calibration factor from:

```text
expected midpoint minutes / estimated minutes
```

That factor is multiplied by the current category multiplier. If the category
does not already exist in the profile, the engine uses the `uncategorized`
multiplier as the starting point and marks the suggestion as adding a new
category.

## Versioning policy

- Scoring profiles follow [semantic versioning](https://semver.org/).
  Multiplier-only changes are emitted as a patch bump.
- Tier sets remain versioned independently.
- `ScoreResult.scoring_version` and `ScoreResult.spec_version` continue to pin
  the exact core and canonical-event versions used for each item score.

See [ADR-0003](../adr/0003-scoring-versioning.md) for the underlying profile
versioning policy.

## Roadmap (plan 0015)

| Item                                | Status                    |
| ----------------------------------- | ------------------------- |
| Reference pack fixture              | Shipped                   |
| `heec calibrate run` CLI            | Shipped                   |
| RMSE / MAE / R² report output       | Shipped                   |
| Suggested profile artifact          | Shipped                   |
| Persisted run history + audit trail | Shipped (SQLite CLI path) |
| Dashboard calibration page          | Planned                   |
| Automated calibration CI job        | Planned                   |
