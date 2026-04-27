# Copilot / Agent Harness — ai-heeczer

This document is **mandatory reading** for any AI coding agent (GitHub Copilot, subagents, automations) working in this repository. Human contributors should also follow it.

## Repository agent files

- `AGENTS.md` — shared root entrypoint for agentic AI working in this repository.
- `CLAUDE.md` — Claude-compatible shim that delegates to `AGENTS.md` and this harness.
- `.github/copilot-instructions.md` — Copilot-compatible shim and chatmode entrypoint.
- `.github/agents/*.md` — custom role definitions and declared tool allowances.
- `docs/agents/AGENT_HARNESS.md` — the cross-role repository-wide harness, rules, and process guidance.

Keep shared repository policy in `AGENTS.md` and this harness. Keep tool-specific entrypoints thin.

## 1. Source-of-truth order

When in conflict, the order of authority is:

1. `LICENSE` (MIT) — legal.
2. `SECURITY.md` — security disclosure and response policy.
3. `docs/prd/ai-heeczer-prd.md` — product source of truth.
4. `docs/adr/*` — accepted architectural decisions (newer ADRs supersede older when they say so).
5. `docs/architecture/*` — current system documentation.
6. `docs/plan/*` — actionable implementation plans with checklists.
7. The code in `main`.

Never resolve a conflict silently. Either update the higher-authority document via PR, or stop and ask.

## 2. The non-negotiable development loop

Every behavior change follows this exact loop. No exceptions.

1. **Locate** the PRD section(s) and ADR(s) the change touches. Cite them in the PR description.
2. **Plan** — pick or create a checklist item under `docs/plan/`. Mark it in-progress.
   Updating the relevant `docs/plan/*` file in the same change is mandatory whenever the work changes status, scope, sequencing, acceptance criteria, shipped behavior, or follow-up backlog for that planned area.
3. **Contract** — if the change touches a contract (schema, API, ABI, scoring math, public SDK surface, Makefile target, dashboard data shape), update the contract doc and fixtures **first**.
4. **Failing test** — write the failing test (unit, integration, contract, parity, UI, migration, benchmark — pick the right layer per `docs/implementation/test-strategy.md`).
5. **Implement** — minimum code to pass the test. No drive-by refactors.
6. **All required tests green** locally (`make test`).
7. **Docs** — update every doc the change affects:
    - README (root or per-package)
    - architecture docs
    - relevant ADR (or write a new one)
    - relevant `docs/plan/*` file (status/checklist/backlog/links) — mandatory for every relevant change, not just at completion
    - per-binding CHANGELOG when public surface changes
    - release impact note ([template](../../.github/RELEASE_IMPACT_TEMPLATE.md)) if package contract changed (PRD §27)
8. **CI** — every test layer used has a corresponding required GitHub Actions job. If you added a new layer, add the job in the same PR and update branch protection requirements in the PR description.
9. **Conventional commits** — required for `release-please` to compute the version (ADR-0009).
10. **Mark plan checklist item done** with a link to the merged PR.

## 3. Mandatory updates per change type

| Change touches…    | Must also update                                                                                              |
| ------------------ | ------------------------------------------------------------------------------------------------------------- |
| Scoring math       | `scoring_version`, golden fixtures, all SDK parity tests, ADR-0003, scoring section in README                 |
| Event schema       | `spec_version` or new `event.vN.json`, ADR-0002, all SDK fixtures, schema docs                                |
| HTTP ingestion API | URL prefix version per PRD §12.16, OpenAPI doc, ingestion service README, dashboard data contract if affected |
| SDK public API     | per-binding README, per-binding CHANGELOG, parity tests, examples                                             |
| Database schema    | migration script (SQLite + PostgreSQL paths), migration test, `heec_schema_migrations`, data-model doc        |
| Dashboard UI       | Playwright E2E for the affected critical flow, screenshots in docs if UX language changes                     |
| CI/CD              | release runbook, branch protection list, ADR-0009 if release flow changes                                     |
| Dependencies       | license check, security audit run, SBOM regeneration on next release                                          |
| Makefile           | README quickstart, CI jobs that invoke the target, `docs/plan/0013-developer-experience.md`                   |

If a change maps to an existing plan slice and the corresponding `docs/plan/*` file is not updated in the same PR, the change is incomplete.

## 4. Subagents and when to invoke them

Subagent definitions live under `.github/agents/`. Each role file also declares the tools that role is authorized to use. If a requested tool is not available in the chosen role, do not assume it is available; either pick a more appropriate role or request the tool be added via PR.

- **Tech Lead** — design reviews, ADR drafting, plan reviews, cross-cutting trade-offs.
- **Tech Writer** — every doc PR; final pass on README and ADRs.
- **Code Reviewer** — every PR.
- **Test Engineer** — whenever a new behavior or layer is added.
- **Release Engineer** — pipeline changes, release runbooks, partial-publish recovery.
- **SDK Engineer** — any binding work (Rust, JS/TS, Python, Go, Java).
- **Security Engineer** — auth, RBAC, data handling, dependency additions, supply chain.
- **DevEx Engineer** — Makefile, bootstrap, examples, contributor onboarding.

Default chain for a non-trivial change: Test Engineer → SDK/DevEx/Security as relevant → Code Reviewer → Tech Writer → Tech Lead → Release Engineer (if release-affecting).

## 5. CI is the truth

A change is "done" only when CI says so. Required jobs include, at minimum:

- `lint` (per ecosystem)
- `format-check` (per ecosystem)
- `unit-test` (per ecosystem)
- `integration-test`
- `contract-test`
- `parity-test` (Rust + each binding against shared fixtures)
- `migration-test` (SQLite + PostgreSQL)
- `ui-test` (Playwright) when dashboard scope is touched
- `benchmark-smoke` (`track()` p95, ack p95, enqueue throughput)
- `security` (CodeQL, Trivy, language-specific audit, betterleaks)
- `release-dry-run` (`release-please` manifest computation, publish dry-run)

Any new test layer added by a PR is added to required jobs in the same PR.

## 6. Documentation drift is a P1 bug

If a README, architecture doc, ADR, or plan is wrong or stale, fix it before you fix anything else. Documentation drift blocks releases.

## 7. Observability and audit

Every state-changing operation in the ingestion service emits:

- a structured log line with `event_id`, `workspace_id`, `correlation_id`, `request_id`.
- a Prometheus counter / histogram update.
- where applicable, an audit-log row (PRD §12.9).

## 8. Privacy defaults

Never store prompt content, model output content, file attachments, secrets, or access tokens by default. Exceptions are explicit, time-boxed, and require a Security Engineer sign-off and an ADR.

## 9. License hygiene

All new dependencies must be MIT-compatible. The CI license check enforces this. Document any dependency with a non-permissive license in an ADR before adoption.

## 10. When stuck

Stop. Open a draft PR or an issue. Tag the Tech Lead. Do not invent a contract, fudge a test, or ship a "TODO" comment.
