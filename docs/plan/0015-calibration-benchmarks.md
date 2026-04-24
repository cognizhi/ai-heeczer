# Plan 15 — Calibration and benchmarks

- **Status:** Active (Phase 2/3 features)
- **Owner:** Tech Lead + SDK Engineer
- **PRD:** §25, §12.8
- **ADR:** n/a

## Goal
Provide a versioned calibration framework so organizations can tune the scoring model against known human-effort baselines.

## Checklist

### Calibration data model
- [x] `heec_benchmark_packs` (versioned definitions). (SQLite migration 0003 + PG counterpart, session Apr-2026)
- [x] `heec_calibration_runs` (results tied to a profile + pack version). (SQLite migration 0003 + PG counterpart, session Apr-2026)
- [ ] Append-only with audit trail.

### Reference benchmark pack (PRD §25)
- [x] Definitions for: long-doc summarize, API spec gen, release notes draft, CI triage, root-cause analysis. (`core/schema/fixtures/calibration/reference-pack-v1.json` with 5 items, session Apr-2026)
- [x] Each item: expected human-effort range, telemetry profile, category, expected confidence band. (session Apr-2026)

### Calibration workflow
- [x] CLI: `heec calibrate run --pack <id> --profile <id>` stub added to `heeczer-cli` (per ADR-0010). (session Apr-2026)
- [ ] Output: per-item delta from expected range, suggested profile adjustments.
- [ ] Profile updates create a new profile version (never mutate).

### Dashboard
- [ ] Calibration page (Plan 10): pack picker, run history, delta visualization.

### Tests
- [ ] Unit on calibration math.
- [ ] Integration: end-to-end run on the reference pack.

### Docs
- [x] `docs/architecture/benchmarks.md` with targets, reference hardware, payload, and bench-smoke workflow description. (session Apr-2026)
- [ ] `docs/architecture/calibration.md`.
- [ ] User guide in dashboard help.

## Acceptance
- Reference pack ships and runs green.
