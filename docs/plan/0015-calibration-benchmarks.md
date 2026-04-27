# Plan 15 — Calibration and benchmarks

- **Status:** Active — core/CLI calibration shipped; interactive dashboard workflow still pending
- **Owner:** Tech Lead + SDK Engineer
- **PRD:** §25, §12.8
- **ADR:** n/a

## Goal

Provide a versioned calibration framework so organizations can tune the scoring model against known human-effort baselines.

## Checklist

### Calibration data model

- [x] `heec_benchmark_packs` (versioned definitions). (SQLite migration 0003 + PG counterpart, session Apr-2026)
- [x] `heec_calibration_runs` (results tied to a profile + pack version). (SQLite migration 0003 + PG counterpart, session Apr-2026)
- [x] Append-only with audit trail. (`heec calibrate run --database-url ...` persists run rows and audit entries, session Apr-2026)

### Reference benchmark pack (PRD §25)

- [x] Definitions for: long-doc summarize, API spec gen, release notes draft, CI triage, root-cause analysis. (`core/schema/fixtures/calibration/reference-pack-v1.json` with 5 items, session Apr-2026)
- [x] Each item: expected human-effort range, telemetry profile, category, expected confidence band. (session Apr-2026)

### Calibration workflow

- [x] CLI: `heec calibrate run --pack <id> --profile <id>` implemented in `heeczer-cli` (per ADR-0010). (session Apr-2026)
- [x] Output: per-item delta from expected range, suggested profile adjustments. (`heeczer-core::run_calibration`, session Apr-2026)
- [x] Profile updates create a new profile version (never mutate). (`--output-profile` writes a patch-bumped profile artifact; persisted suggested profiles also version bump, session Apr-2026)

### Dashboard

- [ ] Calibration page (Plan 10): pack picker, run history, delta visualization.

### Tests

- [x] Unit on calibration math. (`core/heeczer-core/tests/calibration.rs`, session Apr-2026)
- [x] Integration: end-to-end run on the reference pack. (`core/heeczer-cli/tests/cli.rs`, session Apr-2026)

### Docs

- [x] `docs/architecture/benchmarks.md` with targets, reference hardware, payload, and bench-smoke workflow description. (session Apr-2026)
- [x] `docs/architecture/calibration.md`. (session Cat-3)
- [x] User guide in dashboard help. (`dashboard/src/app/admin/calibration/page.tsx`, session Apr-2026)

## Acceptance

- Reference pack ships and runs green.
