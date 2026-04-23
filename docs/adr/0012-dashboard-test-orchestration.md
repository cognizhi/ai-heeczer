# ADR-0012: Dashboard Test-Orchestration View

- **Status:** Accepted
- **Date:** 2026-04-23
- **Deciders:** Tech Lead, DevEx Engineer, Test Engineer
- **Related:** PRD §21 (amended by this ADR), §12.13, §28.1, ADR-0008, ADR-0010

## Context
Plan 0010 already scopes a dashboard for operational and analytical surfaces (PRD §21). The PRD's TDD discipline (§28.1) and the contributor request for a full back-to-back testing surface raise a separate need: a UI through which a developer or QA engineer can drive **the entire scoring pipeline end-to-end against shipped or user-supplied fixtures**, observe deterministic outputs, diff against golden expectations, and re-run the suite without leaving the browser.

`heec` (ADR-0010) covers atomic local validation, but it is a CLI. A GUI complement makes the fixture suite, parity checks, and benchmark harness discoverable to non-Rust contributors and is the natural home for the "GUI dashboard for back-to-back testing" capability that the foundation slice asked for.

## Decision
Plan 0010 (Dashboard) ships a **Test Orchestration view** under an explicit, RBAC-gated `/test-orchestration` route, alongside the user dashboard and admin console. The view is the GUI counterpart to `heec` and the parity / golden-fixture CI jobs.

### Scope
- **Fixture browser** — paginated list of every fixture under `core/schema/fixtures/` (events, scoring profiles, tier sets, golden ScoreResult JSONs), filterable by category and validity (`valid` / `invalid` / `edge`).
- **Run pipeline** — pick an event fixture + profile + tier-set, post to a back-to-back endpoint exposed by the ingestion service (`POST /v1/test/score-pipeline`), render the resulting `ScoreResult` and explainability trace.
- **Golden diff** — for any event fixture that has a corresponding `*.score_result.json` golden file, compute a structural JSON diff between the live result and the golden; surface mismatched paths inline.
- **Suite runner** — kick off the full golden suite in one click; results render as a compact pass/fail matrix with collapsible per-failure detail. Suite progress streams via SSE.
- **Benchmark stub** — invoke the future `heec bench` subcommand (or the equivalent benchmark endpoint) and chart p50/p95 of `score()` over N iterations.
- **Replay** — given an `event_id` already persisted (PRD §20 `heec_events`), re-score it with the currently selected profile and surface any drift versus the persisted score row.

### Constraints
- Read-only against production data unless the operator holds the `Admin` role (RBAC, PRD §22).
- No mutation of `heec_events` or `heec_scores` from this view; replays insert new score rows per the append-only invariant (plan 0003 §Append-only enforcement).
- Test-orchestration endpoints are gated behind a feature flag (`features.test_orchestration`) so production deployments can disable the suite runner if it would compete with live ingestion for capacity.
- All test-orchestration endpoints emit structured audit-log entries (PRD §22).
- UI uses the same component library as the user dashboard (ADR-0008) — no parallel tech stack.

### Non-scope
- No fixture **authoring** UI in v1; contributors author fixtures in the repo and the dashboard renders them.
- No long-running benchmark runs (>60s) — those stay in CI.
- No multi-tenant fixture isolation; fixtures are shared global resources.

## Alternatives Considered
- **CLI only.** Rejected: the user explicitly asked for a GUI; non-Rust contributors get a worse experience.
- **Separate "test-orchestration" microsite.** Rejected: doubles auth, RBAC, and component-library work; users would be context-switching between two URLs.
- **Hidden behind an admin-only flag.** Rejected: testing is a development concern, not strictly an admin one; RBAC + feature flag is sufficient gating.

## Consequences
- Positive: fixture suite, parity checks, and benchmark surface are discoverable from a browser; non-Rust contributors get the same testing leverage that Rust contributors get from `heec`.
- Positive: the back-to-back endpoint (`/v1/test/score-pipeline`) doubles as a contract surface for SDK parity tests that prefer HTTP over FFI.
- Negative: ingestion service grows a small, RBAC-gated test surface; we accept that and gate it behind a feature flag.
- Follow-ups: plan 0010 gains a **Test Orchestration** section; ingestion-service plan (0004) gains the `/v1/test/*` endpoints with RBAC + feature flag; PRD §21 amended (see below).

## References
- PRD §21 (amended)
- PRD §12.13, §12.21, §28.1, §22
- ADR-0008 Dashboard UI Framework
- ADR-0010 Local Developer CLI
- Plan 0010 Dashboard
- Plan 0004 Ingestion Service
