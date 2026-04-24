# ADR-0007: Monorepo Tooling

- **Status:** Accepted
- **Date:** 2026-04-22
- **Related:** PRD §19.5, §26, §27

## Context

The repo hosts Rust core, five language SDKs, ingestion service, dashboard, examples, and shared schema fixtures. We need consistent task running, change-aware test execution, and a Makefile-friendly entrypoint.

## Decision

- **Cargo workspaces** for all Rust crates under `core/`, `bindings/*`, and `server/`.
- **pnpm workspaces** for JS/TS packages and the dashboard under `bindings/node/` and `dashboard/`.
- **uv + hatch** for the Python binding and Python examples.
- **Go modules** per binding with a top-level `go.work` for local development.
- **Maven** for the Java binding.
- **Make** as the universal entrypoint (PRD §26.2). All language tooling is invoked through Make targets.
- **`turbo` (or `nx`)** is **not** adopted in MVP; Make + per-ecosystem caches are sufficient. Revisit when CI time exceeds 20 minutes.

## Alternatives Considered

- **Bazel** — best-in-class change-aware builds; high adoption cost and steep learning curve. Deferred.
- **Nx** — strong for JS/TS-heavy monorepos; not a natural fit for Rust-first.

## Consequences

- Positive: each ecosystem uses its native tooling; no learning of an extra meta-build system.
- Negative: change-aware execution must be implemented via path-filtered GitHub Actions jobs.
- Follow-ups: define path filters in `.github/workflows/ci.yml` per ecosystem.

## References

- PRD §19.5 Monorepo and Build Architecture
- PRD §26 Developer Experience
