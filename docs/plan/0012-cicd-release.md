# Plan 12 — CI/CD and release

- **Status:** Active
- **Owner:** Release Engineer
- **PRD:** §27, §12.10–§12.11
- **ADR:** ADR-0009

## Goal
Stand up the GitHub Actions pipeline as the single quality gate and release control plane, using `release-please` manifest mode.

## Checklist

### CI workflows
- [ ] `lint.yml` — per-ecosystem lint matrix.
- [ ] `format-check.yml` — per-ecosystem formatter check.
- [ ] `unit.yml` — per-ecosystem unit tests.
- [ ] `integration.yml` — ingestion service end-to-end with PG + SQLite.
- [ ] `contract.yml` — schema validation across bindings.
- [ ] `parity.yml` — fixture-driven parity across bindings.
- [ ] `migration.yml` — fresh + incremental on SQLite + PostgreSQL.
- [ ] `ui.yml` — Playwright E2E for dashboard.
- [ ] `bench-smoke.yml` — `track()` p95, ack p95, enqueue throughput.
- [ ] `security.yml` — CodeQL (incl. SQL-injection query packs), Trivy, gitleaks, language audits.
- [ ] `docs.yml` — markdown lint, link check, OpenAPI lint.
- [ ] `release-dry-run.yml` — release-please manifest computation, package dry-run on PRs.

### Release workflows
- [ ] `release-please.yml` — manifest-mode PR creation on push to main; `concurrency: { group: release-please, cancel-in-progress: false }`.
- [ ] `release.yml` — on release-please PR merge: tag, build, sign (cosign keyless), SBOM (CycloneDX), SLSA provenance, publish to npm, PyPI, crates.io, Maven Central, Go tag, GHCR, GitHub Release; `concurrency: { group: release, cancel-in-progress: false }`.
- [ ] `release-resume.yml` — workflow_dispatch to resume partial publish; same `release` concurrency group.

### Branch protection
- [ ] Required jobs documented in `docs/architecture/cicd.md`.
- [ ] Required reviewers: 1 maintainer minimum, Tech Lead for ADR/architecture changes.

### Trusted publishing
- [ ] PyPI OIDC trusted publisher configured.
- [ ] npm OIDC provenance configured.
- [ ] Sonatype token in GitHub secrets for Maven Central until OIDC support matures.

### Release manifest
- [ ] `.github/release-manifest.json` schema documented.
- [ ] Release workflow updates the manifest atomically per target.
- [ ] "Release complete" badge derived from manifest state.

### Docs
- [ ] `docs/architecture/cicd.md` with diagram and matrix.
- [ ] Release runbook with partial-publish recovery steps.

## Acceptance
- A dry-run release passes end-to-end on a PR.
- All required jobs are in branch protection.
