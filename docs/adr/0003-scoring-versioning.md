# ADR-0003: Scoring Profile and Score Versioning

- **Status:** Accepted
- **Date:** 2026-04-22
- **Related:** PRD §14, §19.4, §20

## Context
Scoring formulas, multipliers, tiers, and rates evolve. Historical scores must remain reproducible and auditable. Re-scoring must not destroy prior outputs.

## Decision
- Every persisted score record carries `scoring_version`, `scoring_profile_id`, `tier_version`, and `rate_version`.
- `aih_scores` is append-only. Re-scoring inserts a new row keyed by `(event_id, scoring_version, scoring_profile_id)`; prior rows are never updated or deleted.
- `scoring_version` follows semver. Any change that could alter persisted decimal output is at least a minor version bump and requires updated golden fixtures and contract tests.
- Profile and rate tables are append-only with `effective_at` ranges; "current" is derived by query, not by mutation.
- Fixed-point decimal arithmetic per PRD §14.2.1 is enforced in the Rust core; all SDKs consume the core's output, never recompute.

## Alternatives Considered
- **Mutable scores updated in place** — simpler queries, irreversibly destroys audit history.
- **Untracked profile changes** — cannot reproduce a historical report.

## Consequences
- Positive: full reproducibility, defensible audit trail, safe re-scoring.
- Negative: storage growth proportional to re-scoring frequency; mitigated by retention policies (PRD §12.17) and aggregate roll-ups.
- Follow-ups: views or materialized tables for "current score per event" to keep dashboard queries simple.

## References
- PRD §14 Scoring Model
- PRD §19.4 Reliability
- PRD §20 Storage and Data Model
