# Plan 15 — Calibration and benchmarks

- **Status:** Active (Phase 2/3 features)
- **Owner:** Tech Lead + SDK Engineer
- **PRD:** §25, §12.8
- **ADR:** n/a

## Goal
Provide a versioned calibration framework so organizations can tune the scoring model against known human-effort baselines.

## Checklist

### Calibration data model
- [ ] `aih_benchmark_packs` (versioned definitions).
- [ ] `aih_calibration_runs` (results tied to a profile + pack version).
- [ ] Append-only with audit trail.

### Reference benchmark pack (PRD §25)
- [ ] Definitions for: long-doc summarize, API spec gen, release notes draft, CI triage, root-cause analysis.
- [ ] Each item: expected human-effort range, telemetry profile, category, expected confidence band.

### Calibration workflow
- [ ] CLI: `aih calibrate run --pack <id> --profile <id>` (per ADR-0010).
- [ ] Output: per-item delta from expected range, suggested profile adjustments.
- [ ] Profile updates create a new profile version (never mutate).

### Dashboard
- [ ] Calibration page (Plan 10): pack picker, run history, delta visualization.

### Tests
- [ ] Unit on calibration math.
- [ ] Integration: end-to-end run on the reference pack.

### Docs
- [ ] `docs/architecture/calibration.md`.
- [ ] User guide in dashboard help.

## Acceptance
- Reference pack ships and runs green.
