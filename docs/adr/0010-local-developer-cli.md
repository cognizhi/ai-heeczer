# ADR-0010: Local Developer CLI (`aih`)

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

Ship `aih`, a first-class Rust binary crate at `core/heeczer-cli/`, as part of the foundation deliverables. It is the **canonical reference invocation** of the Rust core (per ADR-0001) and is published to `crates.io` and as a prebuilt binary attached to GitHub Releases (PRD §27.4).

### Scope (MVP, Phase 1)

The CLI exposes the following subcommands, all backed by the `heeczer-core` crate:

- `aih schema validate <FILE|->` — validate a JSON event against `event.v1.json`. Exits non-zero with a human-readable error path on failure. Supports `--strict` (default) and `--compatibility` modes per PRD §13.
- `aih score <FILE|->` — run the scoring engine and emit the full `ScoreResult` (HEE, FEC, confidence, explainability trace) as JSON or pretty-printed table (`--format json|table`).
- `aih score --profile <PATH>` — override the default scoring profile.
- `aih score --tier <ID>` — pick a tier from the loaded tier set.
- `aih fixtures list|show <NAME>` — enumerate and emit shipped golden fixtures (useful for SDK authors to copy into their language's test suites).
- `aih diff <A.json> <B.json>` — diff two `ScoreResult`s and exit non-zero if they differ (parity helper).
- `aih migrate up|status|verify` — apply storage migrations against a configured SQLite/PostgreSQL URL (plan 0003 already calls for this under `heeczerctl`; we collapse it into `aih` to give contributors a single tool — see "Naming" below).
- `aih version` — print CLI, `scoring_version`, `spec_version`, and core crate versions for bug reports.

### Non-scope (MVP)

- No long-running server or queue worker. Use the ingestion service binary for that.
- No dashboard launcher. Use the dashboard's own dev server.
- No live tail of events. Use the ingestion service's structured logs.

### Naming

We adopt `aih` as the user-facing binary name (short, matches the product slug). The previously planned `heeczerctl` name in plan 0003 §Migrations is deprecated in favor of `aih migrate …` to give contributors **one tool**, not two. Plan 0003 will be amended in the same PR.

### Distribution

- Crate: `aih` on crates.io (binary-only crate, depends on `heeczer-core`).
- Prebuilt binaries: Linux/macOS/Windows on x86_64 and aarch64, attached to each GitHub Release, signed with cosign keyless OIDC (PRD §22 Security).
- Container: `ghcr.io/<org>/aih:<version>` for use in CI of downstream framework adapter repos.
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
- Removes `heeczerctl` as a separate binary; one tool, one set of subcommands.
- Provides a stable contract surface (`aih score`'s JSON output) that framework adapter test suites in any language can shell out to.
- Enables a "golden fixture parity" CI job that runs `aih score` on every fixture and compares against checked-in expected outputs without booting any SDK.

### Negative

- One more crate to publish per release (negligible; release-please already aggregates).
- The CLI's UX is now part of the public contract (PRD §12.15) and changes are versioned alongside `scoring_version` / `spec_version`.
- Cross-platform prebuilt-binary CI matrix grows by one artifact per platform.

### Follow-ups

- Plan 0013 (Developer Experience) gains a checklist item: CLI quickstart in README and `make cli-install` Makefile target.
- Plan 0003 (Storage) §Migrations is amended to use `aih migrate` instead of `heeczerctl migrate`.
- Plan 0012 (CI/CD) gains a `cli-build` job and a `cli-release-asset` step in the release workflow.
- A new contract test suite, `core/heeczer-cli/tests/contract/`, asserts that `aih score`'s JSON output is byte-equal to the corresponding golden fixture's expected output.

## References

- PRD §12.13 Makefile Support, §26 Developer Experience, §28.1 TDD Policy, §14.7 Scoring Contract Requirements
- ADR-0001 Rust as the Core Scoring Engine
- ADR-0007 Monorepo Tooling
- ADR-0009 Release Control Plane
- Plan 0003 §Migrations (amended by this ADR)
- Plan 0013 Developer Experience (amended by this ADR)
