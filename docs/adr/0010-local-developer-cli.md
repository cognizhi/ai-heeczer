# ADR-0010: Local Developer CLI (`heec`)

- **Status:** Accepted
- **Date:** 2026-04-22
- **Related:** PRD §12.13, §26, §28.1, §31; ADR-0001, ADR-0007
- **Supersedes:** —

## Context

The PRD mandates TDD discipline (§28.1) and a frictionless developer experience (§26), but plans 0001–0015 do not specify a single, language-neutral way for a contributor or framework integrator to:

- validate a candidate canonical event (PRD §13) without standing up the full ingestion service or writing throwaway language-specific glue;
- run the deterministic scoring engine (PRD §14) against a fixture or hand-crafted JSON event and inspect the explainability trace (PRD §16) interactively;
- diff two scoring outputs across `scoring_version`s for parity verification (PRD §14.7);
- exercise the storage layer locally (apply migrations, insert events, replay) against the SQLite adapter (plan 0003).

Without this surface, every contributor reinvents an ad hoc harness in their preferred SDK, which:

1. risks drifting from the canonical schema and scoring contract (a PRD §6.10 violation in spirit);
2. blocks framework adapter authors who do not yet have a SDK in their target language;
3. makes parity bugs (Rust vs. JS vs. Python) hard to triangulate because there is no neutral reference invocation;
4. forces dashboard and ingestion service work to depend on a running stack rather than a deterministic local fixture pipeline.

## Decision

Ship `heec`, a first-class Rust binary crate at `core/heeczer-cli/`, as part of the foundation deliverables. It is the **canonical reference invocation** of the Rust core (per ADR-0001) and is published to `crates.io` and as a prebuilt binary attached to GitHub Releases (PRD §27.4).

### Scope (MVP, Phase 1)

The CLI exposes the following subcommands, all backed by the `heeczer-core` crate:

