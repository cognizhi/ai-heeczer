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
- [x] `integration.yml` — ingestion service end-to-end with SQLite. (session Apr-2026)
- [x] `contract.yml` — schema validation across bindings. (session Apr-2026)
- [x] `parity.yml` — fixture-driven parity across bindings (per-SDK test suites). (session Apr-2026)
- [x] `migration.yml` — SQLite migration tests + PG file presence check. (session Apr-2026)
- [ ] `ui.yml` — Playwright E2E for dashboard.
- [x] `bench-smoke.yml` — CLI smoke benchmark with 30s budget. (session Apr-2026)
- [x] `security.yml` — CodeQL (`codeql.yml`), cargo-audit, gitleaks, cargo-deny in `ci.yml`; Rust security checks are green locally after narrowing storage deps away from the unused `sqlx-mysql`/`rsa` path, switching the Rust SDK HTTP client to native cert roots, and codifying the remaining unavoidable duplicate-version exceptions in `deny.toml` so `cargo deny check` runs warning-free. `codeql.yml` now runs an explicit Maven compile for `bindings/heeczer-java` between CodeQL `init` and `analyze` so Java analysis does not fail with "no source code seen during the build."
- [x] `workflow-defuser.yml` — daily scheduled/manual PR-only automation that scans `.github/workflows` for non-SHA action refs, only pins actions whose resolved release or commit is at least 14 days old, and reuses the existing automation branch/PR when present instead of creating a fresh review branch; because GitHub suppresses workflow fan-out for repository-token PR events, maintainers may need to rerun required checks manually on the generated PR.
- [x] `docs.yml` — markdown lint + rustdoc build. (session Apr-2026)
- [x] `.markdownlint.yml` created at repo root. (session Apr-2026)
- [x] `release-dry-run.yml` — release-please dry-run, publish dry-runs for Rust, npm, PyPI. (session Apr-2026)

### Release workflows

- [x] `release-please.yml` — manifest-mode PR creation on push to main; `concurrency: { group: release-please, cancel-in-progress: false }`. The Rust workspace release anchor uses a non-published root package plus concrete member crate versions/internal path-version dependencies so `release-please` can update Cargo manifests while preserving the plain `vX.Y.Z` Rust tag contract.
- [x] `release.yml` — on tag push: build, test, publish to npm/PyPI/crates.io/Maven Central/Go tag, GitHub Release; the npm path installs/tests/builds `@cognizhi/heeczer-sdk` via pnpm before `npm publish`, and the Go tag path keys cache off `go.mod`; `concurrency: { group: release, cancel-in-progress: false }`.
- [x] `release-resume.yml` — workflow_dispatch resume for partial publish; same `release` concurrency group. (session Apr-2026)

### Branch protection

- [x] Required jobs documented in `docs/architecture/cicd.md`. (§Branch protection requirements section, session Apr-2026)
- [ ] Required reviewers: 1 maintainer minimum, Tech Lead for ADR/architecture changes.

### Trusted publishing

- [x] PyPI OIDC trusted publisher configured. (workflow uses `pypa/gh-action-pypi-publish` with `id-token: write`; publisher configured on PyPI side, session Apr-2026)
- [x] npm OIDC provenance configured. (`npm publish --provenance` in `release.yml`, session Apr-2026)
- [x] Sonatype token in GitHub secrets for Maven Central until OIDC support matures. (credentials confirmed set up in GitHub, session Apr-2026)

### Release manifest

- [x] `.github/release-please-config.json` and `.github/release-please-manifest.json` schema documented.
- [ ] Release workflow updates the manifest atomically per target.
- [ ] "Release complete" badge derived from manifest state.

### Docs

- [x] `docs/architecture/cicd.md` with workflow catalog, release flow diagram, and trusted publishing setup. (session Apr-2026)
- [x] Release runbook with partial-publish recovery steps. (`docs/runbooks/release.md`, session Cat-3)

## Acceptance

- A dry-run release passes end-to-end on a PR.
- All required jobs are in branch protection.
