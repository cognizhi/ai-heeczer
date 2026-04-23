# Plan 12 — CI/CD and release

- **Status:** Active
- **Owner:** Release Engineer
- **PRD:** §27, §12.10–§12.11
- **ADR:** ADR-0009

## Goal
Stand up the GitHub Actions pipeline as the single quality gate and release control plane, using `release-please` manifest mode.

## Checklist

### CI workflows
- [x] `lint.yml` — per-ecosystem lint matrix (Rust clippy in `ci.yml`; JS typecheck runs via pnpm + lockfile cache, Python mypy+ruff, Go vet added).
- [x] `format-check.yml` — `cargo fmt --check` in `ci.yml`.
- [x] `unit.yml` — per-ecosystem unit tests (`ci.yml`: Rust, JS vitest via pnpm, Python pytest, Go test, Java mvn test).
- [ ] `integration.yml` — ingestion service end-to-end with PG + SQLite.
- [ ] `contract.yml` — schema validation across bindings.
- [ ] `parity.yml` — fixture-driven parity across bindings.
- [ ] `migration.yml` — fresh + incremental on SQLite + PostgreSQL.
- [ ] `ui.yml` — Playwright E2E for dashboard.
- [ ] `bench-smoke.yml` — `track()` p95, ack p95, enqueue throughput.
- [x] `security.yml` — CodeQL (`codeql.yml`), cargo-audit, gitleaks, cargo-deny in `ci.yml`; Rust security checks are green locally after narrowing storage deps away from the unused `sqlx-mysql`/`rsa` path, switching the Rust SDK HTTP client to native cert roots, codifying the remaining unavoidable duplicate-version exceptions in `deny.toml` so `cargo deny check` runs warning-free, and routing the Rust security jobs through `make security-audit-ci` / `make security-licenses-ci` on an explicit stable toolchain so local repro exercises the same clean-install path as CI.
- [ ] `docs.yml` — markdown lint, link check, OpenAPI lint.
- [ ] `release-dry-run.yml` — release-please manifest computation, package dry-run on PRs.

### Release workflows
- [x] `release-please.yml` — manifest-mode PR creation on push to main; `concurrency: { group: release-please, cancel-in-progress: false }`.
- [x] `release.yml` — on tag push: build, test, publish to npm/PyPI/crates.io/Maven Central/Go tag, GitHub Release; the Rust jobs install an explicit stable toolchain, the npm path installs/tests/builds `@heeczer/sdk` via pnpm before `npm publish`, and the Go tag path keys cache off `go.mod`; `concurrency: { group: release, cancel-in-progress: false }`.
- [ ] `release-resume.yml` — workflow_dispatch to resume partial publish; same `release` concurrency group.

### Branch protection
- [ ] Required jobs documented in `docs/architecture/cicd.md`.
- [ ] Required reviewers: 1 maintainer minimum, Tech Lead for ADR/architecture changes.

### Trusted publishing
- [ ] PyPI OIDC trusted publisher configured.
- [ ] npm OIDC provenance configured.
- [ ] Sonatype token in GitHub secrets for Maven Central until OIDC support matures.

### Release manifest
- [x] `.github/release-please-config.json` and `.github/release-please-manifest.json` schema documented.
- [ ] Release workflow updates the manifest atomically per target.
- [ ] "Release complete" badge derived from manifest state.

### Docs
- [ ] `docs/architecture/cicd.md` with diagram and matrix.
- [ ] Release runbook with partial-publish recovery steps.

## Acceptance
- A dry-run release passes end-to-end on a PR.
- All required jobs are in branch protection.