- `heec schema validate <FILE|->` — validate a JSON event against `event.v1.json`. Exits non-zero with a human-readable error path on failure. Supports `--strict` (default) and `--compatibility` modes per PRD §13.
- `heec score <FILE|->` — run the scoring engine and emit the full `ScoreResult` (HEE, FEC, confidence, explainability trace) as JSON or pretty-printed table (`--format json|table`).
- `heec score --profile <PATH>` — override the default scoring profile.
- `heec score --tier <ID>` — pick a tier from the loaded tier set.
- `heec fixtures list|show <NAME>` — enumerate and emit shipped golden fixtures (useful for SDK authors to copy into their language's test suites).
- `heec diff <A.json> <B.json>` — diff two `ScoreResult`s and exit non-zero if they differ (parity helper).
- `heec migrate up|status|verify` — apply storage migrations against a configured SQLite/PostgreSQL URL (plan 0003 previously scoped this as a separate migration CLI; we collapse it into `heec` to give contributors a single tool — see "Naming" below).
- `heec version` — print CLI, `scoring_version`, `spec_version`, and core crate versions for bug reports.

### Scope (Phase 2 — added 2026-04-23)

The following subcommands are added in the post-foundation slice; they remain inside the same binary and reuse the existing `heeczer-core` and `heeczer-storage` deps:

- `heec score detail <FILE|->` — same scoring path as `heec score`, but emits the explainability trace (`heeczer-core::explain`) as a human-formatted, multi-line view (`--format text|json`). Shorthand for the most common debugging session.
- `heec validate profile <FILE|->` — validate a candidate scoring-profile JSON against `scoring_profile.v1.json` (the new `ProfileValidator`); exits non-zero with the JSON Pointer of the first failure.
- `heec validate tier <FILE|->` — validate a candidate tier-set JSON against the upcoming `tier_set.v1.json` schema (deferred until that schema is authored; the subcommand prints a clear "schema not yet shipped" error in the meantime so the surface is reserved).
- `heec replay <DB_URL> <event_id>` — fetch the persisted normalized event from `heec_events`, re-score with the currently configured profile, and emit a `ScoreResult` diff against the latest persisted score row (read-only; **does not insert** a new score row — that is reserved for the dashboard test-orchestration view per ADR-0012).
- `heec bench [--iter N] [--fixture PATH]` — measure `score()` p50/p95/p99 over N iterations of a fixture event; prints a single-line summary and exits non-zero if a `--budget-ms` flag is supplied and exceeded. Reuses Rust `Instant` (no external bench framework dependency).

These additions are reflected in PRD §12.21 (amended).

### Non-scope (MVP)

- No long-running server or queue worker. Use the ingestion service binary for that.
- No dashboard launcher. Use the dashboard's own dev server.
- No live tail of events. Use the ingestion service's structured logs.

### Naming

We adopt `heec` as the user-facing binary name (short, matches the product slug). The earlier separate-migration-CLI plan in plan 0003 §Migrations is collapsed into `heec migrate …` to give contributors **one tool**, not two. Plan 0003 will be amended in the same PR.

### Distribution

- Crate: `heec` on crates.io (binary-only crate, depends on `heeczer-core`).
- Prebuilt binaries: Linux/macOS/Windows on x86_64 and aarch64, attached to each GitHub Release, signed with cosign keyless OIDC (PRD §22 Security).
- Container: `ghcr.io/<org>/heec:<version>` for use in CI of downstream framework adapter repos.
- Homebrew tap and Scoop manifest are deferred to Phase 2.

## Alternatives Considered

1. **No CLI; a Rust example under `core/heeczer-core/examples/`.** Lowest cost, but invisible to non-Rust contributors, not installable, not signed, and not a stable invocation surface. Rejected because the PRD's TDD posture (§28.1) and the cross-language parity requirement (§14.7) need a published, versioned tool.
2. **A Node-based CLI shipped via `npx`.** Reuses the `bindings/node` package. Rejected because it makes the JS/TS binding a hard dependency for any contributor, even those working on the Go or Python SDK; also adds a Node runtime requirement to CI parity jobs.
3. **A Python CLI via `uv tool install`.** Same drawback as (2), with the addition that the Python binding (PyO3) does not yet expose every core surface and is itself under construction.
4. **One CLI per language SDK.** Five surfaces to keep in lockstep; guarantees drift and contradicts ADR-0001.

The Rust binary wins on: (a) zero runtime dependency for end users (static-ish binary), (b) reuses the same crate the SDKs FFI into, (c) signed prebuilt distribution fits the existing release pipeline (ADR-0009).

## Consequences

### Positive

- Single deterministic local harness for schema + scoring + storage, satisfying the "atomically test the analyzer" requirement raised during the foundation slice.
- Removes the need for a separate migration binary; one tool, one set of subcommands.
- Provides a stable contract surface (`heec score`'s JSON output) that framework adapter test suites in any language can shell out to.
- Enables a "golden fixture parity" CI job that runs `heec score` on every fixture and compares against checked-in expected outputs without booting any SDK.

### Negative

- One more crate to publish per release (negligible; release-please already aggregates).
- The CLI's UX is now part of the public contract (PRD §12.15) and changes are versioned alongside `scoring_version` / `spec_version`.
- Cross-platform prebuilt-binary CI matrix grows by one artifact per platform.

### Follow-ups

- Plan 0013 (Developer Experience) gains a checklist item: CLI quickstart in README and `make cli-install` Makefile target.
- Plan 0003 (Storage) §Migrations is amended so `heec migrate` is the only migration CLI surface.
- Plan 0012 (CI/CD) gains a `cli-build` job and a `cli-release-asset` step in the release workflow.
- A new contract test suite, `core/heeczer-cli/tests/contract/`, asserts that `heec score`'s JSON output is byte-equal to the corresponding golden fixture's expected output.

## References

- PRD §12.13 Makefile Support, §26 Developer Experience, §28.1 TDD Policy, §14.7 Scoring Contract Requirements
- ADR-0001 Rust as the Core Scoring Engine
- ADR-0007 Monorepo Tooling
- ADR-0009 Release Control Plane
- Plan 0003 §Migrations (amended by this ADR)
- Plan 0013 Developer Experience (amended by this ADR)
